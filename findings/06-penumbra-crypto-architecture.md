# Penumbra's Cryptographic Architecture (ELI5)

## The Big Picture

Penumbra is a **privacy-focused blockchain** that uses several cryptographic building blocks to hide transaction details while still proving everything is valid.

Think of it like a **magic vault system**:
- You can prove you put money in ✅
- You can prove you took money out ✅
- Nobody can see how much you have inside 🔒
- Nobody can see who you're paying 🔒

## The Main Crypto Components

### 1. Elliptic Curves (The Foundation)

**What**: Special mathematical curves used for cryptography

**Penumbra uses**: **BLS12-377** and **Decaf377**

**Why BLS12-377?**
From the docs at `docs/protocol/src/crypto/proofs.md`:
- Supports pairings (needed for Groth16)
- Allows depth-1 recursion
- Future-proof (can use different proof systems later)

**Why Decaf377?**
- Built on top of BLS12-377's scalar field
- Fast elliptic curve operations
- No cofactor issues (technical detail: every point is prime-order)

**In the code**: `crates/crypto/decaf377*` has all the curve implementations

---

### 2. Hash Functions

**What**: One-way functions that take any input → fixed-size output

**Penumbra uses**:

#### Poseidon377
- **Where**: `poseidon377` crate
- **Why**: Efficient inside ZK circuits (SNARK-friendly)
- **Used for**: Hashing keys, computing nullifiers, commitments

```rust
// From fvk.rs - deriving IVK
let ivk_mod_q = poseidon377::hash_2(&IVK_DOMAIN_SEP, (nk.0, ak_s));
```

#### Domain-separated PRFs
- **Where**: `crates/core/keys/src/prf.rs`
- **Why**: Prevents key reuse across different purposes
- **Used for**: Deriving subkeys from master keys

```rust
// Different domain separators for different purposes
prf::expand(b"Penumbra_DeriOVK", ...)  // For OVK
prf::expand(b"Penumbra_DerivDK", ...)  // For Diversifier Key
prf::expand(b"Penumbra_ExpndSd", ...)  // For expanding seed
```

---

### 3. Commitment Schemes

**What**: A way to commit to a value without revealing it

**Like**: Putting a number in a locked box. You can't change it later, but nobody can see inside.

**Penumbra uses**: Pedersen commitments

#### Balance Commitments
```rust
// From spend/proof.rs
pub balance_commitment: balance::Commitment,
```

**Formula**: `Commitment = value * G + blinding * H`
- `value`: The actual amount (secret)
- `blinding`: Random factor (secret)
- `G, H`: Public generator points
- Result looks random, but proves the value is correct in ZK!

#### Note Commitments
```rust
pub note_commitment: note::StateCommitment,
```

**What it commits to**:
- Note value (amount + asset ID)
- Note address (who can spend it)
- Random seed

**Why**: Lets you prove a note exists in the tree without revealing its contents

---

### 4. Merkle Trees (State Commitment Tree)

**What**: A tree of hashes that commits to all notes in Penumbra

**Location**: `crates/crypto/tct` (Tiered Commitment Tree)

**Structure**:
```
         Root (goes on chain)
        /    \
      /        \
   Node        Node
   /  \        /  \
Note1 Note2 Note3 Note4
```

**Purpose**:
- Every note gets a leaf in the tree
- The root commits to ALL notes
- You can prove "my note is in the tree" with a merkle proof
- The proof is small (~log(n) size) even if there are billions of notes!

**In a Spend proof**:
```rust
pub struct SpendProofPrivate {
    pub state_commitment_proof: tct::Proof,  // Merkle proof
    // ...
}
```

This proves: "My note exists somewhere in this tree" without revealing which note or where!

---

### 5. Nullifiers (Prevent Double-Spending)

**What**: A unique identifier that marks a note as spent

**The Problem**: In a private system, how do you prevent someone from spending the same note twice?

**The Solution**: When you spend a note, you reveal its **nullifier**

**How it's computed**:
```rust
// From spend/proof.rs
let nullifier = Nullifier::derive(
    &nk,              // Nullifier key (secret)
    position,         // Note's position in tree
    &note_commitment, // Note's commitment
);
```

**Formula**: `Nullifier = Hash(nk, position, note_commitment)`

