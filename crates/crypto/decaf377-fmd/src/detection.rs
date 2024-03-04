use std::convert::{TryFrom, TryInto};

use ark_ff::Zero;
use ark_serialize::CanonicalDeserialize;
use bitvec::{order, slice::BitSlice};
use decaf377::Fr;
use rand_core::{CryptoRng, RngCore};

use crate::{hash, hkd, Clue, ClueKey, Error, MAX_PRECISION};

// TODO serialization?

/// Used to examine [`Clue`]s and determine whether they were possibly sent to
/// the detection key's [`ClueKey`].
pub struct DetectionKey {
    /// The detection key.
    dtk: Fr,
    /// Cached copies of the child detection keys; these can be fully derived from `dtk`.
    xs: [Fr; MAX_PRECISION],
}

impl DetectionKey {
    /// Create a random detection key using the supplied `rng`.
    pub fn new<R: RngCore + CryptoRng>(mut rng: R) -> Self {
        Self::from_field(decaf377::Fr::rand(&mut rng))
    }

    /// Create a detection key using the supplied field element directly.
    ///
    /// # Warning
    ///
    /// This function exists to support custom key derivation mechanisms; it's
    /// the caller's responsibility to construct the detection key `dtk`
    /// correctly.
    pub fn from_field(dtk: Fr) -> Self {
        let root_pub = dtk * decaf377::Element::GENERATOR;
        let root_pub_enc = root_pub.vartime_compress();

        let xs: [_; MAX_PRECISION] = (0..MAX_PRECISION)
            .map(|i| {
                hkd::derive_private(
                    &dtk,
                    &root_pub_enc,
                    u8::try_from(i).expect("i < MAX_PRECISION < 256"),
                )
            })
            .collect::<Vec<_>>()
            // this conversion into an array will always succeed because we started with an iterator
            // of length `MAX_PRECISION` and we're converting to an array of the same length
            .try_into()
            .expect("iterator of length `MAX_PRECISION`");

        Self { dtk, xs }
    }

    /// Serialize this detection key to bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.dtk.to_bytes()
    }

    /// Deserialize a detection key from bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self, Error> {
        let dtk = Fr::from_bytes_checked(&bytes).map_err(|_| Error::InvalidDetectionKey)?;
        Ok(Self::from_field(dtk))
    }

    /// Obtain the clue key corresponding to this detection key.
    pub fn clue_key(&self) -> ClueKey {
        ClueKey(
            (self.dtk * decaf377::Element::GENERATOR)
                .vartime_compress()
                .0,
        )
    }

    /// Use this detection key to examine the given `clue`, returning `true` if the
    /// clue was possibly sent to this detection key's clue key.
    ///
    /// This test has false positives, but no false negatives.
    ///
    /// This function executes in constant time with respect to the detection
    /// key material, but short-circuits to return early on a false detection.
    #[allow(non_snake_case)]
    pub fn examine(&self, clue: &Clue) -> bool {
        let P_encoding = decaf377::Encoding::try_from(&clue.0[0..32]).expect("slice is right len");

        let P = if let Ok(P) = P_encoding.vartime_decompress() {
            P
        } else {
            // Invalid P encoding => not a match
            return false;
        };

        let y = if let Ok(y) = Fr::deserialize_compressed(&clue.0[32..64]) {
            y
        } else {
            // Invalid y encoding => not a match
            return false;
        };

        // Reject P = 0 or y = 0, as these never occur in well-formed clues; as
        // noted in the OpenPrivacy implementation, these could allow clues to
        // match any detection key.
        // https://docs.rs/fuzzytags/0.6.0/src/fuzzytags/lib.rs.html#348-351
        if P.is_identity() || y.is_zero() {
            return false;
        }

        let precision_bits = clue.0[64];
        let ciphertexts = BitSlice::<u8, order::Lsb0>::from_slice(&clue.0[65..68]);

        let m = hash::to_scalar(&P_encoding.0, precision_bits, &clue.0[65..68]);
        let Q_bytes = ((y * P) + (m * decaf377::Element::GENERATOR)).vartime_compress();

        for i in 0..(precision_bits as usize) {
            let Px_i = (P * self.xs[i]).vartime_compress();
            let key_i = hash::to_bit(&P_encoding.0, &Px_i.0, &Q_bytes.0);
            let msg_i = (ciphertexts[i] as u8) ^ key_i;
            // Short-circuit if we get a zero; this branch is dependent on the
            // ephemeral key bit `key_i`, not the long-term key `xs[i]`, so we
            // don't risk leaking any long-term secrets through timing channels.
            //
            // On the other hand, this gives a massive speedup, since we have a
            // 1/2 chance of rejecting after 1 iteration, 1/4 chance of
            // rejecting after 2 iterations, ..., so (in expectation) we do <= 2
            // iterations instead of n iterations.
            if msg_i == 0 {
                return false;
            }
        }

        // Otherwise, all message bits were 1 and we return true.
        true
    }
}
