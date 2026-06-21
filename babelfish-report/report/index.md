# quasar-svm through TestSVM: an execution report

Each scenario runs an SPL program on Blueshift's quasar-svm engine through the TestSVM quasar adapter, and renders what the engine witnessed: the structured execution log, a plain sequence diagram, and the authority and ownership graphs. Same report, a third engine.

| Scenario | Outcome | Page |
|---|---|---|
| Transfer SPL tokens | succeeded | [token-transfer.md](token-transfer.md) |
| Reject a transfer signed by the wrong authority | rejected | [token-transfer-rejects-wrong-authority.md](token-transfer-rejects-wrong-authority.md) |
| Create an associated token account | succeeded | [ata-create.md](ata-create.md) |