**Properties**:
- ✅ Unique per note
- ✅ Only the owner can compute it (needs `nk`)
- ✅ Looks random (doesn't reveal anything about the note)
- ✅ Prevents double-spending (chain rejects duplicate nullifiers)

**In the chain**: The blockchain keeps a list of all revealed nullifiers. If you try to spend twice, the second spend is rejected!

---

### 6. Key Agreement (Encryption)

**What**: How to encrypt notes so only the recipient can decrypt them

**Penumbra uses**: **Decaf377-KA** (Key Agreement)

**Location**: `crates/crypto/decaf377-ka`

**How it works** (Diffie-Hellman style):

```
Sender:
  - Has recipient's address (contains transmission_key = pk_d)
  - Generates ephemeral secret (esk)
  - Computes shared secret = esk * pk_d

Receiver:
  - Has incoming viewing key (ivk)
  - Sees transaction's ephemeral public key (epk = esk * G)
  - Computes same shared secret = ivk * epk
```

**Result**: Both derive the same shared secret, used to encrypt/decrypt the note!

```rust
// From ivk.rs
pub fn key_agreement_with(&self, pk: &ka::Public) -> Result<ka::SharedSecret, ka::Error> {
    self.ivk.key_agreement_with(pk)
}
```

---

### 7. Fuzzy Message Detection (FMD)

**What**: A way to quickly check "Is this transaction for me?" without full decryption

**Location**: `crates/crypto/decaf377-fmd`

**The Problem**: Scanning every transaction is SLOW
- You have to try decrypting every single one
- On a phone? Forget it!

**The Solution**: Each transaction includes a "clue"
```rust
pub struct Clue {
    // Encrypted hint about who the recipient is
}
```

**How it works**:
1. Sender creates a clue using recipient's clue key
2. Recipient's detection key can check: "Might be for me?" (fast!)
3. If yes → try full decryption
4. If no → skip (saves tons of time!)

**Trade-off**: False positives (precision parameter controls how many)
- High precision = fewer false positives, less privacy
- Low precision = more false positives, better privacy

**From the code** (in address.rs):
```rust
pub struct Address {
    diversifier: Diversifier,
    transmission_key: ka::Public,
    clue_key: fmd::ClueKey,  // Used for FMD!
}
```

---

### 8. Zero-Knowledge Proofs (The Magic)

**Proof System**: **Groth16** on **BLS12-377**

**The 7 Main Circuits**:

#### 1. Spend Circuit
**File**: `crates/core/component/shielded-pool/src/spend/proof.rs`
**Proves**: "I can legitimately spend this note"
**Constraints**: 35,978
**Proving time**: 433ms

**Public inputs**:
- Merkle root (anchor)
- Balance commitment
- Nullifier
- Randomized verification key (rk)

**Private inputs**:
- The note itself
- Merkle proof
- Keys (ak, nk)
- Randomizers

**What it checks**:
1. Note exists in tree (verify merkle proof)
2. Nullifier is correct
3. Balance commitment matches note value
4. You have the right keys

#### 2. Output Circuit
**File**: `crates/core/component/shielded-pool/src/output/proof.rs`
**Proves**: "This new note is valid"
**Constraints**: 13,875
**Proving time**: 142ms

**Public inputs**:
- Balance commitment (negative, to balance the spend)
- Note commitment

**Private inputs**:
- The note being created
- Balance blinding factor

**What it checks**:
1. Note commitment is correct
2. Balance commitment matches note value
3. Diversified generator is valid (not identity element)

#### 3. Swap Circuit
**Proves**: "I want to trade X tokens for Y tokens"
**Constraints**: 25,704
**Proving time**: 272ms

#### 4. SwapClaim Circuit
**Proves**: "I can claim the output of a swap"
**Constraints**: 46,656 (largest!)
**Proving time**: 456ms

#### 5. Convert Circuit
**Proves**: "I'm converting staking tokens (like undelegating)"
**Constraints**: 14,423
**Proving time**: 179ms

#### 6. Delegator Vote Circuit
**Proves**: "My vote is valid (weighted by my delegation)"
**Constraints**: 38,071
**Proving time**: 443ms

#### 7. Nullifier Derivation Circuit
**Proves**: "This nullifier was derived correctly"
**Constraints**: 394 (smallest!)
**Proving time**: 17ms (fastest!)

---

## How It All Works Together: A Transaction Example

Let's trace a simple payment: Alice sends 10 tokens to Bob

### Step 1: Alice prepares the transaction

```rust
// Alice has a note worth 10 tokens
let alice_note = Note { value: 10, address: alice_address, ... };

// Alice creates a Spend proof
let spend_proof = SpendProof::prove(
    SpendProofPublic {
        anchor: current_merkle_root,
        balance_commitment: commitment_to_10_tokens,
        nullifier: derive_nullifier(alice_nk, position, alice_note),
        rk: randomize_key(alice_ak),
    },
    SpendProofPrivate {
        note: alice_note,
        state_commitment_proof: merkle_proof,
        // ... keys and randomizers
    },
);

// Alice creates an Output proof (new note for Bob)
let bob_note = Note { value: 10, address: bob_address, ... };
let output_proof = OutputProof::prove(
    OutputProofPublic {
        balance_commitment: commitment_to_minus_10_tokens,  // Negative!
        note_commitment: commit(bob_note),
    },
    OutputProofPrivate {
        note: bob_note,
        balance_blinding: random_blinding,
    },
);
```

**Balance**: The two commitments must sum to zero!
- Spend: `+10 tokens * G + r1 * H`
- Output: `-10 tokens * G + r2 * H`
- Sum: `0 * G + (r1 + r2) * H` ← Proves conservation of value!

### Step 2: Alice broadcasts the transaction

The transaction includes:
- Spend proof (192 bytes)
- Output proof (192 bytes)
- Encrypted note for Bob
- Nullifier
- FMD clue for Bob
- Other metadata

**What's public**:
- The proofs ✅
- Nullifier ✅
- Balance commitments ✅
- Encrypted note (looks random) ✅

**What's hidden**:
- Amount (10 tokens) 🔒
- Alice's identity 🔒
- Bob's identity 🔒
- Alice's remaining balance 🔒

### Step 3: Validators verify

```rust
// Verify spend proof
Groth16::verify(spend_vk, spend_proof, spend_public_inputs)?;

// Verify output proof
Groth16::verify(output_vk, output_proof, output_public_inputs)?;

// Check balance commitments sum to zero
assert!(spend_commitment + output_commitment == zero_commitment);

// Check nullifier not already spent
assert!(!nullifiers_db.contains(nullifier));

// All checks pass → transaction is valid!
```

**Result**: Transaction is accepted without revealing any private details!

### Step 4: Bob detects and decrypts

```rust
// Bob's wallet scans the chain
for tx in new_transactions {
    // Quick FMD check
    if bob_detection_key.examine(&tx.clue).is_potential_match() {
        // Try full decryption
        if let Some(note) = bob_ivk.decrypt(&tx.encrypted_note) {
            // Success! This note is for Bob
            println!("Received {} tokens!", note.value);
        }
    }
}
```

---

## The Crypto Stack (Bottom to Top)

```
Layer 7: Transactions (Spend, Output, Swap, etc.)
           ↓
Layer 6: ZK Circuits (Groth16 proofs)
           ↓
Layer 5: Commitments & Nullifiers
           ↓
Layer 4: Merkle Trees (TCT)
           ↓
Layer 3: Encryption (Key Agreement + FMD)
           ↓
Layer 2: Hash Functions (Poseidon, PRF)
           ↓
Layer 1: Elliptic Curves (BLS12-377, Decaf377)
```

Each layer builds on the one below!

---

## The Key Insight: Privacy Through Math

Traditional blockchains: **Privacy through obscurity** (hide your identity, not your actions)

Penumbra: **Privacy through cryptography** (hide your actions, prove they're valid)

**The magic**: Zero-knowledge proofs let you prove something is correct without revealing what it is!

- ✅ Prove you have enough balance (without revealing your balance)
- ✅ Prove a transaction is valid (without revealing sender/receiver/amount)
- ✅ Prove a swap is fair (without revealing trading strategy)

---

## Where to Dive Deeper

Based on your goal of extending viewing keys, focus on:

1. **Keys**: `crates/core/keys/src/`
   - Understand key derivation
   - See how IVK decrypts notes

2. **View Service**: `crates/view/src/service.rs`
   - How scanning works
   - How FMD is used
   - Where decryption happens

3. **Cryptographic Primitives**:
   - `crates/crypto/decaf377-ka` - Key agreement
   - `crates/crypto/decaf377-fmd` - Fuzzy detection
   - PRF functions in `crates/core/keys/src/prf.rs`

4. **Note Structure**: `crates/core/component/shielded-pool/src/note.rs`
   - What's in a note
   - How encryption works

---

## Key Takeaway

Penumbra combines multiple cryptographic tools:
- **Elliptic curves** for key operations
- **Hash functions** for deriving keys and commitments
- **Merkle trees** for efficient state commitments
- **Nullifiers** for preventing double-spending
- **Commitments** for hiding values
- **Key agreement** for encryption
- **FMD** for fast scanning
- **Groth16** for zero-knowledge proofs

All working together to create a private, yet verifiable, blockchain!
