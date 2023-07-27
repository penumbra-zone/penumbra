//! This module serves as a small abstraction layer over the group operations we need.
//!
//! This simplifies logic in other parts of this crate, since we don't need to rely on
//! arkworks directly.
use ark_ec::pairing::{Pairing, PairingOutput};
use ark_ff::fields::PrimeField;
use ark_serialize::CanonicalSerialize;
use blake2b_simd;
use decaf377::Bls12_377;

/// The group used for the left side of pairings.
pub type G1 = <Bls12_377 as Pairing>::G1;

/// A prepared version of G1 for more efficient pairings.
pub type G1Prepared = <Bls12_377 as Pairing>::G1Prepared;

/// The group used for the right side of pairings.
pub type G2 = <Bls12_377 as Pairing>::G2;

/// A prepared version of G2 for more efficient pairings.
pub type G2Prepared = <Bls12_377 as Pairing>::G2Prepared;

/// The group used for the output of pairings.
pub type GT = PairingOutput<Bls12_377>;

/// The field of scalars over which these groups form modules.
pub type F = <Bls12_377 as Pairing>::ScalarField;

/// The pairing operation between the two groups.
pub fn pairing(a: impl Into<G1Prepared>, b: impl Into<G2Prepared>) -> GT {
    <Bls12_377 as Pairing>::pairing(a, b)
}

/// Desired security in bits.
const SECURITY_PARAMETER: usize = 128;

/// The number of bytes needed for a hash output for safe reduction to a scalar.
///
/// The rule of thumb here is that you add 128 bits to size of the modulus,
/// giving you at least 128 bits of security, regardless of what the field looks like.
const SAFE_F_HASH_SIZE: usize =
    ((<F as PrimeField>::MODULUS_BIT_SIZE as usize + SECURITY_PARAMETER) + 8 - 1) / 8;

/// A utility struct for hashing group elements and producing fields.
///
/// This avoids having to deal with some serialization and reduction code from arkworks.
#[derive(Clone)]
pub struct GroupHasher {
    state: blake2b_simd::State,
}

impl GroupHasher {
    /// Create a new hasher with a personalization string.
    ///
    /// Because of BLAKE2's limitations, this has to be 16 bytes at most.
    /// This function will panic if that isn't the case.
    pub fn new(personalization: &'static [u8]) -> Self {
        let state = blake2b_simd::Params::new()
            .personal(personalization)
            .hash_length(SAFE_F_HASH_SIZE)
            .to_state();
        Self { state }
    }

    fn write_len(&mut self, len: usize) {
        // On basically any platform this should fit in a u64
        self.state.update(&(len as u64).to_le_bytes());
    }

    /// Consume some bytes, adding it to the state of the hash.
    ///
    /// These bytes will be length prefixed, and so calling this function
    /// multiple times is not the same as calling it with the concatenation
    /// of those bytes.
    pub fn eat_bytes(&mut self, x: &[u8]) {
        self.write_len(x.len());
        self.state.update(x);
    }

    /// Consume a G1 group element, adding it into the state of this hash.
    ///
    /// This will automatically handle padding between elements, so hashing
    /// a constant number of group elements one after the other is safe,
    /// even if they happened to not have a constant-size serialization; they should though.
    pub fn eat_g1(&mut self, x: &G1) {
        self.write_len(x.compressed_size());
        x.serialize_compressed(&mut self.state)
            .expect("failed to serialize G1 element");
    }

    /// Finalize this hash function, producing a scalar.
    pub fn finalize(self) -> F {
        let bytes = self.state.finalize();
        F::from_le_bytes_mod_order(bytes.as_bytes())
    }
}
