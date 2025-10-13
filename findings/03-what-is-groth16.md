# What is Groth16? (ELI5)

## The Simple Answer

**Groth16 is a specific recipe for making SNARKs.** Just like there are different recipes for making cookies (chocolate chip, oatmeal, etc.), there are different ways to make SNARKs. Groth16 is one popular recipe.

Think of it like:
- SNARK = "a short proof" (the general idea)
- Groth16 = "one specific way to make short proofs" (a specific recipe)

## Why It's Called "Groth16"

- **Groth** = The inventor's last name (Jens Groth)
- **16** = The year it was published (2016)

It's like how Caesar salad is named after Caesar!

## What Makes Groth16 Special?

From the Penumbra documentation I found at `docs/protocol/src/crypto/proofs.md`:

### Pros (Good Things):
1. **Very fast verification** - Super quick to check proofs
2. **Tiny proofs** - Only 192 bytes! (found in code: `GROTH16_PROOF_LENGTH_BYTES`)
3. **Mature and well-tested** - Used by Zcash (Sapling) since 2018

### Cons (Trade-offs):
1. **Needs a setup ceremony** - More on this below!
2. **One setup per circuit** - Each type of proof needs its own setup

## The Setup Ceremony (Trusted Setup)

This is the tricky part of Groth16. Before you can use it, you need to do a "setup ceremony."

### The Simple Analogy

Imagine creating a special lock-and-key system for a building:
- You need to create a "master pattern" that will generate all the locks and keys
- While creating this master pattern, you use some random secret numbers
- **If someone learns those secret numbers, they could create fake keys!**
- So you **destroy** those secret numbers after setup
- As long as ONE person involved in setup is honest and destroys their secrets, the system is secure

### In Penumbra

From `tools/parameter-setup/src/main.rs`, Penumbra does a setup for each proof type:

```rust
// Generate the parameters for each proof circuit:
let (spend_pk, spend_vk) = generate_parameters::<SpendCircuit>();
let (output_pk, output_vk) = generate_parameters::<OutputCircuit>();
let (swap_pk, swap_vk) = generate_parameters::<SwapCircuit>();
// ... etc for each proof type
```

Each circuit gets two keys:
1. **Proving Key (PK)** - Used to CREATE proofs (big file: ~48MB for Spend)
2. **Verifying Key (VK)** - Used to CHECK proofs (small file: ~1KB)

### The Security Requirement: "1-of-N Trust"

**Here's the critical part**: The setup is secure as long as **AT LEAST ONE** participant honestly destroys their secret randomness.

This is called the **"1-of-N trust assumption"**:
- If you have N participants in the ceremony
- And AT LEAST 1 of them is honest (destroys their secrets)
- Then the setup is secure!
- **All N participants would need to be dishonest** (or compromised) to break the system

#### Your Question: Small Penumbra Network

**Q: If I set up a small Penumbra network, as long as I can prove 1 person does it right (myself), then we are good?**

**A: YES!** If you run the ceremony yourself and properly destroy the toxic waste, your network is secure.

Here's why:
```
Participants: You (1 person)
Honest participants needed: 1
Honest participants: You ✓

Result: Setup is secure!
```

**For your development/testing network:**
1. Run the parameter setup yourself
2. Generate the proving and verifying keys
3. **Destroy the random values used during generation** (this is crucial!)
4. Your network is now secure (assuming you were honest)

**From the Penumbra code** (`tools/parameter-setup/src/main.rs`):
```rust
// The ceremony generates random values
let phase1contribution = Phase1Contribution::make(&mut rng, ...);

// After the keys are generated, the random values (rng state) are dropped
// This is the "destruction of toxic waste"
```

In Rust, when variables go out of scope, they're dropped. But for maximum security, you'd want to explicitly zero the memory.

#### Multi-Party Ceremony (Production Networks)

For a **production network** (like mainnet Penumbra), you'd want:

```
Participants: 100+ people from different organizations
Honest participants needed: 1
Likely honest participants: Most of them

Result: Very high confidence the setup is secure!
```

**Why have many participants?**
- **Trust distribution**: You don't need to trust any single person
- **Defense in depth**: Even if 99 are compromised, just 1 honest person keeps it secure
- **Public confidence**: More participants = more trust from users

**Famous examples**:
- **Zcash Sapling ceremony**: 90 participants from around the world
- **Ethereum KZG ceremony**: 140,000+ participants!
- **Penumbra would likely do similar**: Many independent participants

#### The Ceremony Process (Multi-Party)

In a multi-party ceremony:

