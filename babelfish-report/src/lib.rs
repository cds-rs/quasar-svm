//! Report rendering for the quasar-svm execution dogfood: one Markdown page per
//! scenario (structured log, plain sequence diagram, authority + ownership
//! graphs), engine-neutral so it renders the quasar adapter's `model::Transaction`
//! exactly as it renders mollusk's or litesvm's.

use testsvm::model::Transaction;

fn fenced(lang: &str, body: &str) -> String {
    format!("```{lang}\n{}\n```\n", body.trim_end())
}

/// The 1-based line of `fn <test_fn>(` in the test file, for a `#L<n>` anchor on
/// the source link. Read at render time, so it matches the committed report.
fn test_fn_line(test_file: &str, test_fn: &str) -> Option<usize> {
    let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), test_file);
    let needle = format!("fn {test_fn}(");
    std::fs::read_to_string(path)
        .ok()?
        .lines()
        .position(|line| line.contains(&needle))
        .map(|i| i + 1)
}

/// One scenario's Markdown page: intent, outcome, a link back to the test, then
/// the four trace-sourced renders. The sequence diagram is plain (no lifelines).
pub fn render_scenario(
    title: &str,
    intent: &str,
    test_file: &str,
    test_fn: &str,
    tx: &Transaction,
) -> String {
    let outcome = match &tx.error {
        None => "succeeded".to_string(),
        Some(e) => format!("failed: `{e}`"),
    };
    let anchor = test_fn_line(test_file, test_fn)
        .map(|n| format!("#L{n}"))
        .unwrap_or_default();
    let mut md = String::new();
    md.push_str(&format!("# {title}\n\n"));
    md.push_str(&format!("**Intent.** {intent}\n\n"));
    md.push_str(&format!("**Outcome.** The transaction {outcome}.\n\n"));
    md.push_str(&format!(
        "**Source.** [`{test_file}::{test_fn}`](../{test_file}{anchor})\n\n"
    ));
    md.push_str("## Structured execution log\n\n");
    md.push_str(&fenced("", &tx.pretty_cpi_tree()));
    md.push_str("\n## Sequence diagram\n\n");
    md.push_str(tx.mermaid_string().trim_end());
    md.push_str("\n\n## Authority graph\n\n");
    md.push_str("Who signed for what; an `invoke_signed` PDA appears as its own authority.\n\n");
    md.push_str(tx.authority_graph_string().trim_end());
    md.push_str("\n\n## Ownership graph\n\n");
    md.push_str("Which program owns each account the transaction wrote.\n\n");
    md.push_str(tx.ownership_graph_string().trim_end());
    md.push('\n');
    md
}

/// The index page: one row per scenario, linking to its page.
pub fn render_index(entries: &[(String, String, String)]) -> String {
    let mut md = String::new();
    md.push_str("# quasar-svm through TestSVM: an execution report\n\n");
    md.push_str(
        "Each scenario runs an SPL program on Blueshift's quasar-svm engine through the \
         TestSVM quasar adapter, and renders what the engine witnessed: the structured \
         execution log, a plain sequence diagram, and the authority and ownership graphs. \
         Same report, a third engine.\n\n",
    );
    md.push_str("| Scenario | Outcome | Page |\n|---|---|---|\n");
    for (file, title, outcome) in entries {
        md.push_str(&format!("| {title} | {outcome} | [{file}]({file}) |\n"));
    }
    md
}
