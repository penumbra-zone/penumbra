# Roadmap for Extending Viewing Keys

## Current State (What Exists Today)

Based on the codebase at `crates/core/keys/src/keys/`:

### Existing Keys:
1. **Spend Key** - Full control (spend + view)
2. **Full Viewing Key (FVK)** - View everything (incoming + outgoing)
3. **Incoming Viewing Key (IVK)** - View incoming only
4. **Outgoing Viewing Key (OVK)** - View outgoing only

### What They Can Do:

| Key | Spend | View Incoming | View Outgoing | Generate Addresses |
|-----|-------|---------------|---------------|-------------------|
| Spend Key | ✅ | ✅ | ✅ | ✅ |
| FVK | ❌ | ✅ | ✅ | ✅ |
| IVK | ❌ | ✅ | ❌ | ✅ |
| OVK | ❌ | ❌ | ✅ | ❌ |

---

## Ideas for Extensions

### Extension 1: Filtered Viewing Keys

**Goal**: View only transactions matching certain criteria

**Examples**:
- Asset-specific viewing key (only see USDC transactions)
- Amount-range viewing key (only see transactions > $1000)
- Time-bound viewing key (only see transactions from last month)
- Address-specific viewing key (only see transactions to/from certain addresses)

**How to implement**:
```rust
pub struct FilteredViewingKey {
    base_fvk: FullViewingKey,
    filter: ViewingFilter,
}

pub enum ViewingFilter {
    AssetId(AssetId),                    // Only this asset
    AmountRange { min: Amount, max: Amount },  // Amount filter
    TimeRange { start: u64, end: u64 },        // Time filter
    AddressSet(HashSet<Address>),              // Specific addresses
}

impl FilteredViewingKey {
    pub fn decrypt_if_matches(&self, note: &EncryptedNote) -> Option<Note> {
        // First decrypt with base FVK
        let note = self.base_fvk.decrypt(note)?;

        // Then check if it matches filter
        match &self.filter {
            ViewingFilter::AssetId(id) => {
                if note.asset_id() == *id {
                    Some(note)
                } else {
                    None
                }
            }
            // ... other filters
        }
    }
}
```

**Files to modify**:
- `crates/core/keys/src/keys/fvk.rs` - Add filtered variant
- `crates/view/src/service.rs` - Update scanning logic

**Challenge**: How to create filter without revealing the filter criteria to others?

---

### Extension 2: Delegated Viewing Keys

**Goal**: Create time-limited or revocable viewing keys

**Use case**: Give your accountant access for tax season, then revoke it

**How to implement**:
```rust
pub struct DelegatedViewingKey {
    fvk: FullViewingKey,
    valid_until: u64,              // Expiration timestamp
    delegation_signature: Signature, // Proves it's authorized
}

impl DelegatedViewingKey {
    pub fn create(
        spend_key: &SpendKey,
        duration: u64,
    ) -> Self {
        let fvk = spend_key.full_viewing_key().clone();
        let valid_until = current_time() + duration;

        // Sign the delegation
        let message = format!("delegate:{}:{}", fvk, valid_until);
        let signature = spend_key.sign(&message);

        DelegatedViewingKey {
            fvk,
            valid_until,
            delegation_signature: signature,
        }
    }

    pub fn is_valid(&self) -> bool {
        current_time() < self.valid_until
    }
}
```

**Files to create**:
- `crates/core/keys/src/keys/delegated.rs` - New key type
- `crates/core/keys/src/keys/revocation.rs` - Revocation logic

