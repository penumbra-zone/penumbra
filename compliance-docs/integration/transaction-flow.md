# Transaction Flow

## End-to-End Flow

```
+------------------------------------------------------------------+
|  1. USER INITIATES TRANSFER                                       |
|     pcli tx send <recipient> <amount> <asset>                     |
+------------------------------------------------------------------+
                              |
                              v
+------------------------------------------------------------------+
|  2. PLANNER BUILDS TRANSACTION                                    |
|     - Creates SpendPlan + OutputPlan(s)                           |
|     - Calls enrich_with_compliance()                              |
+------------------------------------------------------------------+
                              |
                              v
+------------------------------------------------------------------+
|  3. COMPLIANCE ENRICHMENT (per asset)                             |
|     a) Query registry for asset regulation status                 |
|     b) Query user/asset tree anchors and Merkle proofs            |
|     c) Generate dual ciphertexts (sender + receiver)              |
|     d) For unregulated: use BLACK_HOLE_ACK                        |
+------------------------------------------------------------------+
                              |
                              v
+------------------------------------------------------------------+
|  4. PROOF GENERATION                                              |
|     - SpendCircuit binds compliance data                          |
|     - OutputCircuit binds compliance data                         |
|     - Proves: correct encryption, valid Merkle paths              |
+------------------------------------------------------------------+
                              |
                              v
+------------------------------------------------------------------+
|  5. TRANSACTION BROADCAST                                         |
|     - Validator verifies proofs               |
|     - Ciphertexts stored in block                                 |
+------------------------------------------------------------------+
                              |
                              v
+------------------------------------------------------------------+
|  6. COMPLIANCE SCANNING                                           |
|     a) Auditor derives daily keys  (Orbis later)                  |
|     b) For each tx: try_detect_asset() with sk_detection on EPK   |
|     c) If match: decrypt with sk_core/sk_extension (Orbis later)  |
+------------------------------------------------------------------+
```

## Key Integration Points

### Planner (`crates/view/src/planner.rs`)

```rust
// Line ~964: Automatic compliance enrichment
self.enrich_with_compliance(view, &mut plan).await?;
```

The planner:
1. Scans actions for spends and outputs
2. Skips if multi-spend (>1 spend) or unbalanced
3. Fetches Merkle proofs from view service
4. Generates ciphertexts for sender and receiver
5. Sets all witness fields on plans

### SpendPlan Fields

```rust
pub compliance_path: MerklePath,        // User tree path
pub compliance_anchor: StateCommitment, // User tree root
pub compliance_ciphertext: Vec<u8>,     // Sender ciphertext
pub compliance_leaf: Option<ComplianceLeaf>,
pub compliance_ephemeral_secret: Option<Fr>,
pub is_regulated: bool,
pub counterparty_address: Option<Address>,
pub counterparty_leaf: Option<ComplianceLeaf>,
pub asset_path: MerklePath,             // Asset tree path
pub asset_anchor: StateCommitment,      // Asset tree root
```

### OutputPlan Fields

Same structure as SpendPlan but from receiver's perspective.

## Known Limitations

### Blocked Actions

Swap, Delegate, IBC, Position, CommunityPool, and Governance actions reject regulated assets. Only Spend/Output support compliance.

### Multi-Spend Transactions

```rust
// planner.rs line ~650
if spend_count > 1 {
    tracing::warn!("Skipping compliance enrichment for multi-spend");
    return Ok(());
}
```

Multi-spend transactions silently skip compliance enrichment.

### Anchor Staleness

Plans can become stale if user tree changes between planning and submission. Currently no retry logic exists.

## Source Files

| Component | Location |
|-----------|----------|
| Planner enrichment | `crates/view/src/planner.rs` lines 600-980 |
| ViewClient extension | `crates/view/src/client_compliance.rs` |
| SpendPlan | `crates/core/component/shielded-pool/src/spend/plan.rs` |
| OutputPlan | `crates/core/component/shielded-pool/src/output/plan.rs` |
