# Commitments Explained (ELI5)

## The Problem: How to Hide But Not Lie?

Imagine you want to prove you have at least $5 to buy coffee, but you don't want to reveal you actually have $100 in your wallet. How do you do this?

**The naive approach doesn't work:**
- If you say "I have money" → Too vague, could be lying
- If you say "I have $100" → Too revealing, everyone knows your balance
- If you encrypt "I have $100" → Can't prove anything about the encrypted value

**We need something special: A commitment!**

---

## What is a Commitment? (The Locked Box Analogy)

A **commitment** is like putting your balance in a locked box:

### Step 1: You put your secret in the box
```
Your secret: "I have $100"
↓
[Put in locked box]
↓
Box sealed with your unique lock
```

### Step 2: You show everyone the locked box
```
Everyone sees: [🔒 Locked Box #847293]
```

The box has a serial number (this is the commitment value), but **nobody can see inside**.

### Step 3: Later, you can open the box to prove what was inside
```
You provide the key
↓
Everyone opens the box
↓
They see: "I have $100"
↓
They verify: "Yes, Box #847293 contained '$100'"
```

### The Magic Properties:

1. **Hiding**: Nobody can tell what's in the box just by looking at it
   - Box #847293 could contain "$100" or "$1 million" - looks the same!

2. **Binding**: You can't change what's in the box after you lock it
   - Can't swap "$100" for "$1" after you've shown the box
   - The serial number (commitment) is tied to the contents

3. **Verifiable**: Anyone can verify the box contains what you claim
   - When you open it, the serial number proves it's the right box

---

## The Math (Pedersen Commitments)

Now let's see how this works mathematically in Penumbra.

### The Setup (Public Parameters)

Everyone agrees on two special numbers (elliptic curve points):
- **G** = A fixed generator point (like 5)
- **H** = Another generator point (like 7)

These are **public** - everyone knows them!

### Creating a Commitment

To commit to a value `v` (like 100 tokens):

```
Commitment = v*G + r*H
```

Where:
- `v` = Your secret value (100 tokens)
- `r` = A random "blinding factor" (keeps it secret)
- `*` = Elliptic curve multiplication

### Concrete Example

Let's commit to **100 tokens**:

```rust
// Your secret value
let value = 100;

// Pick a random blinding factor (this is secret too!)
let blinding = 87263591827365;  // Some random big number

// Public generators (everyone knows these)
let G = decaf377::Element::GENERATOR;
let H = decaf377::Element::ALTERNATE_GENERATOR;

// Create commitment
let commitment = value * G + blinding * H;
```

**What everyone sees**: `commitment = decaf377::Element(0x1a2b3c4d...)`
- This is just a point on an elliptic curve
- Looks completely random!
- Reveals **nothing** about value or blinding

**What you know**:
- `value = 100`
- `blinding = 87263591827365`
- These are your secrets!

---

## Why Can't Someone Reverse It?

**Question**: If everyone knows G and H, and they see `commitment = 100*G + r*H`, can't they figure out that value = 100?

**Answer**: No! Here's why:

There are infinitely many solutions:
- `commitment = 100*G + 87263591827365*H`
- `commitment = 50*G + 42387264923847*H`  (different value, different blinding!)
- `commitment = 200*G + 12093847562934*H` (different value, different blinding!)
- ... infinite possibilities!

Without knowing `r` (the blinding factor), you **cannot** figure out `v`.

It's like this equation:
```
x + y = 10
```
What are x and y? Could be:
- x=5, y=5
- x=3, y=7
- x=100, y=-90
- Infinite solutions!

Same with commitments - infinite possible (value, blinding) pairs that give the same commitment.

---

## Property #1: Hiding

Let me show you why commitments hide the value:

```rust
// Commitment to 100 tokens
let commit_100 = 100 * G + random1 * H;
// Output: Element(0x7a3f2e...)

// Commitment to 1,000,000 tokens
let commit_million = 1_000_000 * G + random2 * H;
// Output: Element(0x9b4d1c...)
```

**Do these look related?** NO! They both look like random elliptic curve points.

You **cannot tell** which one is bigger, smaller, or even if they're different amounts. They're perfectly hidden!

---

## Property #2: Binding

Once you create a commitment, you **cannot** change what you committed to.

**Why not?** Let's say you try to cheat:

```rust
// You create a commitment to 100
let commit = 100 * G + r * H;

// You show everyone: "My commitment is Element(0x7a3f2e...)"

// Later, you try to lie and say it was actually 200
// You'd need to find an r2 where:
// 200 * G + r2 * H = 100 * G + r * H
// This means: 100 * G = (r - r2) * H

// But G and H are chosen so this is IMPOSSIBLE!
// (technically: discrete log problem - computationally infeasible)
```

