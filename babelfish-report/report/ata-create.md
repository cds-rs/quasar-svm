# Create an associated token account

**Intent.** Create the payer's ATA; the program allocates the account through System and initializes it through Token, so the tree nests both CPIs.

**Outcome.** The transaction succeeded.

**Source.** [`tests/report.rs::ata_create`](../tests/report.rs#L109)

## Structured execution log

```
CPI Tree (21,745 BPF CU / 1,400,000 budget):
└── (21,745 / 1,400,000 CU) AssociatedToken
    │ >> log:  Create
    │ >> log:  Initialize the associated token account
    ├── GetAccountDataSize (1,595 / 1,393,109 CU) Token
    │     >> log:  Program return: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA pQAAAAAAAAA=
    ├── CreateAccount System
    ├── InitializeImmutableOwner (1,405 / 1,386,604 CU) Token
    │     >> log:  Please upgrade to SPL Token 2022 for immutable owner support
    └── InitializeAccount3 (4,214 / 1,382,773 CU) Token
```

## Sequence diagram

```mermaid
sequenceDiagram
    autonumber
    participant Payer
    participant AssociatedToken
    participant Token
    participant System
    Payer ->> AssociatedToken: Create (21745cu)
    AssociatedToken ->> Token: GetAccountDataSize (1595cu)
    AssociatedToken ->> System: CreateAccount
    AssociatedToken ->> Token: InitializeImmutableOwner (1405cu)
    AssociatedToken ->> Token: InitializeAccount3 (4214cu)
```

## Authority graph

Who signed for what; an `invoke_signed` PDA appears as its own authority.

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    AssociatedToken[AssociatedToken]:::program
    Payer([Payer]):::signer
    ATA([ATA]):::signer
    Token[Token]:::program
    System[System]:::program
    Payer -->|signs| AssociatedToken
    Payer -->|signs| System
    ATA -->|signs| System
    AssociatedToken -->|writes| ATA
    Token -->|writes| ATA
```

## Ownership graph

Which program owns each account the transaction wrote.

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    System[System]:::owner
    Payer[(Payer)]:::account
    Token[Token]:::owner
    ATA[(ATA)]:::account
    System -->|owns| Payer
    Token -->|owns| ATA
```