**Challenge**: Revocation requires on-chain state (can't just expire locally)

---

### Extension 3: Audit Keys (Prove Without Revealing)

**Goal**: Prove properties about your transactions without revealing the transactions

**Examples**:
- Prove "I paid my taxes" without showing all transactions
- Prove "I didn't receive more than $X" for compliance
- Prove "I traded within regulations" for exchange audit

**How to implement**:
```rust
pub struct AuditProof {
    statement: AuditStatement,
    proof: Groth16Proof,
}

pub enum AuditStatement {
    TotalReceived { min: Amount, max: Amount },
    TaxesPaid { amount: Amount },
    NoInteractionsWith { addresses: Vec<Address> },
}

// New circuit
pub struct AuditCircuit {
    // Public
    pub statement: AuditStatement,

    // Private (witness)
    pub fvk: FullViewingKey,
    pub transactions: Vec<Transaction>,
}

impl ConstraintSynthesizer<Fq> for AuditCircuit {
    fn generate_constraints(&self, cs: ConstraintSystemRef<Fq>) -> Result<()> {
        // Prove you can decrypt all transactions
        // Prove they satisfy the statement
        // Without revealing the transactions!
    }
}
```

**Files to create**:
- `crates/core/component/audit/src/circuit.rs` - New circuit
- `crates/core/keys/src/keys/audit.rs` - Audit key type
- `tools/parameter-setup/src/main.rs` - Add audit circuit to setup

**Challenge**: Very complex circuit, may have many constraints

---

### Extension 4: Hierarchical Viewing Keys

**Goal**: Derive multiple viewing keys from one master key, each with different permissions

**Use case**: Enterprise wallet with different departments

**How to implement**:
```rust
pub struct HierarchicalViewingKey {
    fvk: FullViewingKey,
    path: DerivationPath,  // Like BIP44 but for viewing keys
}

impl FullViewingKey {
    pub fn derive_child(&self, index: u32) -> HierarchicalViewingKey {
        // Use key derivation function
        let child_fvk = self.derive_hardened(index);

        HierarchicalViewingKey {
            fvk: child_fvk,
            path: DerivationPath::new(index),
        }
    }
}

// Example: Company structure
let company_fvk = spend_key.full_viewing_key();
let accounting_vk = company_fvk.derive_child(0);  // Can see all
let sales_vk = company_fvk.derive_child(1);       // Only sales
let hr_vk = company_fvk.derive_child(2);          // Only HR
```

**Files to modify**:
- `crates/core/keys/src/keys/fvk.rs` - Add derivation methods
- `crates/core/keys/src/keys/bip44.rs` - Extend for viewing keys

**Challenge**: Need to design the derivation scheme carefully (hardened vs non-hardened)

---

### Extension 5: Collaborative Viewing Keys

**Goal**: Multiple parties must cooperate to view transactions

**Use case**: Multi-sig wallet where multiple auditors must agree

**How to implement**:
```rust
pub struct ThresholdViewingKey {
    shares: Vec<ViewingKeyShare>,
    threshold: usize,  // e.g., 2-of-3
}

pub struct ViewingKeyShare {
    share_data: Vec<u8>,
    participant_id: u32,
}

impl ThresholdViewingKey {
    pub fn decrypt(&self, note: &EncryptedNote) -> Option<Note> {
        // Collect threshold shares
        let shares = self.collect_shares(self.threshold)?;

        // Combine shares to reconstruct decryption key
        let temp_key = combine_shares(shares)?;

        // Decrypt note
        temp_key.decrypt(note)
    }
}
```

**Files to use**:
- `crates/crypto/decaf377-frost/` - FROST threshold signatures (already exists!)
- Adapt for threshold decryption instead of threshold signing

**Challenge**: Need to design secure share distribution and combination

---

### Extension 6: Fast Sync Viewing Keys

**Goal**: Optimize scanning for resource-constrained devices

**Current problem**: Mobile wallets need to scan every transaction (slow!)

**Improvements**:

#### Better FMD (Fuzzy Message Detection)
```rust
pub struct EnhancedDetectionKey {
    base_dtk: fmd::DetectionKey,
    bloom_filters: Vec<BloomFilter>,  // Multiple filters for faster rejection
    prefix_hints: Vec<u8>,            // Quick prefix match
}

impl EnhancedDetectionKey {
    pub fn quick_filter(&self, tx: &Transaction) -> ScanPriority {
        // Multi-stage filtering
        if !self.bloom_filters.contains(&tx.clue) {
            return ScanPriority::Skip;  // Definitely not for us
        }

        if self.prefix_hints.matches(&tx.clue) {
            return ScanPriority::High;  // Very likely for us
        }

        ScanPriority::Normal  // Maybe for us, try decryption
    }
}
```

#### View Key Delegation to Server
```rust
pub struct ServerScanningKey {
    // Can detect transactions for you, but can't decrypt amounts
    detection_key: fmd::DetectionKey,
    cant_decrypt: (),  // Explicitly no decryption capability
}

// Client gives this to server
let server_key = fvk.derive_server_scanning_key();

// Server scans and returns candidates
let candidates = server.scan_for_notes(server_key);

// Client decrypts locally
for candidate in candidates {
    if let Some(note) = fvk.decrypt(candidate) {
        // Process note
    }
}
```

**Files to modify**:
- `crates/crypto/decaf377-fmd/` - Enhance FMD
- `crates/view/src/service.rs` - Add delegated scanning

---

## Implementation Priority (My Recommendation)

### Phase 1: Foundation (Start Here)
1. **Understand current implementation deeply**
   - Read all viewing key code
   - Trace a full scanning flow
   - Understand FMD in detail

2. **Create filtered viewing keys**
   - Simplest extension
   - High utility
   - Good learning project

### Phase 2: Optimization
3. **Improve FMD and scanning**
   - Real performance impact
   - Helps mobile wallets
   - Good balance of research + engineering

### Phase 3: Advanced Features
4. **Hierarchical viewing keys**
   - Enables enterprise use cases
   - Clean cryptographic design

5. **Delegated viewing keys**
   - Very useful for compliance
   - Requires revocation mechanism

### Phase 4: Research Territory
6. **Audit keys with ZK proofs**
   - Very complex
   - Requires new circuits
   - High research value

7. **Threshold viewing keys**
   - Complex cryptography
   - Needs careful security analysis

---

## Technical Prerequisites

### What You Need to Know:

1. **Rust** ✅ (you have this)
2. **Elliptic curve cryptography**
   - Read `findings/06-penumbra-crypto-architecture.md`
   - Understand Decaf377
3. **Key derivation**
   - PRF functions
   - Domain separation
4. **Symmetric encryption**
   - How notes are encrypted
5. **ZK circuits** (for advanced extensions)
   - Read `findings/04-what-is-a-circuit.md`
   - Study existing circuits

### Codebase Areas to Master:

1. **Keys module**: `crates/core/keys/src/`
   - Start with `keys/fvk.rs`, `keys/ivk.rs`
   - Understand the derivation hierarchy

2. **View service**: `crates/view/src/service.rs`
   - How scanning works
   - Where decryption happens

3. **Note structure**: `crates/core/component/shielded-pool/src/note.rs`
   - What's in a note
   - How encryption works

4. **FMD**: `crates/crypto/decaf377-fmd/`
   - Clue generation
   - Detection algorithm

5. **Proofs** (for advanced): `crates/core/component/shielded-pool/src/*/proof.rs`
   - Spend circuit
   - Output circuit

---

## Development Workflow

### Step 1: Local experimentation
```bash
# Create a test file
cd crates/core/keys/src/keys
touch filtered.rs

# Add to mod.rs
pub mod filtered;

# Write your filtered viewing key implementation

# Test it
cargo test -p penumbra-keys
```

### Step 2: Integration
```rust
// In your code
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_keys;

    #[test]
    fn test_filtered_viewing_key() {
        let spend_key = test_keys::spend_key();
        let fvk = spend_key.full_viewing_key();

        let filtered = FilteredViewingKey::new(
            fvk.clone(),
            ViewingFilter::AssetId(test_asset_id()),
        );

        // Create test note with matching asset
        let note = create_test_note(test_asset_id());

        // Should decrypt
        assert!(filtered.decrypt_if_matches(&note).is_some());

        // Create note with different asset
        let wrong_note = create_test_note(different_asset_id());

        // Should not decrypt
        assert!(filtered.decrypt_if_matches(&wrong_note).is_none());
    }
}
```

### Step 3: Performance testing
```bash
# Benchmark your changes
cargo bench -p penumbra-keys
```

### Step 4: Integration with view service
```bash
# Test full scanning flow
cargo test -p penumbra-view
```

---

## Resources and Next Steps

### Read These First:
1. `findings/05-viewing-keys-explained.md` - Understand current keys
2. `findings/06-penumbra-crypto-architecture.md` - Overall system
3. Penumbra Protocol Docs: https://protocol.penumbra.zone

### Code to Study:
1. **Full viewing key derivation**:
   - `crates/core/keys/src/keys/fvk.rs:76-103`

2. **IVK decryption**:
   - `crates/core/keys/src/keys/ivk.rs:81-84`

3. **Note structure**:
   - `crates/core/component/shielded-pool/src/note.rs`

4. **View service scanning**:
   - `crates/view/src/service.rs`

### Connect with Team:
- Read existing GitHub issues about viewing keys
- Check Discord discussions about privacy features
- Look for RFCs about key management

---

## A Simple First Project

**Goal**: Add a "Read-Only Viewing Key" that can view but not generate addresses

**Why**: Good learning project, useful feature, manageable scope

**Implementation**:
```rust
// In crates/core/keys/src/keys/readonly.rs
pub struct ReadOnlyViewingKey {
    fvk: FullViewingKey,
}

impl ReadOnlyViewingKey {
    pub fn from_fvk(fvk: FullViewingKey) -> Self {
        ReadOnlyViewingKey { fvk }
    }

    // Can view incoming
    pub fn decrypt_incoming(&self, note: &EncryptedNote) -> Option<Note> {
        self.fvk.incoming().decrypt(note)
    }

    // Can view outgoing
    pub fn decrypt_outgoing(&self, note: &EncryptedNote) -> Option<Note> {
        self.fvk.outgoing().decrypt(note)
    }

    // CANNOT generate addresses (this is the restriction)
    // payment_address() method is NOT included!
}
```

**Test it**:
```rust
#[test]
fn readonly_key_cannot_generate_addresses() {
    let spend_key = test_keys::spend_key();
    let fvk = spend_key.full_viewing_key();
    let readonly = ReadOnlyViewingKey::from_fvk(fvk.clone());

    // Can decrypt
    let note = create_test_note();
    assert!(readonly.decrypt_incoming(&note).is_some());

    // Cannot generate addresses (compile error - good!)
    // readonly.payment_address(0);  // This shouldn't compile!
}
```

This teaches you:
- How to create new key types
- How to restrict capabilities
- How decryption works
- How to write tests

**Then**: Gradually add more complex features!

---

## Key Takeaway

**Start small, iterate, and deeply understand each layer before moving to the next.**

The viewing key system is elegant but complex. Master the basics first:
1. How keys are derived
2. How encryption/decryption works
3. How FMD speeds up scanning

Then extend with new capabilities that maintain the security and privacy guarantees!

Good luck! 🚀
