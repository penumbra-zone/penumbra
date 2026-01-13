# Registry Design

## Overview

Two QuadTree Merkle trees store compliance state on-chain:

1. **User Registry**: Maps (address, asset) -> ComplianceLeaf
2. **Asset Registry**: Maps asset_id -> is_regulated

## User Registry

### ComplianceLeaf Structure

```rust
struct ComplianceLeaf {
    address: Address,           // User's Penumbra address
    key: AddressComplianceKey,  // ACK = MCK * B_d
    asset_id: asset::Id,        // Asset this registration applies to
}
```

### Leaf Commitment

```
commitment = poseidon_hash_4(
    domain_sep,
    g_d,              // Diversified generator
    pk_d,             // Transmission key
    ack,              // Address compliance key
    asset_id
)
```

### State Keys

| Key | Value |
|-----|-------|
| `compliance/user_tree` | Serialized QuadTree |
| `compliance/user_count` | Number of users (u64) |
| `compliance/user/{addr}/{asset}/position` | Tree position |
| `compliance/user/{addr}/{asset}/leaf` | Full ComplianceLeaf |

## Asset Registry

### Registration Commitment

```
commitment = poseidon_hash_2(
    0,
    asset_id,
    is_regulated ? 1 : 0
)
```

### State Keys

| Key | Value |
|-----|-------|
| `compliance/asset_tree` | Serialized QuadTree |
| `compliance/asset_count` | Number of assets (u64) |
| `compliance/asset/{id}/index` | Tree position |
| `compliance/asset/{id}/status` | Regulation byte (0/1) |

## QuadTree Properties

| Property | Value |
|----------|-------|
| Arity | 4 (quad-tree) |
| Depth | 16 levels |
| Max Leaves | ~4 billion |
| Hash | Poseidon377 hash_4 |
| Zero Hashes | Precomputed for empty subtrees |

## Merkle Paths

Used in ZK proofs to verify membership:

```rust
struct MerklePath {
    layers: Vec<MerklePathLayer>,  // Leaf to root
}

struct MerklePathLayer {
    siblings: [Fq; 3],  // 3 siblings per quad-tree node
    position: u8,       // 0-3 position in parent
}
```

## Genesis Initialization

At chain init:
- Staking token (penumbra) registered as **unregulated**
- TEST_USD registered as unregulated (for testing)

This prevents bootstrap problem (would need compliance for fee payment).

## Current Limitations

- No re-registration (hard error on duplicates)
- No update mechanism for existing entries
- Missing signature verification on user registration
- No governance authorization for asset registration

---

## Future Work

| Item | Current | Future |
|------|---------|--------|
| Historical anchors | Requires exact anchor match | Accept past tree roots (like SCT) |
| Asset tree mutability | Append-only, no unregister | Analyze if assets can be unregistered |
| Local tree sync | Proofs queried on-demand from pd | Sync trees locally like SCT |

## Source Files

- Registry traits: `crates/core/component/compliance/src/registry.rs`
- QuadTree: `crates/core/component/compliance/src/tree.rs`
- State keys: `crates/core/component/compliance/src/state_key.rs`
- Component: `crates/core/component/compliance/src/component/state.rs`
