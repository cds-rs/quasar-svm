//! quasar-svm's SPL scenarios, run through the TestSVM quasar adapter and
//! rendered to `report/`: a token transfer, a wrong-authority reject, and an
//! ATA creation (which CPIs System + Token). `QuasarBackend::new()` preloads the
//! SPL programs, so the scenarios only fabricate state and send.

use {
    quasar_babelfish_report::{render_index, render_scenario},
    quasar_svm::{
        token::{create_keyed_mint_account, create_keyed_token_account},
        system_program, SPL_ASSOCIATED_TOKEN_PROGRAM_ID, SPL_TOKEN_PROGRAM_ID,
    },
    solana_instruction::{AccountMeta, Instruction},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    spl_token_interface::state::{Account as TokenAccount, AccountState, Mint},
    testsvm::{model::Transaction, TestSVM},
    testsvm_quasar::QuasarBackend,
};

fn fab_mint(engine: &mut QuasarBackend, name: &str, address: Pubkey, authority: Pubkey) -> Pubkey {
    let account = create_keyed_mint_account(
        &address,
        &Mint {
            mint_authority: Some(authority).into(),
            supply: 1_000_000_000,
            decimals: 9,
            is_initialized: true,
            freeze_authority: None.into(),
        },
    );
    engine.prop_at(name, &address, account.to_pair().1)
}

fn fab_token(
    engine: &mut QuasarBackend,
    name: &str,
    address: Pubkey,
    mint: Pubkey,
    owner: Pubkey,
    amount: u64,
) -> Pubkey {
    let account = create_keyed_token_account(
        &address,
        &TokenAccount {
            mint,
            owner,
            amount,
            state: AccountState::Initialized,
            ..TokenAccount::default()
        },
    );
    engine.prop_at(name, &address, account.to_pair().1)
}

/// SPL Token `Transfer` (tag 3 + u64 amount).
fn spl_transfer(source: Pubkey, dest: Pubkey, authority: Pubkey, amount: u64) -> Instruction {
    let mut data = vec![3u8];
    data.extend_from_slice(&amount.to_le_bytes());
    Instruction::new_with_bytes(
        SPL_TOKEN_PROGRAM_ID,
        &data,
        vec![
            AccountMeta::new(source, false),
            AccountMeta::new(dest, false),
            AccountMeta::new_readonly(authority, true),
        ],
    )
}

/// Associated Token Account `Create` (empty data).
fn create_ata(payer: Pubkey, ata: Pubkey, wallet: Pubkey, mint: Pubkey) -> Instruction {
    Instruction {
        program_id: SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(payer, true),
            AccountMeta::new(ata, false),
            AccountMeta::new_readonly(wallet, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(SPL_TOKEN_PROGRAM_ID, false),
        ],
        data: vec![],
    }
}

// Report: ../report/token-transfer.md
fn token_transfer(engine: &mut QuasarBackend) -> Transaction {
    let authority = engine.actor("Authority", 1_000_000_000);
    let mint = Pubkey::new_unique();
    fab_mint(engine, "Mint", mint, authority.pubkey());
    let alice = fab_token_at(engine, "Alice", mint, authority.pubkey(), 500_000);
    let bob = fab_token_at(engine, "Bob", mint, authority.pubkey(), 0);
    engine.send(&[spl_transfer(alice, bob, authority.pubkey(), 100)], &[&authority])
}

// Report: ../report/token-transfer-rejects-wrong-authority.md
fn token_transfer_rejects_wrong_authority(engine: &mut QuasarBackend) -> Transaction {
    let owner = engine.actor("Owner", 1_000_000_000);
    let mallory = engine.actor("Mallory", 1_000_000_000);
    let mint = Pubkey::new_unique();
    fab_mint(engine, "Mint", mint, owner.pubkey());
    let alice = fab_token_at(engine, "Alice", mint, owner.pubkey(), 500_000);
    let bob = fab_token_at(engine, "Bob", mint, owner.pubkey(), 0);
    // Mallory signs, but Alice is owned by Owner: the token program rejects it.
    engine.send(&[spl_transfer(alice, bob, mallory.pubkey(), 100)], &[&mallory])
}

// Report: ../report/ata-create.md
fn ata_create(engine: &mut QuasarBackend) -> Transaction {
    let payer = engine.actor("Payer", 10_000_000_000);
    let wallet = engine.actor("Wallet", 0);
    let mint = Pubkey::new_unique();
    fab_mint(engine, "Mint", mint, payer.pubkey());
    let (ata, _bump) = Pubkey::find_program_address(
        &[
            wallet.pubkey().as_ref(),
            SPL_TOKEN_PROGRAM_ID.as_ref(),
            mint.as_ref(),
        ],
        &SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    engine.register_alias(&ata, "ATA");
    engine.send(&[create_ata(payer.pubkey(), ata, wallet.pubkey(), mint)], &[&payer])
}

/// A token account at a fresh address, aliased and propped.
fn fab_token_at(
    engine: &mut QuasarBackend,
    name: &str,
    mint: Pubkey,
    owner: Pubkey,
    amount: u64,
) -> Pubkey {
    fab_token(engine, name, Pubkey::new_unique(), mint, owner, amount)
}

type Scenario = fn(&mut QuasarBackend) -> Transaction;

#[test]
fn generate_report() {
    let scenarios: &[(&str, &str, &str, Scenario)] = &[
        (
            "token_transfer",
            "Transfer SPL tokens",
            "Move 100 tokens from Alice to Bob; the owning authority signs.",
            token_transfer,
        ),
        (
            "token_transfer_rejects_wrong_authority",
            "Reject a transfer signed by the wrong authority",
            "Mallory signs a transfer of Alice's tokens; the token program rejects it because Alice is owned by someone else.",
            token_transfer_rejects_wrong_authority,
        ),
        (
            "ata_create",
            "Create an associated token account",
            "Create the payer's ATA; the program allocates the account through System and initializes it through Token, so the tree nests both CPIs.",
            ata_create,
        ),
    ];

    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/report");
    std::fs::create_dir_all(dir).unwrap();
    let mut index = Vec::new();
    for (test_fn, title, intent, run) in scenarios {
        let tx = run(&mut QuasarBackend::new());
        let page = format!("{}.md", test_fn.replace('_', "-"));
        std::fs::write(
            format!("{dir}/{page}"),
            render_scenario(title, intent, "tests/report.rs", test_fn, &tx),
        )
        .unwrap();
        let outcome = if tx.error.is_none() { "succeeded" } else { "rejected" }.to_string();
        index.push((page, title.to_string(), outcome));
    }
    std::fs::write(format!("{dir}/index.md"), render_index(&index)).unwrap();
}
