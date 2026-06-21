# Transfer SPL tokens

**Intent.** Move 100 tokens from Alice to Bob; the owning authority signs.

**Outcome.** The transaction succeeded.

**Source.** [`tests/report.rs::token_transfer`](../tests/report.rs#L87)

## Structured execution log

```
CPI Tree (4,645 BPF CU / 1,400,000 budget):
└── Transfer (4,645 / 1,400,000 CU) Token (no CPIs)
```

## Sequence diagram

```mermaid
sequenceDiagram
    autonumber
    participant Authority
    participant Token
    Authority ->> Token: Transfer (4645cu)
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
    Authority([Authority]):::signer
    Authority -->|signs| Token
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
