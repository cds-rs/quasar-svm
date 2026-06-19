//! PROTOTYPE: reconstruct the CPI call tree from the execution trace.
//!
//! Quasar's [`ExecutionResult`] already carries `execution_trace`: a flat,
//! pre-order list of every invocation, each with its `stack_depth`, the full
//! `Instruction` (program id, account metas, data), compute units, and result.
//! Nesting that by `stack_depth` yields a CPI tree richer than a log parse:
//! every frame keeps its inner instruction data and account privileges, with no
//! truncation and no dependence on a program logging `Instruction:`.

use {
    crate::svm::{ExecutedInstruction, ExecutionResult},
    solana_instruction::AccountMeta,
    solana_pubkey::Pubkey,
};

/// A node in the CPI call tree.
#[derive(Debug, Clone)]
pub struct CpiNode {
    pub program_id: Pubkey,
    pub accounts: Vec<AccountMeta>,
    pub data: Vec<u8>,
    pub compute_units_consumed: u64,
    /// 0 = success; an error code otherwise.
    pub result: u64,
    pub children: Vec<CpiNode>,
}

impl ExecutionResult {
    /// Reconstruct the CPI call tree from `execution_trace`, nesting by
    /// `stack_depth`. Each node carries the full instruction (program id,
    /// account metas, data) that a log parse cannot recover.
    pub fn cpi_tree(&self) -> Vec<CpiNode> {
        let mut pos = 0;
        build(&self.execution_trace.instructions, &mut pos, 0)
    }

    /// Render the CPI tree as an indented call tree, one line per frame.
    pub fn pretty_cpi_tree(&self) -> String {
        let mut out = String::new();
        for node in &self.cpi_tree() {
            render(node, 0, &mut out);
        }
        out
    }
}

/// The trace is pre-order with `stack_depth`, so a recursive descent rebuilds
/// the tree: at each depth, take consecutive frames, and for each, recurse to
/// claim its deeper subtree before moving to its next sibling.
fn build(instrs: &[ExecutedInstruction], pos: &mut usize, depth: u8) -> Vec<CpiNode> {
    let mut nodes = Vec::new();
    while instrs.get(*pos).is_some_and(|e| e.stack_depth == depth) {
        let e = &instrs[*pos];
        let node = CpiNode {
            program_id: e.instruction.program_id,
            accounts: e.instruction.accounts.clone(),
            data: e.instruction.data.clone(),
            compute_units_consumed: e.compute_units_consumed,
            result: e.result,
            children: Vec::new(),
        };
        *pos += 1;
        let children = build(instrs, pos, depth + 1);
        nodes.push(CpiNode { children, ..node });
    }
    nodes
}

fn render(node: &CpiNode, depth: usize, out: &mut String) {
    use std::fmt::Write;
    let indent = "  ".repeat(depth);
    let status = if node.result == 0 { "✓" } else { "✗" };
    let _ = writeln!(
        out,
        "{indent}{status} {} [{}cu, {} accounts, {} data bytes]",
        node.program_id,
        node.compute_units_consumed,
        node.accounts.len(),
        node.data.len(),
    );
    for child in &node.children {
        render(child, depth + 1, out);
    }
}
