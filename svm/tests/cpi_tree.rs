//! PROTOTYPE demo: `ExecutionResult::cpi_tree()` reconstructs the nested CPI
//! tree from `execution_trace`, with each frame's full instruction data and
//! account metas intact — the thing a log parse cannot recover.

use quasar_svm::token::{create_keyed_mint_account, Mint};
use quasar_svm::{
    Account, Pubkey, QuasarSvm, SPL_ASSOCIATED_TOKEN_PROGRAM_ID, SPL_TOKEN_PROGRAM_ID,
};
use solana_instruction::{AccountMeta, Instruction};

#[test]
fn ata_creation_cpi_tree_from_trace() {
    let mut svm = QuasarSvm::new()
        .with_token_program()
        .with_associated_token_program();

    let payer = Pubkey::new_unique();
    let wallet = Pubkey::new_unique();
    let mint_addr = Pubkey::new_unique();

    let payer_account = Account {
        address: payer,
        owner: quasar_svm::system_program::ID,
        lamports: 10_000_000_000,
        data: vec![],
        executable: false,
    };
    let wallet_account = Account {
        address: wallet,
        owner: quasar_svm::system_program::ID,
        lamports: 0,
        data: vec![],
        executable: false,
    };
    let mint = create_keyed_mint_account(
        &mint_addr,
        &Mint { decimals: 6, supply: 1_000_000, ..Default::default() },
    );

    let (ata_address, _bump) = Pubkey::find_program_address(
        &[wallet.as_ref(), SPL_TOKEN_PROGRAM_ID.as_ref(), mint_addr.as_ref()],
        &SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    let create_ata_ix = Instruction {
        program_id: SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(ata_address, false),
            AccountMeta::new_readonly(wallet, false),
            AccountMeta::new_readonly(mint_addr, false),
            AccountMeta::new_readonly(quasar_svm::system_program::ID, false),
            AccountMeta::new_readonly(SPL_TOKEN_PROGRAM_ID, false),
        ],
        data: vec![],
    };

    // This fixture's inner call fails (custom 0x2) by design — quasar's own ATA
    // test asserts on the trace, not success. That is fine here: the point is
    // the tree, and a failing inner call even shows the tree locating it.
    let result = svm.process_instruction(&create_ata_ix, &[payer_account, wallet_account, mint]);

    // The tree, reconstructed from the structured trace (not the logs).
    let tree = result.cpi_tree();
    println!("\n=== ExecutionResult::cpi_tree() ===\n{}", result.pretty_cpi_tree());

    // One top-level instruction (the ATA program) with an inner CPI: depth.
    assert_eq!(tree.len(), 1, "one root: the ATA-create instruction");
    let root = &tree[0];
    assert_eq!(root.program_id, SPL_ASSOCIATED_TOKEN_PROGRAM_ID);
    assert!(
        !root.children.is_empty(),
        "the ATA program CPIs inward, so the root has children"
    );

    // The richness a log parse cannot give: an inner frame carries its own
    // program id, account metas, and instruction data, pulled from the
    // TransactionContext — plus its result code, so the tree shows where it
    // halted.
    let inner = &root.children[0];
    assert!(
        !inner.accounts.is_empty(),
        "an inner CPI frame carries its account metas, not just a name"
    );
    println!(
        "inner CPI to {}: {} account metas, {} data bytes, result={}",
        inner.program_id,
        inner.accounts.len(),
        inner.data.len(),
        inner.result,
    );
}
