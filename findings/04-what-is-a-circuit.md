# What is a Circuit? (ELI5)

## The Simple Answer

A **circuit** is a set of mathematical rules (constraints) that define what you're trying to prove. Think of it like a recipe or a checklist that the proof system follows.

**In regular programming**: You write functions that compute things
**In zero-knowledge programming**: You write circuits that prove things

## The Cooking Analogy

Imagine you're proving you baked a cake correctly:

**A regular program** would be:
```
1. Mix ingredients
2. Pour into pan
3. Bake at 350°F
4. Return the finished cake
```

**A circuit** would be:
```
1. Check: Did you use flour? (constraint 1)
2. Check: Did you use eggs? (constraint 2)
3. Check: Was oven at 350°F? (constraint 3)
4. Check: Did you bake for 30 minutes? (constraint 4)
5. If all checks pass → proof is valid
```

The circuit doesn't actually bake the cake—it **proves** you followed the recipe correctly!

## Real Example: A Simple "Greater Than" Circuit

Let's build a circuit that proves: **"I know a number that's greater than 10"** without revealing the number.

### Step-by-Step:

**What you know (private):**
- `secret_number = 15`

**What you want to prove (public):**
- `secret_number > 10`

**The circuit (constraints):**
```
Constraint 1: secret_number - 10 = difference
Constraint 2: difference * inverse = 1  (proves difference ≠ 0)
Constraint 3: Check difference is positive (not negative)
```

If all three constraints are satisfied, the proof is valid! But nobody learns that your secret number is 15—they only learn "yes, it's greater than 10."

## How Penumbra Uses Circuits

From the codebase, Penumbra has 7 main circuits. Let's look at the **Spend Circuit** as an example:

### The Spend Circuit (Simplified)

**Location in code**: `crates/core/component/shielded-pool/src/spend/proof.rs`

**What it proves**: "I can legitimately spend this note (token)"

**Public inputs** (everyone sees):
```rust
pub struct SpendProofPublic {
    pub anchor: tct::Root,                    // Merkle tree root
    pub balance_commitment: Commitment,        // Hidden balance
    pub nullifier: Nullifier,                 // Prevents double-spending
    pub rk: VerificationKey,                  // Randomized verification key
}
```

**Private inputs** (only you know):
```rust
pub struct SpendProofPrivate {
    pub note: Note,                           // The actual note you're spending
    pub state_commitment_proof: tct::Proof,   // Proof the note exists in tree
    pub v_blinding: Fr,                       // Random blinding factor
    pub spend_auth_randomizer: Fr,           // Key randomizer
    pub ak: VerificationKey,                  // Your authorization key
    pub nk: NullifierKey,                    // Your nullifier key
}
```

**What the circuit checks** (the constraints):

1. ✅ **Note exists**: Check the note is in the merkle tree (using state_commitment_proof)
2. ✅ **Correct nullifier**: Check nullifier = Hash(nk, position, note_commitment)
3. ✅ **You own it**: Check you have the right keys (ak, nk match the note)
4. ✅ **Balance is correct**: Check balance_commitment matches the note value
5. ✅ **Key is valid**: Check rk = ak + (spend_auth_randomizer * generator)

If all 5 checks pass → you proved you can spend the note!
If any check fails → proof generation fails

## Mock Circuit Example: Password Checker

Let's build a simple circuit that proves "I know the password" without revealing it!

### Setup:
- **Password hash (public)**: `hash = 12345` (everyone knows this)
- **Password (private)**: `password = "secret"` (only you know this)

### The Circuit:

```rust
// Mock pseudocode (simplified)
struct PasswordCircuit {
    // Public inputs
    pub expected_hash: u64,

    // Private inputs (witness)
    password: String,
}

impl PasswordCircuit {
    fn generate_constraints() {
        // Constraint 1: Compute hash of password
        let computed_hash = Hash(self.password);

        // Constraint 2: Check computed hash equals expected hash
        assert_equal(computed_hash, self.expected_hash);

        // If both constraints satisfied -> proof valid!
    }
}
```

**To use it:**
```rust
// Create the circuit with your password
let circuit = PasswordCircuit {
    expected_hash: 12345,  // Public
    password: "secret",     // Private
};

// Generate proof (uses Groth16)
let proof = Groth16::prove(proving_key, circuit);

// Anyone can verify without knowing password!
let valid = Groth16::verify(verifying_key, proof, expected_hash);
// valid = true, but they never learned password = "secret"
```

## How Circuits Are Built (R1CS)

Under the hood, circuits use something called **R1CS** (Rank-1 Constraint System).

Every constraint has the form:
```
(A) * (B) = (C)
```

For example, to check `x * y = z`:
```
(x) * (y) = (z)   ← This is one R1CS constraint
```

To check `x + y = z`, you rewrite it:
```
(x + y) * (1) = (z)   ← Now it's in R1CS form!
```

Complex circuits are just lots of these constraints chained together!

### Penumbra's Circuit Sizes

From the docs at `/tmp/penumbra-guide/pages/dev/parameter_setup.md`:

| Circuit | Number of Constraints | Proof Time |
|---------|----------------------|------------|
| Spend | 35,978 constraints | 433ms |
| Output | 13,875 constraints | 142ms |
| Swap | 25,704 constraints | 272ms |
| SwapClaim | 46,656 constraints | 456ms |
| Delegator Vote | 38,071 constraints | 443ms |
| Convert | 14,423 constraints | 179ms |
| Nullifier Derivation | 394 constraints | 17ms |

More constraints = more complex proof = longer proving time

## Circuit Programming in Rust

Here's how you actually write a circuit in Penumbra's codebase:

```rust
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::ConstraintSynthesizer;

pub struct MyCircuit {
    pub public_input: u64,
    private_witness: u64,
}

impl ConstraintSynthesizer<Fq> for MyCircuit {
    fn generate_constraints(
        self,
        cs: ConstraintSystemRef<Fq>,
    ) -> Result<(), SynthesisError> {
        // Allocate public input
        let public_var = FqVar::new_input(cs.clone(), || Ok(self.public_input))?;

        // Allocate private witness
        let private_var = FqVar::new_witness(cs.clone(), || Ok(self.private_witness))?;

        // Add constraint: private * 2 = public
        let two = FqVar::new_constant(cs.clone(), Fq::from(2u64))?;
        let result = &private_var * &two;
        result.enforce_equal(&public_var)?;

        Ok(())
    }
}
```

This circuit proves: "I know a number that when doubled equals the public input"

## The Flow: From Circuit to Proof

```
1. Write Circuit
   ↓
2. Generate Proving/Verifying Keys (trusted setup)
   ↓
3. Fill in private values (your secret data)
   ↓
4. Generate Constraints (check all the rules)
   ↓
5. Create Proof (using Groth16)
   ↓
6. Anyone can verify the proof!
```

## Why Are Circuits Important?

Circuits are the **bridge between regular computation and zero-knowledge proofs**.

- You can't just prove "any random thing" with ZK
- You need to write a circuit that defines **exactly** what you're proving
- The circuit is like a contract: "If these constraints are satisfied, the statement is true"

## Key Takeaway

**A circuit is a set of mathematical constraints that define what you're trying to prove.**

Think of it as:
- **A checklist** that must all be satisfied
- **A recipe** that must be followed exactly
- **A contract** that enforces the rules of your proof

In Penumbra, circuits prove things like "I can spend this note" or "This swap is valid" without revealing the secret data!
