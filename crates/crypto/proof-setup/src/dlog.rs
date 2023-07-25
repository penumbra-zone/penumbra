use ark_ff::UniformRand;
use rand_core::CryptoRngCore;

use crate::group::{GroupHasher, F, G1};

// Note: one choice you could make for these structs is to have them take
// references to their data, instead of copying them. However, operations like
// scalar multiplication take a move instead of a reference, at least in arkworks,
// so you don't avoid a move by doing that.

#[derive(Clone, Copy)]
pub struct Statement {
    result: G1,
    base: G1,
}

#[derive(Clone, Copy)]
pub struct Witness {
    dlog: F,
}

///A Proof of knowledge
#[derive(Clone, Copy)]
pub struct Proof {
    big_k: G1,
    s: F,
}

// This method is pulled out to be used in both proving and verifying.

/// Generate the challenge, given the context, statement, and nonce commitment.
fn challenge(ctx: &[u8], statement: &Statement, big_k: &G1) -> F {
    let mut hasher = GroupHasher::new(b"PAH:crmny_dlog");
    hasher.eat_bytes(ctx);
    hasher.eat_g1(&statement.result);
    hasher.eat_g1(&statement.base);
    hasher.eat_g1(big_k);
    hasher.finalize()
}

/// Create a proof that one knows a discrete logarithm relative to a given base element.
///
/// This requires the statement, describing the base element, and the result of scalar
/// multiplication, along with a witness, holding the scalar used for this multiplication.
///
/// We also take in a context string; the proof will only verify with a matching string.
/// This allows binding a proof to a given context.
pub fn prove<R: CryptoRngCore>(
    rng: &mut R,
    ctx: &[u8],
    statement: Statement,
    witness: Witness,
) -> Proof {
    let k = F::rand(rng);
    let big_k = statement.base * k;

    let e = challenge(ctx, &statement, &big_k);

    let s = k + e * witness.dlog;

    Proof { big_k, s }
}

/// Verify a proof that one knows a discrete logarithm relative to a given base element.
///
/// This requires the statement, describing the base element, and the result of scalar
/// multiplication, and the proof to verify, in lieu of a witness.
///
/// We also take in a context string; the proof will only verify with a string matching
/// the one used to create the proof.
#[must_use]
pub fn verify<'a>(ctx: &[u8], statement: Statement, proof: &Proof) -> bool {
    let e = challenge(ctx, &statement, &proof.big_k);
    statement.base * proof.s == proof.big_k + statement.result * e
}
