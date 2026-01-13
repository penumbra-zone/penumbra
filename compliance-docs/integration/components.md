# Component Integration

## Overview

This document describes every Penumbra component modified to support compliance transactions.

---

## Action Types

### Existing Actions

```
┌─────────────────────────────────────────────────────────────────┐
│                    Action Types                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  MODIFIED Actions (now carry compliance data):                  │
│  ├── Spend      - spends a note, now includes compliance_ct    │
│  └── Output     - creates a note, now includes compliance_ct   │
│                                                                 │
│  BLOCKED Actions (reject regulated assets):                     │
│  ├── Swap           - DEX swaps                                │
│  ├── SwapClaim      - claim swap outputs                       │
│  ├── Delegate       - stake to validator                       │
│  ├── Undelegate     - unstake from validator                   │
│  ├── UndelegateClaim                                           │
│  ├── IbcRelay                                                  │
│  ├── Ics20Withdrawal                                           │
│  ├── ProposalSubmit/Withdraw/Vote/DepositClaim                 │
│  ├── PositionOpen/Close/Withdraw                               │
│  └── CommunityPoolSpend/Deposit/Output                         │
│                                                                 │
│  UNMODIFIED Actions (no compliance impact):                     │
│  └── ValidatorDefinition                                       │
│                                                                 │
│  NEW Compliance Actions:                                        │
│  ├── RegisterAsset { asset_id, is_regulated }                  │
│  └── RegisterUser { leaf, signature }                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### New Action: RegisterAsset

Registers an asset's compliance status in the on-chain registry.

```rust
pub struct MsgRegisterAsset {
    pub asset_id: asset::Id,    // Which asset (e.g., uusdc)
    pub is_regulated: bool,     // true = requires compliance scanning
}
```

**Who uses it:** Asset issuers (e.g., Circle for USDC)
**Effect:** Adds asset to asset tree, marks as regulated or unregulated

### New Action: RegisterUser

Publishes a user's Address Compliance Key (ACK) on-chain.

```rust
pub struct MsgRegisterUser {
    pub leaf: ComplianceLeaf,   // address + ACK + asset_id
    pub signature: Vec<u8>,     // Proves ownership of address
}
```

**Who uses it:** Users transacting with regulated assets
**Effect:** Adds user's ACK to user tree for the specific asset

---

## Client Experience

### Client Local Storage (Before Compliance)

Clients sync compact blocks and maintain local state in SQLite:

| State | Storage | Description |
|-------|---------|-------------|
| **Pruned SCT** | `sct_hashes`, `sct_commitments` | Sparse State Commitment Tree. All commitments inserted, but only paths to user's own notes retained (`Witness::Keep` vs `Witness::Forget`). Allows local Merkle proof generation. |
| **Spendable Notes** | `spendable_notes` | note_commitment, nullifier, position, height_created, height_spent, address_index |
| **Nullifiers** | `spendable_notes.nullifier` | Derived locally when note detected: `Nullifier::derive(nk, position, commitment)` |
| **Note Plaintexts** | `notes` | Decrypted note data: address, amount, asset_id, rseed |
| **App Parameters** | `kv` table | Chain ID, fee config, epoch duration |
| **FMD Parameters** | `kv` table | Fuzzy Message Detection params for probabilistic note filtering |
| **Gas Prices** | `kv` table | Current fee estimation |
| **Asset Metadata** | `assets` table | Denominations, display info |
| **Transactions** | `tx`, `tx_by_nullifier` | Full tx bytes, nullifier→tx mapping |

### How SCT Witness Generation Works

During block sync:
```rust
// User's note → retain path data
state_commitment_tree.insert(Witness::Keep, *commitment)

