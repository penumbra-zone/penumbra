# Ciphertext Design

## Dual Ciphertext Architecture

Each transfer creates TWO compliance ciphertexts:

1. **Sender ciphertext** (on SpendPlan): Decryptable by sender's daily key
2. **Receiver ciphertext** (on OutputPlan): Decryptable by receiver's daily key

Both contain the same data but encrypted for different parties.

## Wire Format

**Total: 256 bytes** (32 EPK + 224 ciphertext payload)

| Field | Bytes | Description |
|-------|-------|-------------|
| EPK | 32 | Ephemeral public key `R = r * B_d` |
| Detection Tag | 32 | 1 Fq - encrypted asset_id |
| Encrypted Core | 96 | 3 Fq - encrypted (amount + self address) |
| Encrypted Extension | 96 | 3 Fq - encrypted counterparty address |

## Tiered Plaintext Layout

Three segments encrypted with different keys:

**Detection Segment (32 bytes → 1 Fq)**
| Field | Bytes |
|-------|-------|
| asset_id | 32 |

**Core Segment (80 bytes → 3 Fq at 31-byte chunks)**
| Field | Bytes |
|-------|-------|
| amount | 16 |
| self_g_d | 32 |
| self_pk | 32 |

**Extension Segment (64 bytes → 3 Fq at 31-byte chunks)**
| Field | Bytes |
|-------|-------|
| counterparty_g_d | 32 |
| counterparty_pk | 32 |

## 3-Key Tiered Encryption

Each segment uses a different shared secret derived from a dedicated daily key:

```
1. Derive 3 daily public keys (with domain separators):
   PK_detection = ACK + Hash(date, detection_domain) * B_d
   PK_core      = ACK + Hash(date, core_domain) * B_d
   PK_extension = ACK + Hash(date, extension_domain) * B_d

2. Generate single ephemeral secret: r (random Fr)
3. Compute EPK: R = r * B_d

4. Compute 3 shared secrets:
   S_detection = r * PK_detection
   S_core      = r * PK_core
   S_extension = r * PK_extension

5. Derive 3 seeds:
   seed_X = hash_2(domain, S_X, R)

6. Encrypt each segment with its seed:
   Detection: C[0]   = asset_id + hash_2(seed_detection, 0)
   Core:      C[1-3] = core[i] + hash_2(seed_core, i)
   Extension: C[4-6] = ext[i]  + hash_2(seed_extension, i)
```

### Decryption (with 3 Daily Keys)

```
1. Compute 3 shared secrets: S_X = dmk_X * R
2. Derive 3 seeds: seed_X = hash_2(domain, S_X, R)
3. Decrypt each segment:
   asset_id = C[0] - hash_2(seed_detection, 0)
   core     = C[1-3] - hash_2(seed_core, i)
   ext      = C[4-6] - hash_2(seed_extension, i)
```

### Why 3 Keys?

Different access tiers for different auditor needs:
- **Detection key only**: O(1) asset filtering without seeing amounts
- **Core key**: See transfer amounts and self address
- **Extension key**: Full access including counterparty

## BLACK_HOLE_ACK

For unregulated assets, encryption uses a special key:

```rust
BLACK_HOLE_ACK = Element::GENERATOR
```


## Randomness

Each ciphertext uses fresh ephemeral secret `r`:

```rust
let r = Fr::rand(&mut rng);
```

### Randomness Requirements

- `r` must be cryptographically random
- Same `r` must never be reused across ciphertexts
- Randomness source must be properly seeded

## Source Files

- Encryption: `crates/core/component/compliance/src/crypto.rs`
- Structs: `crates/core/component/compliance/src/structs.rs`
- Constants: compile-time assertions verify wire format sizes
