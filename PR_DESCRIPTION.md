# Compliance System (POC)

Privacy-preserving compliance for regulated assets. Issuers can scan transactions for their assets without compromising privacy of other users.

## Demo Scope

Transfers of regulated assets between registered users with on-chain compliance ciphertexts decryptable by the issuer.

**Supported:** Spend + Output actions with regulated assets
**Blocked:** All other actions (Swap, Delegate, IBC, etc.) reject regulated assets

## Compliance Flow

1. **Asset Registration** — Issuer registers asset as regulated, receives Asset Master Key (AMK)
2. **User Registration** — Each address registers with an Address Compliance Key (ACK) per asset
3. **Transaction** — Planner enriches tx with dual ciphertexts (sender + receiver), ZK proofs bind compliance data
4. **Scanning** — Issuer derives daily keys, scans blocks to detect and decrypt relevant transactions

See [compliance-flow.md](compliance-docs/integration/compliance-flow.md)

## Key Hierarchy

Four-level hierarchy using hash (one-way) and linear (delegatable) derivation:

```
AMK (Asset Master Key) — Orbis only
  └─▶ UK (User Key) — Hash derivation
       └─▶ AK (Address Key) — Linear derivation
            └─▶ Daily Keys (detection, core, extension) — Linear derivation
```

Tiered access: detection key filters by asset, core key reveals amounts, extension key reveals counterparty.

See [key-hierarchy.md](compliance-docs/architecture/key-hierarchy.md)

## Ciphertext Design

256-byte ciphertext per action (EPK + 3 encrypted segments):

| Segment | Key | Reveals |
|---------|-----|---------|
| Detection | sk_detection | asset_id |
| Core | sk_core | amount, self address |
| Extension | sk_extension | counterparty address |

Unregulated assets use BLACK_HOLE_ACK — indistinguishable but undecryptable.

See [ciphertext-design.md](compliance-docs/architecture/ciphertext-design.md)

## Registry Design

Two on-chain QuadTree Merkle trees:

- **Asset Tree** — Maps asset_id → is_regulated
- **User Tree** — Maps (address, asset) → ComplianceLeaf (address + ACK)

See [registry-design.md](compliance-docs/architecture/registry-design.md)

## Known Limitations

- Multi-spend transactions skip compliance enrichment
- Anchor staleness with no retry logic
- Linear key derivation allows detection keys to recover all key types

## Roadmap

See [compliance-docs/roadmap/](compliance-docs/roadmap/README.md)