// Other notes → insert but prune path data
state_commitment_tree.insert(Witness::Forget, *commitment)
```

At transaction time, `witness()` generates Merkle proofs **locally** from the pruned SCT:
```rust
// service.rs:1582 - no network call needed
sct.witness(*note_commitment)  // Returns auth_path from local tree
```

### Queries Before a Transaction (Before Compliance)

All data is local. The "queries" are to the local view service:

| Query | Source | Purpose |
|-------|--------|---------|
| `notes()` | Local SQLite | Get spendable notes with positions |
| `witness()` | Local SCT | Generate Merkle proofs from pruned tree |
| `app_params()` | Local `kv` | Chain configuration |
| `gas_prices()` | Local `kv` | Fee estimation |
| `address_by_index()` | Local derivation | Change output addresses |

**Key insight:** No network calls needed for transaction planning (after initial sync).

---

### New Compliance State

Compliance data is **NOT** stored locally. It's queried on-demand from pd:

| State | Storage | Description |
|-------|---------|-------------|
| **Compliance Anchors** | Queried | `compliance_anchor` (user tree root), `asset_anchor` (asset tree root) |
| **Merkle Proofs** | Queried | Compliance path, asset path, positions |
| **User Leaves** | Queried | Per user/asset: address + ACK + asset_id |
| **Regulation Status** | Queried | Which assets are regulated |

### New Compliance Queries (Network Calls)

These are **actual RPC calls** to pd node, unlike the SCT which is local:

| Query | When | Purpose |
|-------|------|---------|
| `compliance_asset_status(asset_id)` | Per asset | Check if regulated |
| `compliance_merkle_proofs(addr, asset)` | Per user per asset | Get proofs + anchors |
| `compliance_user_leaf(addr, asset)` | If regulated | Get registered ACK |

**Example: Alice sends USDC to Bob**
```
1. compliance_asset_status(USDC)     → is_regulated=true
2. compliance_merkle_proofs(Alice)   → sender proofs + anchors
3. compliance_user_leaf(Alice)       → sender's ACK
4. compliance_merkle_proofs(Bob)     → recipient proofs
5. compliance_user_leaf(Bob)         → recipient's ACK