You're **bound** to your original value. Can't change it!

---

## Property #3: Homomorphic (The Magic Property!)

This is where it gets really cool! **Commitments can be added together!**

```rust
// Alice commits to 50 tokens
let alice_commit = 50 * G + r1 * H;

// Bob commits to 30 tokens
let bob_commit = 30 * G + r2 * H;

// Add the commitments
let total_commit = alice_commit + bob_commit;
               = (50 * G + r1 * H) + (30 * G + r2 * H)
               = 50*G + 30*G + r1*H + r2*H
               = 80*G + (r1+r2)*H

// This is a commitment to 80!
```

**This means**: You can add commitments without revealing the values!

### Why This Matters for Transactions

In a transaction:
```
Inputs:  Commit to +100 tokens (what you're spending)
Outputs: Commit to -60 tokens (payment) + Commit to -40 tokens (change)

Sum: 100*G + r1*H - 60*G - r2*H - 40*G - r3*H
   = (100 - 60 - 40)*G + (r1 - r2 - r3)*H
   = 0*G + blinding_sum*H
```

If the commitments sum to a commitment to **zero** (plus some blinding), the transaction balances!

**You just proved the transaction is valid without revealing any amounts!** 🎉

---

## Real Penumbra Code

Here's how commitments actually work in Penumbra:

### Creating a Balance Commitment

From `crates/core/asset/src/balance/commitment.rs`:

```rust
pub struct Commitment(pub decaf377::Element);

impl Balance {
    pub fn commit(&self, blinding: Fq) -> Commitment {
        let mut commitment = decaf377::Element::default();

        // For each (amount, asset_id) in the balance:
        for (asset_id, amount) in self.iter() {
            // Get the generator for this asset
            let generator = asset_id.value_generator();

            // Add amount * generator to commitment
            let amount_scalar = Fq::from(amount);
            commitment += generator * amount_scalar;
        }

        // Add blinding term
        let blinding_generator = decaf377::Element::ALTERNATE_GENERATOR;
        commitment += blinding_generator * blinding;

        Commitment(commitment)
    }
}
```

### In a Transaction

From the Spend proof (`crates/core/component/shielded-pool/src/spend/proof.rs`):

```rust
pub struct SpendProofPublic {
    pub balance_commitment: balance::Commitment,  // This is v*G + r*H!
    // ... other fields
}
```

Everyone sees `balance_commitment` but has NO IDEA what the actual balance is!

---

## Concrete Transaction Example

Let's walk through a real example with actual numbers:

### Setup
```
Alice has: 100 PEN (Penumbra tokens)
Alice wants to send: 60 PEN to Bob
Alice's change: 40 PEN
```

### Step 1: Alice creates input commitment

```rust
let input_value = 100;
let input_blinding = random(); // Say this is 12345

let input_commitment = input_value * G + input_blinding * H;
// = 100*G + 12345*H
```

### Step 2: Alice creates output commitments

```rust
// Output 1: Payment to Bob (60 PEN)
let output1_value = 60;
let output1_blinding = random(); // Say this is 67890
let output1_commitment = output1_value * G + output1_blinding * H;
// = 60*G + 67890*H

// Output 2: Change back to Alice (40 PEN)
let output2_value = 40;
let output2_blinding = random(); // Say this is 11111
let output2_commitment = output2_value * G + output2_blinding * H;
// = 40*G + 11111*H
```

### Step 3: Verify balance (what validators do)

```rust
// In Penumbra, outputs are NEGATIVE (being consumed)
// So the check is: input + (-output1) + (-output2) = 0

let balance_check = input_commitment - output1_commitment - output2_commitment;
                  = (100*G + 12345*H) - (60*G + 67890*H) - (40*G + 11111*H)
                  = 100*G - 60*G - 40*G + 12345*H - 67890*H - 11111*H
                  = 0*G + (12345 - 67890 - 11111)*H
                  = 0*G + (-66656)*H
```

**Wait, that's not zero!** That's correct! The blinding factors don't need to sum to zero - they're random. What matters is the **value** part sums to zero.

Actually, in Penumbra, Alice includes a "balance commitment" that proves this:

```rust
// Alice proves in zero-knowledge that:
// input_value - output1_value - output2_value = 0
// (100 - 60 - 40 = 0) ✓

// The ZK proof ensures the values balance without revealing them!
```

---

## The "Commitment-Opening" Process

When you want to prove what's in the commitment:

### Creating a commitment:
```rust
let value = 100;
let blinding = 12345;
let commitment = value * G + blinding * H;

// Show everyone: commitment
```

