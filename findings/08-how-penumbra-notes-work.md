# How Penumbra Notes Work (ELI5)

## What is a Note?

A **note** is like a **digital envelope with money inside**.

In traditional blockchains:
- You have a balance: "Alice has 100 tokens"
- Everyone can see this

In Penumbra:
- You have **notes**: encrypted envelopes
- Each note contains: "X tokens for Address Y"
- Only the owner can open their envelopes
- Everyone else sees random-looking encrypted data

**Think of it like**: Instead of having a bank account balance, you have a pile of sealed envelopes, each containing different amounts of money. Only you can open them!

---

## The Structure of a Note

From the code at `crates/core/component/shielded-pool/src/note.rs`:

```rust
pub struct Note {
    value: Value,              // Amount + Asset ID (what's inside the envelope)
    rseed: Rseed,             // Random seed (32 bytes, for deriving secrets)
    address: Address,          // Who owns this note (the "To:" address)
    transmission_key_s: Fq,    // Part of the address (stored for efficiency)
}
```

Let's break down each part:

### 1. Value (The Money)
```rust
pub struct Value {
    amount: Amount,      // How much (e.g., 100)
    asset_id: AssetId,   // What kind (e.g., USDC, ETH, etc.)
}
```

**Example**:
- `amount: 50`
- `asset_id: penumbra`
- → "50 Penumbra tokens"

### 2. Rseed (Random Seed)
```rust
pub struct Rseed(pub [u8; 32]);  // 32 random bytes
```

**Why random?** This seed is used to derive:
- **Ephemeral secret key (esk)**: For encrypting this specific note
- **Note blinding factor**: For hiding the note in commitments

**From the code**:
```rust
impl Rseed {
    pub fn derive_esk(&self) -> ka::Secret {
        // Derives ephemeral secret key from seed
    }

    pub fn derive_note_blinding(&self) -> Fq {
        // Derives blinding factor from seed
    }
}
```

### 3. Address (Who Owns It)
```rust
pub struct Address {
    diversifier: Diversifier,     // 16 bytes (makes address unique)
    transmission_key: ka::Public,  // 32 bytes (for encryption)
    clue_key: fmd::ClueKey,       // 32 bytes (for fast detection)
}
```

This is the "To:" field - who can spend this note.

### 4. Transmission Key S
This is just the s-coordinate of the transmission key, stored separately for efficiency. It's used in note commitments.

---

## The Life of a Note

### Phase 1: Creation (Alice wants to pay Bob)

```rust
// Alice creates a note for Bob
let bob_note = Note::generate(
    &mut rng,           // Random number generator
    &bob_address,       // Bob's address
    Value {             // The value
        amount: 50,
        asset_id: penumbra_id,
    }
);
```

**What's generated**:
1. A random `rseed` (32 bytes)
2. From rseed → ephemeral secret key (esk)
3. From rseed → note blinding factor

**In the code** (`note.rs:174-180`):
```rust
pub fn generate(rng: &mut (impl Rng + CryptoRng), address: &Address, value: Value) -> Self {
    let rseed = Rseed::generate(rng);
    Note::from_parts(address.clone(), value, rseed)
        .expect("transmission key in address is always valid")
}
```

---

### Phase 2: Commitment (Putting it in the State)

Before sending, Alice creates a **note commitment** - a cryptographic hash that represents the note without revealing its contents.

```rust
pub fn commit(&self) -> StateCommitment {
    self::commitment(
        self.note_blinding(),           // Random blinding factor
        self.value,                     // The value (amount + asset)
        self.diversified_generator(),   // From address
        self.transmission_key_s,        // From address
        self.address.clue_key(),       // From address
    )
}
```

**The commitment function** (`note.rs:368-388`):
```rust
pub fn commitment(
    note_blinding: Fq,
    value: Value,
    diversified_generator: decaf377::Element,
    transmission_key_s: Fq,
    clue_key: &fmd::ClueKey,
) -> StateCommitment {
    let commit = poseidon377::hash_6(
        &NOTECOMMIT_DOMAIN_SEP,
        (
            note_blinding,              // Random
            value.amount.into(),        // How much
            value.asset_id.0,          // What asset
            diversified_generator.compress(),  // From address
            transmission_key_s,        // From address
            clue_key_field,           // From address
        ),
    );
    StateCommitment(commit)
}
```

**What this does**:
- Hashes all the note's data together
- Output: A single field element (looks random)
- This commitment goes into the Merkle tree!

**Properties**:
- ✅ **Hiding**: Commitment reveals nothing about the note
- ✅ **Binding**: Can't change the note without changing commitment
- ✅ **Deterministic**: Same note → same commitment

**The domain separator**:
```rust
static NOTECOMMIT_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.notecommit").as_bytes())
});
```
This ensures note commitments can't be confused with other types of commitments.

---