```
Round 1: Alice generates randomness → creates partial parameters → destroys randomness
           ↓
         (Alice publishes her contribution)
           ↓
Round 2: Bob takes Alice's output → adds his randomness → destroys randomness
           ↓
         (Bob publishes his contribution)
           ↓
Round 3: Carol takes Bob's output → adds her randomness → destroys randomness
           ↓
         ... continues ...
           ↓
Round N: Final participant → Final parameters (Proving Key & Verifying Key)
```

**Security**: The setup is broken ONLY if **every single participant** kept their secrets. If even ONE person destroyed their randomness, the whole setup is secure!

**Think of it like**: A chain of locked boxes. To open the final box, you'd need EVERY key from EVERY participant. If even one person destroyed their key, the box stays locked forever.

**Visual of the trust model:**

```
Single Participant (You):
  You → [Generate] → Parameters
  ✓ Secure IF you destroy secrets

Multi-Party Ceremony (Production):
  Alice → [Add randomness] → Partial params
    ↓
  Bob → [Add randomness] → Partial params
    ↓
  Carol → [Add randomness] → Partial params
    ↓
  ... 97 more people ...
    ↓
  Final → Parameters

  ✓ Secure IF ANY ONE person destroyed secrets
  ✗ Broken ONLY IF ALL 100 kept secrets (extremely unlikely!)
```

**Attack scenario** (what would need to happen):
```
For 1 participant:
  - You → Must be dishonest/compromised (100% of participants)

For 100 participants:
  - Alice → Must be dishonest/compromised
  - AND Bob → Must be dishonest/compromised
  - AND Carol → Must be dishonest/compromised
  - AND ... all 97 others → Must be dishonest/compromised

  Probability: Essentially zero!
```

#### What You Need to Do (Small Network)

For your small Penumbra network:

1. **Generate parameters**:
   ```bash
   cd tools/parameter-setup
   cargo run --release --bin penumbra-parameter-setup
   ```

2. **What happens**:
   - Generates random values (the "toxic waste")
   - Creates proving keys (big files, ~48MB each)
   - Creates verifying keys (small files, ~1KB each)
   - Random values are dropped when program exits

3. **Verify integrity** (optional but good practice):
   ```bash
   # Check the parameter IDs
   cat proof-params/src/gen/spend_id.rs
   # Shows: PROVING_KEY_ID and VERIFICATION_KEY_ID
   ```

4. **Deploy**:
   - Give proving keys to users (for creating proofs)
   - Give verifying keys to validators (for checking proofs)
   - Keep a copy for yourself

5. **Security note**:
   - The random values are gone after generation
   - Even you can't recreate them (unless you kept the RNG seed)
   - For dev/testing: This is fine!
   - For production: You'd want witnesses to verify you destroyed everything

#### Trust Model Comparison

| Scenario | Participants | Trust Assumption | Suitable For |
|----------|--------------|------------------|--------------|
| **You alone** | 1 (yourself) | You trust yourself | Dev/test networks |
| **Small team** | 3-5 people | Trust at least 1 teammate | Internal/private networks |
| **Public ceremony** | 50-100+ people | Trust at least 1 stranger | Production networks |
| **Massive ceremony** | 1000s of people | Trust at least 1 human | Maximum confidence |

#### The "Toxic Waste"

**What is toxic waste?**

During the ceremony, you generate random values. Let's call them `τ` (tau):

```
Proving Key = f(circuit, τ)
Verifying Key = g(circuit, τ)
```

If someone keeps `τ`, they can:
- ❌ Create fake proofs that look valid
- ❌ Prove things that aren't true (like "I have 1 million tokens")
- ❌ Break the entire system

**Why is it called "toxic"?**
- Like radioactive waste - extremely dangerous if not disposed of properly!
- Must be destroyed after the ceremony
- Can never be recreated or recovered

**How to destroy it?**
1. Generate the keys using `τ`
2. Overwrite `τ` with zeros in memory
3. Delete any files containing `τ`
4. Restart your computer (clears RAM)

In practice, Rust's memory safety helps here - when variables go out of scope, they're dropped.

#### Can You Verify Someone Destroyed Their Secrets?

**No!** This is the fundamental challenge.

You **cannot prove** someone destroyed their secrets. You can only:
- ✅ Trust them to do so
- ✅ Use hardware security (HSMs that guarantee deletion)
- ✅ Use many participants (so you only need to trust one)
- ✅ Have public commitments and transparency

**This is why "1-of-N" is powerful**: With 100 participants, even if you can't verify each one, the probability that ALL 100 are dishonest is extremely low.

#### For Your Use Case

Since you're asking about a **small Penumbra network**:

**Development/Testing:**
- ✅ You run the ceremony yourself
- ✅ Generate parameters with the provided tool
- ✅ No need for multiple participants
- ✅ Trust yourself to not keep the secrets