Total: 5 network RPC calls (vs 0 for SCT)
```

---

### Summary: Before vs After

| Aspect | Before | After |
|--------|--------|-------|
| SCT proofs | Local (from pruned tree) | Local (unchanged) |
| Compliance proofs | N/A | **Network query** |
| Network calls to plan | 0 (after sync) | 2-5 per transaction |
| Transaction size | ~2-5 KB | ~4-9 KB |
| Registration needed | No | Yes (once per user per asset) |

---

### Future Improvement: Local Compliance Trees

The current compliance implementation queries proofs on-demand, unlike the SCT model where proofs are generated locally from a synced tree.

**To follow the SCT pattern:**

1. **Sync compliance trees** during block scanning (like SCT)
2. **Store locally** in SQLite tables: `compliance_hashes`, `compliance_commitments`, `asset_hashes`
3. **Generate proofs locally** using `Witness::Keep` for user's own registrations
4. **Eliminate network calls** during transaction planning

This would restore the "zero network calls after sync" property that the existing SCT architecture provides.

---

## Transaction Structure

### Original Spend Fields

```rust
pub struct SpendBody {
    // Original fields
    pub balance_commitment: balance::Commitment,  // Pedersen commitment to value
    pub nullifier: Nullifier,                     // Prevents double-spend
    pub rk: VerificationKey<SpendAuth>,           // Randomized auth key
    pub encrypted_backref: EncryptedBackref,      // Encrypted note reference
}
```

### Original Output Fields

```rust
pub struct OutputBody {
    // Original fields
    pub note_payload: NotePayload,                // Encrypted note for recipient
    pub balance_commitment: balance::Commitment,  // Pedersen commitment to value
    pub wrapped_memo_key: WrappedMemoKey,         // Encrypted memo key
    pub ovk_wrapped_key: OvkWrappedKey,           // For sender to decrypt later
}
```

### New Compliance Fields (Both Spend and Output)

```rust
// NEW: Added to both SpendBody and OutputBody
pub compliance_ciphertext: Vec<u8>,       // 256 bytes: EPK + encrypted data
pub target_timestamp: u64,                // Unix timestamp for daily key derivation
pub sender_leaf_hash: Fq,                 // Blinded hash of sender's leaf
pub counterparty_leaf_hash: Fq,           // Blinded hash of counterparty's leaf
pub compliance_anchor: StateCommitment,   // User tree root at tx time
pub asset_anchor: StateCommitment,        // Asset tree root at tx time
```

### Visual Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                         Transaction                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────┐   ┌─────────────────────────┐     │
│  │        Spend            │   │        Output           │     │
│  │                         │   │                         │     │
│  │  ORIGINAL:              │   │  ORIGINAL:              │     │
│  │  ├─ balance_commitment  │   │  ├─ note_payload        │     │
│  │  ├─ nullifier           │   │  ├─ balance_commitment  │     │
│  │  ├─ rk                  │   │  ├─ wrapped_memo_key    │     │
│  │  └─ encrypted_backref   │   │  └─ ovk_wrapped_key     │     │
│  │                         │   │                         │     │
│  │  NEW COMPLIANCE:        │   │  NEW COMPLIANCE:        │     │
│  │  ├─ compliance_ct       │   │  ├─ compliance_ct       │     │
│  │  ├─ target_timestamp    │   │  ├─ target_timestamp    │     │
│  │  ├─ sender_leaf_hash    │   │  ├─ sender_leaf_hash    │     │
│  │  ├─ cpty_leaf_hash      │   │  ├─ cpty_leaf_hash      │     │
│  │  ├─ compliance_anchor   │   │  ├─ compliance_anchor   │     │
│  │  └─ asset_anchor        │   │  └─ asset_anchor        │     │
│  └─────────────────────────┘   └─────────────────────────┘     │
│              │                           │                      │
│              │   same tx_blinding_nonce  │                      │
│              └───────────┬───────────────┘                      │
│                          │                                      │
│              Links spend to output in same tx                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Validator Verification

### Spend Action - Before Compliance

**Stateless (`check_stateless`):**
1. Spend auth signature verifies against `rk`
2. ZK proof verifies (SpendProof)

**Stateful (`check_and_execute`):**
1. Nullifier has not been spent
2. → Mark nullifier as spent

### Spend Action - After Compliance

**Stateless (`check_stateless`):**
1. Spend auth signature verifies against `rk`
2. **Extended ZK proof** verifies with new public inputs:
   - `asset_anchor`, `compliance_anchor`
   - `compliance_epk`, `compliance_ciphertext`
   - `target_timestamp`
   - `sender_leaf_hash`, `counterparty_leaf_hash`

**Stateful (`check_and_execute`):**
1. **NEW:** `target_timestamp` within 1 hour of block time
2. **NEW:** `compliance_anchor` matches chain's user tree root
3. **NEW:** `asset_anchor` matches chain's asset tree root
4. Nullifier has not been spent
5. → Mark nullifier as spent
6. **TODO:** Cross-action leaf hash binding validation

### Output Action - Before Compliance

**Stateless (`check_stateless`):**
1. ZK proof verifies (OutputProof)

**Stateful (`check_and_execute`):**
1. → Add note payload to SCT

### Output Action - After Compliance

**Stateless (`check_stateless`):**
1. **Extended ZK proof** verifies with new public inputs:
   - `compliance_epk`, `compliance_ciphertext`
   - `asset_anchor`, `compliance_anchor`
   - `target_timestamp`
   - `receiver_leaf_hash`, `counterparty_leaf_hash`

**Stateful (`check_and_execute`):**
1. **NEW:** `target_timestamp` within 1 hour of block time
2. **NEW:** `compliance_anchor` matches chain's user tree root
3. **NEW:** `asset_anchor` matches chain's asset tree root
4. → Add note payload to SCT
5. **TODO:** Cross-action leaf hash binding validation

### Summary Table

| Check | Before | After |
|-------|--------|-------|
| Auth signature | ✓ | ✓ |
| ZK proof | Original | Extended (+ compliance inputs) |
| Nullifier unspent | ✓ | ✓ |
| Timestamp window | - | ✓ (within 1 hour) |
| Compliance anchor | - | ✓ (must match chain) |
| Asset anchor | - | ✓ (must match chain) |
| Leaf hash binding | - | TODO (cross-action) |

### TODO: Transaction-Level Validation

Not yet implemented - from code comments:
```
For each spend/output pair:
  spend.counterparty_leaf_hash == output.receiver_leaf_hash
  output.counterparty_leaf_hash == spend.sender_leaf_hash