### Phase 3: Encryption (Sealing the Envelope)

Alice encrypts the note so only Bob can decrypt it:

```rust
pub fn encrypt(&self) -> NoteCiphertext {
    // 1. Generate ephemeral key pair
    let esk = self.ephemeral_secret_key();  // From rseed
    let epk = esk.diversified_public(&self.diversified_generator());

    // 2. Key agreement with Bob's transmission key
    let shared_secret = esk
        .key_agreement_with(self.transmission_key())
        .expect("key agreement succeeded");

    // 3. Derive encryption key from shared secret
    let key = PayloadKey::derive(&shared_secret, &epk);

    // 4. Encrypt the note plaintext
    let note_plaintext: Vec<u8> = self.into();
    let encryption_result = key.encrypt(note_plaintext, PayloadKind::Note);

    // 5. Return ciphertext (176 bytes)
    let ciphertext: [u8; NOTE_CIPHERTEXT_BYTES] = encryption_result
        .try_into()
        .expect("note encryption result fits");

    NoteCiphertext(ciphertext)
}
```

**Step-by-step**:

1. **Generate ephemeral keys** (one-time use for this note)
   ```
   esk (secret) = derive from rseed
   epk (public) = esk * diversified_generator
   ```

2. **Key agreement** (Diffie-Hellman)
   ```
   Alice computes: shared_secret = esk * bob_pk_d
   Bob can compute: shared_secret = bob_ivk * epk
   → Both get the same shared secret!
   ```

3. **Derive encryption key**
   ```
   PayloadKey = Hash(shared_secret, epk)
   ```

4. **Encrypt note**
   ```
   plaintext = note serialized (160 bytes)
   ciphertext = ChaCha20Poly1305(plaintext, PayloadKey)
   → Output: 176 bytes (160 + 16 byte auth tag)
   ```

**Constants from the code**:
```rust
pub const NOTE_LEN_BYTES: usize = 160;           // Plaintext size
pub const NOTE_CIPHERTEXT_BYTES: usize = 176;    // Ciphertext size (160 + 16)
```

---

### Phase 4: Broadcasting (Sending the Transaction)

Alice's transaction includes:

```rust
pub struct NotePayload {
    note_commitment: StateCommitment,  // The commitment (public)
    ephemeral_key: ka::Public,         // epk (public, needed for decryption)
    encrypted_note: NoteCiphertext,    // The encrypted note (176 bytes)
}
```

**What goes on chain**:
- ✅ Note commitment (goes in Merkle tree)
- ✅ Ephemeral public key (epk)
- ✅ Encrypted note
- ✅ FMD clue (for fast detection)
- ❌ **NOT** the plaintext note
- ❌ **NOT** who it's for
- ❌ **NOT** the amount

**What everyone sees**: Random-looking data!

---

### Phase 5: Detection (Is This Note For Me?)

Bob's wallet scans the chain looking for notes sent to him.

#### Step 5a: Fast Detection (FMD)

First, Bob uses **Fuzzy Message Detection** to quickly filter:

```rust
// Each transaction has a "clue"
let clue: fmd::Clue = transaction.clue;

// Bob's detection key checks it
if bob_detection_key.examine(&clue).is_potential_match() {
    // Might be for Bob! Try full decryption
} else {
    // Definitely not for Bob, skip
}
```

**This is fast!** Avoids trying to decrypt every single transaction.

**Trade-off**: False positives (some non-Bob transactions will pass this filter)

#### Step 5b: Full Decryption

If the clue matches, Bob tries to decrypt:

```rust
pub fn decrypt(
    ciphertext: &NoteCiphertext,
    ivk: &IncomingViewingKey,
    epk: &ka::Public,
) -> Result<Note, Error> {
    // 1. Key agreement using Bob's IVK
    let shared_secret = ivk
        .key_agreement_with(epk)
        .map_err(|_| Error::DecryptionError)?;

    // 2. Derive same payload key
    let key = PayloadKey::derive(&shared_secret, epk);

    // 3. Decrypt
    Note::decrypt_with_payload_key(ciphertext, &key, epk)
}
```

**Key agreement**:
```
Alice encrypted with: esk * bob_pk_d = shared_secret
Bob decrypts with:    bob_ivk * epk = shared_secret
→ Same shared secret! (Diffie-Hellman magic)
```

**Decryption** (`note.rs:327-349`):
```rust
pub fn decrypt_with_payload_key(
    ciphertext: &NoteCiphertext,
    payload_key: &PayloadKey,
    epk: &ka::Public,
) -> Result<Note, Error> {
    // Decrypt using ChaCha20Poly1305
    let plaintext = payload_key
        .decrypt(ciphertext.0.to_vec(), PayloadKind::Note)
        .map_err(|_| Error::DecryptionError)?;

    // Convert to Note struct
    let note: Note = plaintext.try_into()
        .map_err(|_| Error::DecryptionError)?;

    // IMPORTANT: Integrity check (ZIP 212)
    if note.ephemeral_public_key() != *epk {
        return Err(Error::DecryptionError);
    }

    Ok(note)
}
```