**Production (if this becomes public):**
- ⚠️  Consider a multi-party ceremony
- ⚠️  Invite multiple independent parties
- ⚠️  Document the process publicly
- ⚠️  Build community trust

**Example script for your setup:**
```bash
#!/bin/bash
# Generate Penumbra parameters for your network

echo "Generating Groth16 parameters..."
echo "This will create proving and verifying keys for all circuits."

cd tools/parameter-setup
cargo run --release --bin penumbra-parameter-setup

echo "✓ Parameters generated!"
echo "✓ Random values used in generation have been destroyed."
echo ""
echo "Generated files:"
ls -lh ../../crates/crypto/proof-params/src/gen/*.bin
ls -lh ../../crates/crypto/proof-params/src/gen/*.param

echo ""
echo "Your network is ready to use!"
```

#### TL;DR - Direct Answer to Your Question

**Q: For a small Penumbra network, as long as I can prove 1 person does it right (myself), then we are good?**

**A: YES! ✅**

Here's the guarantee:
- **1-of-N trust**: Need only 1 honest participant
- **You = 1 participant**: You are that 1 honest participant
- **Therefore**: Your network is secure

**What you need to do:**
1. Run: `cargo run --release --bin penumbra-parameter-setup`
2. Don't save the random values (Rust drops them automatically)
3. Done! Your network is secure.

**Why this works:**
- The toxic waste (random τ) is in RAM during generation
- After generation, Rust drops the variables
- Even you can't recover τ (unless you saved it)
- No one can create fake proofs

**When you need more participants:**
- Dev/test network: 1 person (you) is fine ✅
- Private network: 3-5 people is good ✅
- Public network: 50-100+ for trust ⚠️
- Mainnet: 100s-1000s for maximum confidence ⚠️

**Key insight**: More participants = more trust, but you only need ONE honest participant for security. For your small network, you are that one person!

### The Two-Phase Ceremony

From the docs at `pages/dev/parameter_setup.md`, the setup has two phases:

**Phase 1**: Creates general parameters (can be reused)
- Takes ~71 seconds
- Checking takes ~147 seconds

**Phase 2**: Creates circuit-specific parameters
- Takes ~14 seconds
- Checking takes ~0.21 seconds

**Transition Phase**: Converts Phase 1 results to Phase 2 format
- Takes ~131 seconds

### Why Multiple Setups?

Each type of proof needs its own setup:
- Spend proof → needs its own setup
- Output proof → needs its own setup
- Swap proof → needs its own setup
- etc.

This is one of the downsides of Groth16! Other SNARK systems like PLONK have a "universal setup" that works for all circuits, but Groth16 is faster and has smaller proofs.

## The Math (Very Simplified)

Groth16 uses something called **pairings** on elliptic curves. Here's a super simplified view:

1. Take your secret information (private inputs)
2. Run it through a "circuit" (like a recipe of math operations)
3. Use the proving key to create a tiny proof
4. Anyone can use the verifying key to check your proof

The magic is that the proof is:
- Only 3 elliptic curve points (3 × 64 bytes = 192 bytes total)
- Super fast to verify (just a few pairing checks)

## What Curve Does Penumbra Use?

From the code at `tools/parameter-setup/src/main.rs`:

```rust
use decaf377::Bls12_377;

fn generate_parameters<D: DummyWitness>()
    -> (ProvingKey<Bls12_377>, VerifyingKey<Bls12_377>)
```

**Penumbra uses BLS12-377!**

From the reasoning in `docs/protocol/src/crypto/proofs.md`:
- **BLS12-377** was chosen over BLS12-381
- Why? It supports **depth-1 recursion** (proving things about proofs)
- This gives future flexibility without changing the whole system

## Groth16 in Action

When you make a transaction in Penumbra:

1. Your wallet creates a circuit showing "I can spend this note"
2. Uses the **proving key** to make a 192-byte Groth16 proof
3. Broadcasts the proof to the network
4. Validators use the **verifying key** to check it's valid
5. They learn nothing except "yes, this person can spend"

## Alternatives to Groth16

From the docs, other SNARK options exist:

| System | Proof Size | Setup | Recursion |
|--------|-----------|-------|-----------|
| **Groth16** | Tiny (192 bytes) | Per-circuit | Possible |
| PLONK | Medium | Universal | Possible |
| Halo 2 | Larger | None! | Easy |

Penumbra chose Groth16 for:
- Best performance for fixed functionality
- Smallest proofs
- Battle-tested in production (Zcash)

## Key Takeaway

**Groth16 = A specific, well-tested recipe for making very small, very fast SNARK proofs**

The trade-off is you need to do a trusted setup ceremony first, but once that's done, you get the fastest and smallest proofs available!

Think of it as: **"The sports car of SNARKs - super fast and compact, but requires careful setup."**
