//! This module serves as a small abstraction layer over the group operations we need.
//!
//! This simplifies logic in other parts of this crate, since we don't need to rely on
//! arkworks directly.
use ark_ec::pairing::{Pairing, PairingOutput};
use ark_ec::scalar_mul::{variable_base::VariableBaseMSM, ScalarMul};
use ark_serialize::CanonicalSerialize;
use decaf377::Bls12_377;
use rand_core::CryptoRngCore;

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

// # Batched Pairing Checks

/// Sample a random field that's "small" but still big enough for pairing checks.
fn rand_small_f<R: CryptoRngCore>(rng: &mut R) -> F {
    // 128 bits of security
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);
    F::from_le_bytes_mod_order(&bytes)
}

// This is just a little trick to avoid code duplication in the variants of this pairing checker
// for different base groups.
macro_rules! make_batched_pairing_checker {
    ($name:ident, $gl:ident, $gr:ident, $glp:ident, $grp:ident, $pairing:ident) => {
        /// A tool for efficiently making many pairing checks.
        ///
        /// This version is for pairing checks where the varying parts
        /// of each side of the pairing equality are in $gl and $gr, respectively.
        pub struct $name {
            // Invariant: both vecs have the same length.
            vary_l: Vec<$gl>,
            base_l: $glp,
            vary_r: Vec<$gr>,
            base_r: $grp,
        }

        impl $name {
            pub fn new(base_l: impl Into<$glp>, base_r: impl Into<$grp>) -> Self {
                Self {
                    vary_l: Vec::new(),
                    base_l: base_l.into(),
                    vary_r: Vec::new(),
                    base_r: base_r.into(),
                }
            }

            pub fn add(&mut self, l: $gl, r: $gr) {
                self.vary_l.push(l);
                self.vary_r.push(r);
            }

            #[must_use]
            pub fn check<R: CryptoRngCore>(self, rng: &mut R) -> bool {
                let n = self.vary_l.len();
                let scalars = (0..n).map(|_| rand_small_f(rng)).collect::<Vec<_>>();

                let ready_to_msm_l = <$gl as ScalarMul>::batch_convert_to_mul_base(&self.vary_l);
                let l = <$gl as VariableBaseMSM>::msm_unchecked(&ready_to_msm_l, &scalars);
                let ready_to_msm_r = <$gr as ScalarMul>::batch_convert_to_mul_base(&self.vary_r);
                let r = <$gr as VariableBaseMSM>::msm_unchecked(&ready_to_msm_r, &scalars);

                pairing(l, self.base_l) == $pairing(self.base_r, r)
            }
        }
    };
}

// This is just a gimmick to support our macro shenanigans
fn swapped_pairing(a: impl Into<G2Prepared>, b: impl Into<G1Prepared>) -> GT {
    pairing(b, a)
}

make_batched_pairing_checker!(
    BatchedPairingChecker11,
    G1,
    G1,
    G2Prepared,
    G2Prepared,
    swapped_pairing
);
make_batched_pairing_checker!(
    BatchedPairingChecker12,
    G1,
    G2,
    G2Prepared,
    G1Prepared,
    pairing
);

/// The size of the hash we use.
pub(crate) const HASH_SIZE: usize = 32;

/// The hash output we use when we need bytes.
pub(crate) type Hash = [u8; 32];

/// A utility struct for hashing group elements and producing fields.
///
/// This avoids having to deal with some serialization and reduction code from arkworks.
///
/// All methods of this struct will handle separation between elements correctly.
/// This means that feeding in two elements is distinct from feeding in the "concatenation"
/// of this elements. One place where you still need manual effort on the user's end
/// is when you're hashing a variable number of elements.
#[derive(Clone)]
pub(crate) struct GroupHasher {
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
            .to_state();
        Self { state }
    }

    // Separate methods because the semantics of what this is trying to do are different,
    // even if eating a usize happens to do the right thing.
    fn write_len(&mut self, len: usize) {
        self.eat_usize(len);
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

    /// Eat anything that's canonically serializable (yummy!).
    ///
    /// This will handle padding between elements, using the declared length.
    ///
    /// We keep this internal, to make a simpler public API, since we only have
    /// a handful of types we actually need to use this for.
    fn eat_canonical<T: CanonicalSerialize>(&mut self, x: &T) {
        self.write_len(x.compressed_size());
        x.serialize_compressed(&mut self.state)
            .expect("failed to serialize element");
    }

    /// Consume a usize value, adding it into the state of this hash.
    ///
    /// This is useful for (i.e. intended for) encoding metadata.
    pub fn eat_usize(&mut self, x: usize) {
        // On basically any platform this should fit in a u64
        self.state.update(&(x as u64).to_le_bytes());
    }

    /// Consume a G1 group element, adding it into the state of this hash.
    pub fn eat_g1(&mut self, x: &G1) {
        self.eat_canonical(x);
    }

    /// Consume a G2 group element, adding it into the state of this hash.
    pub fn eat_g2(&mut self, x: &G2) {
        self.eat_canonical(x);
    }

    /// Consume a scalar, adding it into the state of this hash.
    pub fn eat_f(&mut self, x: &F) {
        self.eat_canonical(x);
    }

    /// Finalize this hash function, producing a scalar.
    pub fn finalize(self) -> F {
        F::from_le_bytes_mod_order(self.state.finalize().as_bytes())
    }

    /// Finalize this hash function, producing bytes.
    pub fn finalize_bytes(self) -> Hash {
        let mut out = [0u8; HASH_SIZE];
        out.copy_from_slice(&self.state.finalize().as_bytes()[..HASH_SIZE]);
        out
    }
}