**Security check**: The ephemeral key derived from the decrypted note must match the epk from the transaction. This prevents attacks!

**If decryption succeeds** → Note is for Bob!
**If decryption fails** → Note is for someone else (or false positive from FMD)

---

### Phase 6: Spending (Bob Uses the Note)

Later, Bob wants to spend this note:

1. **Create nullifier** (prevents double-spending)
   ```rust
   let nullifier = Nullifier::derive(
       &bob_nk,              // Bob's nullifier key
       position,             // Note's position in Merkle tree
       &note_commitment,     // Note's commitment
   );
   ```

2. **Create Spend proof** (proves Bob can spend it)
   ```rust
   SpendProof::prove(
       public: SpendProofPublic {
           anchor,                  // Merkle root
           balance_commitment,      // Hidden balance
           nullifier,              // Prevents double-spend
           rk,                     // Randomized verification key
       },
       private: SpendProofPrivate {
           note,                   // The actual note (secret!)
           merkle_proof,           // Proof note is in tree
           keys,                   // Bob's keys
           // ...
       }
   )
   ```

3. **Submit transaction** with:
   - Spend proof (192 bytes)
   - Nullifier (revealed, prevents double-spend)
   - Output note(s) (encrypted, for recipients)

The chain verifies:
- ✅ Spend proof is valid
- ✅ Nullifier hasn't been seen before
- ✅ Balance is conserved (commitments sum to zero)

---

## How Note Commitments Go Into the Tree

When Alice sends a note to Bob:

1. **Note commitment** is computed: `cm = Hash(note_blinding, value, address_parts)`

2. **Inserted into TCT** (Tiered Commitment Tree):
   ```
              Root
             /    \
           ...    ...
          /  \    /  \
        cm1  cm2 cm3 [Bob's cm]
   ```

3. **Merkle proof** generated: Path from leaf to root

4. **Later**, Bob can prove "my note is in this tree" using the Merkle proof

**Why this matters**: Bob doesn't need to reveal which note is his! The Spend proof shows:
- "I have a note in this tree" ✅
- "Here's its nullifier" ✅
- **NOT**: "It's at position 12,345"
- **NOT**: "It's worth 50 tokens"

---

## Outgoing Viewing Key Encryption

Alice also encrypts the note for herself using her OVK:

```rust
pub fn encrypt_key(&self, ovk: &OutgoingViewingKey, cv: balance::Commitment) -> OvkWrappedKey {
    let esk = self.ephemeral_secret_key();
    let epk = esk.diversified_public(&self.diversified_generator());

    // Derive outgoing cipher key
    let ock = OutgoingCipherKey::derive(ovk, cv, self.commit(), &epk);

    // Get the shared secret (same one used for recipient)
    let shared_secret = esk
        .key_agreement_with(self.transmission_key())
        .expect("key agreement succeeded");

    // Encrypt the shared secret using OVK
    let encryption_result = ock.encrypt(shared_secret.0.to_vec(), PayloadKind::Note);

    OvkWrappedKey(encryption_result.try_into().expect("fits"))
}
```

**Why?** So Alice can later decrypt her own sent notes:
- Alice can use her OVK to decrypt the wrapped key
- Use wrapped key to decrypt the note
- Now Alice knows "I sent 50 tokens to Bob"

**Decryption with OVK** (`note.rs:297-310`):
```rust
pub fn decrypt_outgoing(
    ciphertext: &NoteCiphertext,
    wrapped_ovk: OvkWrappedKey,
    cm: StateCommitment,
    cv: balance::Commitment,
    ovk: &OutgoingViewingKey,
    epk: &ka::Public,
) -> Result<Note, Error> {
    // 1. Decrypt the wrapped key using OVK
    let shared_secret = Note::decrypt_key(wrapped_ovk, cm, cv, ovk, epk)?;

    // 2. Derive payload key
    let key = PayloadKey::derive(&shared_secret, epk);

    // 3. Decrypt note
    Note::decrypt_with_payload_key(ciphertext, &key, epk)
}
```

---

## Summary: The Complete Flow

