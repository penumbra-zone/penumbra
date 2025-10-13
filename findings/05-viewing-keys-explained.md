# Viewing Keys in Penumbra (ELI5)

## The Problem They Solve

On a traditional blockchain, there's ONE key:
- If you share it → ❌ People can SPEND your money
- If you don't share it → ❌ Nobody can see your transactions

This is bad for:
- Tax accountants (need to see your history, but shouldn't be able to spend)
- Auditors (need to verify compliance)
- Mobile wallets (phones are too slow to decrypt everything)

**Penumbra's solution: Multiple keys with different powers!**

## The Key Hierarchy

Think of it like hotel keys:

```
Master Key (Spend Key)
    ↓ derives
Full Viewing Key
    ↓ splits into
    ├── Incoming Viewing Key (see incoming payments)
    └── Outgoing Viewing Key (see outgoing payments)
```

### 1. Spend Key (Master Key)

**Power**: Can do EVERYTHING
- Spend money ✅
- View all transactions ✅
- Generate all other keys ✅

**Size**: 32 bytes

**Location in code**: `crates/core/keys/src/keys/spend.rs`

```rust
pub struct SpendKey {
    seed: SpendKeyBytes,           // The 32-byte seed
    ask: SigningKey<SpendAuth>,    // Authorization signing key
    fvk: FullViewingKey,           // Full viewing key (derived)
}
```

**How it's generated**: From a seed phrase using BIP39
```rust
// From seed phrase
let seed_phrase = SeedPhrase::from_str("word1 word2 ... word24");
let spend_key = SpendKey::from_seed_phrase_bip44(seed_phrase, &Bip44Path::new(0));
```

**Keep this SECRET!** If someone gets this, they can spend all your money.

---

### 2. Full Viewing Key (FVK)

**Power**: Can VIEW everything, but CANNOT spend
- Spend money ❌
- View all transactions (sent + received) ✅
- Decrypt all your notes ✅
- Generate addresses ✅

**Size**: 64 bytes (32 bytes ak + 32 bytes nk)

**Location in code**: `crates/core/keys/src/keys/fvk.rs`

```rust
pub struct FullViewingKey {
    ak: VerificationKey<SpendAuth>,  // Authorization key (verification only)
    nk: NullifierKey,                // Nullifier key (to detect spends)
    ovk: OutgoingViewingKey,         // For outgoing notes
    ivk: IncomingViewingKey,         // For incoming notes
}
```

**How it's derived**:
```rust
impl SpendKey {
    pub fn full_viewing_key(&self) -> &FullViewingKey {
        &self.fvk  // FVK is automatically derived from spend key
    }
}

// The actual derivation (from spend.rs):
let ask = SigningKey::new_from_field(
    prf::expand_ff(b"Penumbra_ExpndSd", &seed.0, &[0; 1])
);
let nk = NullifierKey(
    prf::expand_ff(b"Penumbra_ExpndSd", &seed.0, &[1; 1])
);
let fvk = FullViewingKey::from_components(ask.into(), nk);
```

**When to share**: Give to your accountant or auditor who needs to see everything you do, but you don't want them to be able to spend.

---

### 3. Incoming Viewing Key (IVK)

**Power**: Can VIEW incoming payments only
- See who sent you money ✅
- See how much you received ✅
- Decrypt received notes ✅
- Generate addresses ✅
- See outgoing payments ❌
- Spend money ❌

**Size**: 64 bytes

**Location in code**: `crates/core/keys/src/keys/ivk.rs`

```rust
pub struct IncomingViewingKey {
    pub(super) ivk: ka::Secret,      // Key agreement secret (for decryption)
    pub(super) dk: DiversifierKey,   // Diversifier key (for addresses)
}
```

**How it's derived from FVK**:
```rust
// From fvk.rs
let ivk = {
    let ak_s = Fq::from_bytes_checked(ak.as_ref()).expect("valid");
    let ivk_mod_q = poseidon377::hash_2(&IVK_DOMAIN_SEP, (nk.0, ak_s));
    ka::Secret::new_from_field(Fr::from_le_bytes_mod_order(&ivk_mod_q.to_bytes()))
};

let dk = {
    let hash_result = prf::expand(b"Penumbra_DerivDK", &nk.0.to_bytes(), ak.as_ref());
    let mut dk = [0; 16];
    dk.copy_from_slice(&hash_result.as_bytes()[0..16]);
    DiversifierKey(dk)
};

let ivk = IncomingViewingKey { ivk, dk };
```

**What it can do**:
```rust
impl IncomingViewingKey {
    // Generate a payment address
    pub fn payment_address(&self, index: AddressIndex) -> (Address, fmd::DetectionKey) {
        let d = self.dk.diversifier_for_index(&index);
        let g_d = d.diversified_generator();
        let pk_d = self.ivk.diversified_public(&g_d);
        // ... creates address
    }

    // Check if you can see an address
    pub fn views_address(&self, address: &Address) -> bool {
        self.ivk.diversified_public(address.diversified_generator())
            == *address.transmission_key()
    }
}
```

---

### 4. Outgoing Viewing Key (OVK)

**Power**: Can VIEW outgoing payments only
- See who you sent money to ✅
- See how much you sent ✅
- Decrypt sent notes ✅
- See incoming payments ❌
- Spend money ❌

**Size**: 32 bytes

**Location in code**: `crates/core/keys/src/keys/ovk.rs`

```rust
pub struct OutgoingViewingKey(pub(crate) [u8; 32]);
```

**How it's derived**:
```rust
// From fvk.rs
let ovk = {
    let hash_result = prf::expand(
        b"Penumbra_DeriOVK",
        &nk.0.to_bytes(),
        ak.as_ref()
    );
    let mut ovk = [0; 32];
    ovk.copy_from_slice(&hash_result.as_bytes()[0..32]);
    ovk
};
```

**Use case**: Less common, but useful if you want to prove "I sent this payment" without revealing what you received.

---

## The Complete Key Derivation Tree

```
Seed Phrase (24 words)
    ↓ PBKDF2 + BIP44
Spend Key Bytes (32 bytes)
    ↓ PRF expansion with domain separation
    ├─→ ask (Authorization Signing Key)
    └─→ nk (Nullifier Key)
         ↓ from_components(ask, nk)
    Full Viewing Key (64 bytes: ak + nk)
         ↓ derives
         ├─→ OVK = Hash("Penumbra_DeriOVK", nk, ak)
         └─→ IVK = Hash(IVK_DOMAIN_SEP, nk, ak_compressed)
              └─→ DK = Hash("Penumbra_DerivDK", nk, ak)[0..16]
```

**Key insight**: Everything flows DOWN from the spend key. You can derive viewing keys from the spend key, but you CANNOT go backwards (viewing key → spend key). This is one-way cryptography!

## Addresses: The Output of Keys

Using the IVK, you can generate addresses:

```rust
// Generate address for account 0
let (address, detection_key) = ivk.payment_address(AddressIndex::from(0));
```

**An address looks like**: `penumbra1...` (bech32 encoded)

**What's in an address** (from the code):
```rust
pub struct Address {
    diversifier: Diversifier,        // 16 bytes (makes addresses unlinkable)
    transmission_key: ka::Public,    // 32 bytes (for encryption)
    clue_key: fmd::ClueKey,         // 32 bytes (for fast detection)
}
```

### Address Features:

1. **Diversified**: Each account can have 2^96 different addresses, all pointing to the same account!
2. **Unlinkable**: Two addresses from the same account look completely unrelated
3. **Viewable**: Only the owner (with IVK) can tell which account an address belongs to

## The Wallet ID

From the FVK, Penumbra also derives a **Wallet ID**:

```rust
pub fn wallet_id(&self) -> WalletId {
    let hash_result = hash_2(
        &ACCOUNT_ID_DOMAIN_SEP,
        (
            self.nk.0,
            Fq::from_le_bytes_mod_order(&self.ak.to_bytes()[..]),
        ),
    );
    let hash = hash_result.to_bytes()[..32].try_into().expect("32 bytes");
    WalletId(hash)
}
```

This is a unique identifier for your wallet, similar to an account number, but derived from your keys.

## How Penumbra Uses These Keys

### When you SEND tokens:

1. Your wallet uses **Spend Key** to:
   - Sign the transaction (using `ask`)
   - Create nullifiers (using `nk`)
   - Prove you own the notes

2. Uses **OVK** to:
   - Encrypt the note for yourself (so you can see outgoing payments later)

### When you RECEIVE tokens:

1. Your wallet scans all transactions using **IVK**:
   - Attempts to decrypt each note
   - If decryption works → "This note is for me!"
   - Extracts amount, asset type, memo

2. Uses **FMD (Fuzzy Message Detection)** for faster scanning:
   - Each transaction has a "clue"
   - Your detection key can quickly filter out irrelevant transactions
   - Only try full decryption on potential matches

### When you want to SHARE your history:

- **Share FVK** → They see everything (all transactions)
- **Share IVK only** → They see only received payments
- **Share OVK only** → They see only sent payments
- **Keep Spend Key** → Nobody can spend your funds

## Extending Viewing Keys (Your Goal!)

You mentioned wanting to "extend viewing keys." Here's what that might mean:

### Option 1: Add More Granularity
Create viewing keys that show:
- Only specific asset types
- Only transactions above/below a certain amount
- Only transactions in a specific time range

### Option 2: Add More Capabilities
Create keys that can:
- Prove ownership without revealing balance
- Generate proofs on behalf of the user (delegated proving)
- Selective disclosure of individual transactions

### Option 3: Add More Efficiency
Optimize viewing keys to:
- Scan faster (better FMD)
- Use less bandwidth
- Work on weaker devices

**The key components you'd work with**:
- `crates/core/keys/src/keys/fvk.rs` - Full viewing key
- `crates/core/keys/src/keys/ivk.rs` - Incoming viewing key
- `crates/core/keys/src/keys/ovk.rs` - Outgoing viewing key
- `crates/view/src/service.rs` - View service that uses these keys

## Key Takeaway

**Penumbra's viewing keys provide granular control over who can see what**:

- **Spend Key**: Master key - keep this secret!
- **Full Viewing Key**: See everything, can't spend
- **Incoming Viewing Key**: See only received payments
- **Outgoing Viewing Key**: See only sent payments

This is called **selective disclosure** - you choose exactly what financial information to reveal and to whom!

All keys are derived cryptographically from your seed phrase using one-way functions (you can go Spend → FVK → IVK, but never backwards).