```

This cryptographically binds spend↔output without revealing identities.

---

## Module Changes

### 1. Keys (`crates/core/keys/`)

**Files:** `src/keys/cvk.rs` (new), `src/keys.rs` (modified)

**What changed:**
- New compliance key types: `MasterComplianceKey`, `DailyMasterKey`, `AddressComplianceKey`
- Key derivation functions for daily keys
- Detection method for O(1) asset filtering

### 2. Compliance Component (`crates/core/component/compliance/`)

**Files:** All new - `lib.rs`, `structs.rs`, `crypto.rs`, `tree.rs`, `r1cs.rs`, `registry.rs`

**What it provides:**
- On-chain registry state (asset tree, user tree)
- Encryption/decryption primitives
- Quad Merkle tree implementation
- ZK circuit constraints for compliance verification
- gRPC query service

### 3. Shielded Pool (`crates/core/component/shielded-pool/`)

**Files:** `spend/plan.rs`, `spend/proof.rs`, `spend/action.rs`, `output/plan.rs`, `output/proof.rs`, `output/action.rs`

**What changed:**
- SpendPlan/OutputPlan: New compliance fields for planning
- SpendProof/OutputProof: Extended circuits verify compliance
- SpendBody/OutputBody: New wire format fields

### 4. Transaction (`crates/core/transaction/`)

**Files:** `action.rs`, `gas.rs`, `plan/action.rs`, `view.rs`

**What changed:**
- New action variants: `RegisterAsset`, `RegisterUser`
- Gas costs for registration actions
- Action plan building for compliance actions

### 5. View Service (`crates/view/`)

**Files:** `planner.rs`, `client.rs`, `client_compliance.rs` (new), `service.rs`

**What changed:**
- `enrich_with_compliance()`: Auto-enriches plans with compliance data
- New ViewClient methods for querying registry
- gRPC handlers forward to pd node

### 6. CLI (`crates/bin/pcli/`)

**Files:** `command/tx/compliance.rs` (new), `command/tx.rs`

**What changed:**
- New subcommands: `register-asset`, `register-user`, `derive-daily-key`, `scan`
- Terminal display for compliance actions

### 7. Proto (`proto/penumbra/`)

**Files:** `compliance.proto` (new), `shielded_pool.proto`, `transaction.proto`, `view.proto`

**What changed:**
- New compliance message types and query service
- Extended Spend/Output protos with compliance fields
- New action plan variants

---

## Query Service

The compliance component exposes gRPC queries:

| RPC | Purpose |
|-----|---------|
| `ComplianceAssetStatus` | Check if asset is registered/regulated |
| `ComplianceAnchors` | Get current tree roots |
| `ComplianceMerkleProofs` | Get Merkle proofs for ZK circuits |
| `ComplianceUserLeaf` | Get user's registered ACK |

---

## Anchor Management

### What are Anchors?

Anchors are Merkle tree roots that commit to the registry state at a point in time.

| Anchor | Tree | Purpose |
|--------|------|---------|
| `compliance_anchor` | User tree | Proves user is registered |
| `asset_anchor` | Asset tree | Proves asset regulation status |

### Staleness

Plans can become stale if the registry changes between planning and submission.

**Current behavior:** Transaction fails if anchors don't match current state
**Future:** Retry logic or anchor validity windows

---

## SCT (State Commitment Tree)

The existing SCT is **not modified**. Compliance uses separate Quad Merkle trees:

| Tree | Purpose | Depth |
|------|---------|-------|
| User Tree | Stores ComplianceLeaf commitments | 16 |
| Asset Tree | Stores asset registration status | 16 |

Both use arity-4 (quad) for efficiency: 3 siblings per level instead of 1.

---

## Source Files

| Component | Location |
|-----------|----------|
| Key hierarchy | `crates/core/keys/src/keys/cvk.rs` |
| Compliance component | `crates/core/component/compliance/src/` |
| Spend modifications | `crates/core/component/shielded-pool/src/spend/` |
| Output modifications | `crates/core/component/shielded-pool/src/output/` |
| Transaction actions | `crates/core/transaction/src/action.rs` |
| Planner enrichment | `crates/view/src/planner.rs` |
| ViewClient extension | `crates/view/src/client_compliance.rs` |
| CLI commands | `crates/bin/pcli/src/command/tx/compliance.rs` |
| Protos | `proto/penumbra/penumbra/core/component/compliance/` |