### Opening the commitment:
```rust
// Reveal the secret values:
let revealed_value = 100;
let revealed_blinding = 12345;

// Everyone verifies:
let recomputed = revealed_value * G + revealed_blinding * H;
assert_eq!(recomputed, commitment); // ✓ Matches!
```

If you try to lie:
```rust
// You claim it was 200:
let fake_value = 200;
let fake_blinding = 12345; // same blinding

let recomputed = fake_value * G + fake_blinding * H;
assert_eq!(recomputed, commitment); // ✗ DOESN'T MATCH!
```

**You can't fake it!** The commitment binds you to the original value.

---

## Why Don't We Just Open Commitments?

**Question**: If you can open commitments to prove the value, why not just do that and skip the ZK proofs?

**Answer**: Because opening reveals your balance!

In Penumbra:
- You **never** open your commitments
- Instead, you use **zero-knowledge proofs** to prove properties about committed values
- For example: "These commitments sum to zero" (without revealing what they commit to!)

The ZK proof says:
> "I know values v1, v2, v3 and blindings r1, r2, r3 such that:
> - commitment1 = v1*G + r1*H
> - commitment2 = v2*G + r2*H
> - commitment3 = v3*G + r3*H
> - v1 - v2 - v3 = 0"

Everyone can verify the proof, but they never learn v1, v2, or v3!

---

## Visual Summary

```
┌─────────────────────────────────────────────────────────────┐
│ COMMITMENT SCHEME                                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Secret Value: v = 100 tokens                               │
│  Secret Blinding: r = 87263591827365                        │
│                                                             │
│            ↓ (v * G + r * H)                                │
│                                                             │
│  Commitment: Element(0x1a2b3c4d5e6f...)                     │
│             [Looks completely random!]                       │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Properties:                                                │
│                                                             │
│  ✓ HIDING:     Cannot determine v from commitment           │
│  ✓ BINDING:    Cannot change v after creating commitment    │
│  ✓ HOMOMORPHIC: Can add commitments → commit to sum         │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  In Transactions:                                           │
│                                                             │
│  Input:  Commit(100 PEN)                                    │
│  Output: Commit(60 PEN)  ← to Bob                           │
│  Output: Commit(40 PEN)  ← change                           │
│                                                             │
│  Verify: Commit(100) - Commit(60) - Commit(40) = Commit(0)  │
│          ↑ All values hidden!                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## The Key Insight

**Commitments are the bridge between privacy and verifiability!**

Without commitments:
- ❌ Public amounts → no privacy
- ❌ Encrypted amounts → can't verify balance

With commitments:
- ✅ Hidden amounts (privacy!)
- ✅ Can verify balance (security!)
- ✅ Can use ZK proofs to prove properties

**This is the magic that makes private blockchains like Penumbra possible!**

---

## Common Misconceptions

### ❌ "The commitment is an encrypted value"
**No!** Encryption means you can decrypt it. A commitment **cannot** be "decrypted" - you can only verify a claimed value against it.

### ❌ "You can brute-force the value"
**No!** The blinding factor has 256 bits of randomness. Even if the value is small (like 1-100), the blinding makes it impossible to brute-force.

### ❌ "Commitments prove the value is correct"
**No!** Commitments only **hide** the value. You need a **zero-knowledge proof** to prove properties about the committed value.

---

## Try It Yourself!

Here's simplified pseudocode you can understand:

```rust
// Simple commitment scheme (not cryptographically secure, just for learning!)

fn commit(value: u64, blinding: u64) -> u64 {
    // In reality, this uses elliptic curves
    // But for intuition, imagine:
    let G = 5;  // public
    let H = 7;  // public

    value * G + blinding * H
}

fn verify(commitment: u64, claimed_value: u64, claimed_blinding: u64) -> bool {
    commit(claimed_value, claimed_blinding) == commitment
}

// Example:
let value = 100;
let blinding = 42;
let commit = commit(100, 42);  // = 100*5 + 42*7 = 500 + 294 = 794

println!("Commitment: {}", commit);  // 794

// Later, to verify:
let is_valid = verify(794, 100, 42);  // true
let is_fake = verify(794, 200, 42);   // false - trying to cheat!
```

Of course, real commitments use elliptic curves which make this secure!

---

## Key Takeaway

**A commitment is a cryptographic way to:**
1. **Hide** a value (no one can see what's committed)
2. **Bind** you to that value (you can't change it later)
3. **Verify** the value (when you reveal it, anyone can check)
4. **Compute on hidden values** (add commitments → commitment to sum!)

In Penumbra, commitments let you prove transactions are valid (inputs = outputs) **without revealing any amounts!**

This is the foundation of Penumbra's privacy! 🔒✨