```
┌─────────────────────────────────────────────────────────┐
│ ALICE (Sender)                                          │
├─────────────────────────────────────────────────────────┤
│ 1. Create Note                                          │
│    - value: 50 penumbra                                 │
│    - address: Bob's address                             │
│    - rseed: random                                      │
│                                                          │
│ 2. Commit Note                                          │
│    cm = Hash(blinding, value, address_parts)            │
│                                                          │
│ 3. Encrypt Note                                         │
│    - Generate ephemeral keys (esk, epk)                 │
│    - Key agreement: shared = esk * bob_pk               │
│    - Encrypt plaintext → ciphertext (176 bytes)         │
│                                                          │
│ 4. Create Spend Proof (for Alice's input note)          │
│                                                          │
│ 5. Broadcast Transaction                                │
│    - Spend proof                                        │
│    - Output: note commitment, epk, ciphertext           │
│    - FMD clue                                           │
│    - OVK-wrapped key (for Alice to view later)          │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│ BLOCKCHAIN                                              │
├─────────────────────────────────────────────────────────┤
│ • Verify spend proof ✓                                  │
│ • Check nullifier not spent ✓                           │
│ • Insert note commitment into TCT                       │
│ • Store encrypted note                                  │
│                                                          │
│ Public data:                                            │
│  - Note commitment (looks random)                       │
│  - Ephemeral key (looks random)                         │
│  - Ciphertext (looks random)                            │
│  - Nullifier (looks random)                             │
│                                                          │
│ Hidden:                                                 │
│  - Amount (50 penumbra) 🔒                              │
│  - Asset (penumbra) 🔒                                  │
│  - Sender (Alice) 🔒                                    │
│  - Receiver (Bob) 🔒                                    │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│ BOB (Receiver)                                          │
├─────────────────────────────────────────────────────────┤
│ 1. Scan Chain                                           │
│    - FMD quick filter (is clue for me?)                 │
│                                                          │
│ 2. Try Decryption                                       │
│    - Key agreement: shared = bob_ivk * epk              │
│    - Decrypt ciphertext                                 │
│    - Check epk integrity                                │
│                                                          │
│ 3. Success! Decrypted Note:                             │
│    - value: 50 penumbra                                 │
│    - address: Bob's address                             │
│    - rseed: (same as Alice generated)                   │
│                                                          │
│ 4. Store Note                                           │
│    - Bob's wallet balance += 50 penumbra                │
│    - Remember position in tree (for spending later)     │
└─────────────────────────────────────────────────────────┘
```

---

## Key Cryptographic Properties

### 1. Confidentiality
- Only Bob can decrypt notes sent to him (IVK decryption)
- Only Alice can view her sent notes (OVK decryption)
- Everyone else sees random data

### 2. Integrity
- Ephemeral key check prevents malleability attacks
- Note commitment binds all note data together
- Can't change note without invalidating commitment

### 3. Authenticity
- Spend proof demonstrates ownership
- Only holder of correct keys can spend

### 4. Non-Malleability
- ChaCha20Poly1305 authenticated encryption
- Commitment scheme prevents tampering

### 5. Privacy
- Notes are unlinkable (different notes look independent)
- Addresses are diversified (can't tell if same owner)
- Amounts are hidden (commitments look random)

---

## The Note Plaintext Format

From the code, a note serializes to exactly 160 bytes:

```rust
impl From<&Note> for [u8; NOTE_LEN_BYTES] {  // 160 bytes
    fn from(note: &Note) -> Self {
        let mut bytes = [0u8; NOTE_LEN_BYTES];
        bytes[0..16].copy_from_slice(&note.value.amount.to_le_bytes());
        bytes[16..48].copy_from_slice(&note.value.asset_id.0.to_bytes());
        bytes[48..80].copy_from_slice(&note.rseed.0);
        bytes[80..96].copy_from_slice(&note.diversifier().0);
        bytes[96..128].copy_from_slice(&note.transmission_key().0);
        bytes[128..160].copy_from_slice(&note.clue_key().0);
        bytes
    }
}
```

**Layout**:
- Bytes 0-15: Amount (16 bytes, u128)
- Bytes 16-47: Asset ID (32 bytes)
- Bytes 48-79: Rseed (32 bytes)
- Bytes 80-95: Diversifier (16 bytes)
- Bytes 96-127: Transmission key (32 bytes)
- Bytes 128-159: Clue key (32 bytes)
- **Total**: 160 bytes

After encryption with ChaCha20Poly1305:
- Plaintext: 160 bytes
- Ciphertext: 176 bytes (160 + 16 byte auth tag)

---

## Key Takeaways

1. **Notes are the fundamental unit** of value in Penumbra (not account balances)

2. **Encryption uses Diffie-Hellman key agreement**:
   - Sender uses ephemeral secret
   - Receiver uses their viewing key
   - Both derive same shared secret

3. **Commitments hide notes** while allowing verification

4. **FMD enables fast scanning** without full decryption

5. **Two decryption paths**:
   - Incoming: Recipient decrypts with IVK
   - Outgoing: Sender decrypts with OVK

6. **Spending requires**:
   - The note itself
   - Merkle proof it's in the tree
   - Zero-knowledge proof of ownership
   - A nullifier (to prevent double-spending)

**The beauty**: All the privacy guarantees flow from notes being encrypted and commitments being hiding!
