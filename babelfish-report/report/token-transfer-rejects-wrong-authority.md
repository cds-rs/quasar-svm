# Reject a transfer signed by the wrong authority

**Intent.** Mallory signs a transfer of Alice's tokens; the token program rejects it because Alice is owned by someone else.

**Outcome.** The transaction failed: `custom program error: 0x4`.

**Source.** [`tests/report.rs::token_transfer_rejects_wrong_authority`](../tests/report.rs#L97)

## Structured execution log

```
CPI Tree (4,471 BPF CU / 1,400,000 budget):
└── Transfer FAILED: custom program error: 0x4 (4,471 / 1,400,000 CU) Token (no CPIs)
      >> log:  Error: owner does not match
```

## Sequence diagram

```mermaid
sequenceDiagram
    autonumber
    participant Mallory
    participant Token
    Mallory ->> Token: Transfer (4471cu)
    rect rgb(255, 220, 220)
    note over Token: ✗ custom program error: 0x4
    end
```

## Authority graph

Who signed for what; an `invoke_signed` PDA appears as its own authority.

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    Token[Token]:::program
    Alice[(Alice)]:::writable
    Bob[(Bob)]:::writable
    Mallory([Mallory]):::signer
    Mallory -->|signs| Token
    Token -->|writes| Alice
    Token -->|writes| Bob
```

## Ownership graph

Which program owns each account the transaction wrote.

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    Token[Token]:::owner
    Alice[(Alice)]:::account
    Bob[(Bob)]:::account
    Token -->|owns| Alice
    Token -->|owns| Bob
```
