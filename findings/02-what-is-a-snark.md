# What is a SNARK? (ELI5)

## The Name

**SNARK** stands for: **S**uccinct **N**on-interactive **AR**gument of **K**nowledge

Let's break that down:

### Succinct
**Short and quick to verify**

Imagine you did 1000 pages of math homework. Instead of your teacher checking all 1000 pages (which takes forever), you create a tiny 1-page summary proof that shows all your work is correct. The teacher only needs to check that 1 page!

In Penumbra: The proof is about 192 bytes (from the code), no matter how complex the transaction is!

### Non-interactive
**You don't need to talk back and forth**

Like leaving a note vs. having a conversation:
- ❌ Interactive: "Did you do your homework?" "Yes!" "Show me problem 1..." "Here..." (back and forth)
- ✅ Non-interactive: You hand in your homework with a proof attached. Teacher checks it once. Done!

In Penumbra: You create a proof once and broadcast it. Anyone can verify it without asking you questions.

### Argument of Knowledge
**You prove you actually know something (not just guessing)**

Like proving you actually read a book by discussing specific plot points, not just the summary on the back cover.

In Penumbra: You prove you know the secret keys, amounts, and other private data needed to make a valid transaction.

## How SNARKs Work (Simple Version)

Think of a SNARK like a magic math compressor:

1. **The Statement**: "I have enough money to buy this candy"
2. **The Proof**: A tiny mathematical signature that proves statement #1 is true
3. **Verification**: Anyone can check the math signature is valid in milliseconds

### The Magic Trick

The SNARK lets you:
- Take a complex computation (like verifying you have 100 tokens)
- Compress it into a tiny proof (192 bytes!)
- Let anyone verify the proof super fast (milliseconds)
- Without revealing the secret data (your balance, keys, etc.)

## Real Code Example from Penumbra

In Penumbra's codebase at `crates/core/component/shielded-pool/src/spend/proof.rs`:

```rust
/// The public input for a SpendProof
pub struct SpendProofPublic {
    pub anchor: tct::Root,                    // The merkle tree root
    pub balance_commitment: balance::Commitment,  // Hidden balance
    pub nullifier: Nullifier,                 // Prevents double-spending
    pub rk: VerificationKey<SpendAuth>,       // Randomized key
}

/// The private input for a SpendProof
pub struct SpendProofPrivate {
    pub state_commitment_proof: tct::Proof,   // Secret: proof note exists
    pub note: Note,                           // Secret: the actual note
    pub v_blinding: Fr,                       // Secret: random blinding factor
    pub spend_auth_randomizer: Fr,           // Secret: key randomizer
    pub ak: VerificationKey<SpendAuth>,      // Secret: your auth key
    pub nk: NullifierKey,                    // Secret: nullifier key
}
```

**Public data**: Everyone sees the balance commitment and nullifier
**Private data**: Only you know the actual note, amount, and keys

The SNARK proves you know all the private data and it matches the public data, without revealing the private data!

## Why Use SNARKs Instead of Regular Proofs?

### Regular Proof (without SNARK):
- Size: Megabytes (huge!)
- Verification time: Minutes or longer
- Privacy: Often reveals the data you're trying to hide

### SNARK Proof:
- Size: 192 bytes (tiny!)
- Verification time: Milliseconds (fast!)
- Privacy: Reveals nothing except "the statement is true"

## In Penumbra

From the documentation I found at `/tmp/penumbra-guide/pages/dev/parameter_setup.md`:

Penumbra uses SNARKs for 7 different types of proofs:
1. **Spend** - Prove you can spend a note (443ms to generate)
2. **Output** - Prove a new note is valid (142ms to generate)
3. **Swap** - Prove a DEX swap is valid (272ms to generate)
4. **SwapClaim** - Prove you can claim swap proceeds (456ms to generate)
5. **Convert** (Undelegate Claim) - Prove delegation changes (179ms to generate)
6. **Delegator Vote** - Prove your vote is valid (443ms to generate)
7. **Nullifier Derivation** - Prove nullifier is computed correctly (17ms to generate)

## Key Takeaway

A SNARK is a **short proof that's quick to verify**, and you can create it **without revealing your secret data**. It's the core technology that makes Penumbra's privacy possible!

Think of it as: **"A tiny mathematical certificate that proves you did something correctly, without showing how you did it."**
