use std::{cell::RefCell, convert::TryFrom};

use bitvec::{array::BitArray, order};
use decaf377::{Fq, Fr};
use rand_core::{CryptoRng, RngCore};

use crate::{hash, hkd, Clue, Error, Precision};

/// Bytes representing a clue key corresponding to some
/// [`DetectionKey`](crate::DetectionKey).
///
/// This type is a refinement type for plain bytes, and is suitable for use in
/// situations where clue key might or might not actually be used.  This saves
/// computation; at the point that a clue key will be used to create a [`Clue`],
/// it can be expanded to an [`ExpandedClueKey`].
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ClueKey(pub [u8; 32]);

/// An expanded and validated clue key that can be used to create [`Clue`]s
/// intended for the corresponding [`DetectionKey`](crate::DetectionKey).
pub struct ExpandedClueKey {
    root_pub: decaf377::Element,
    root_pub_enc: decaf377::Encoding,
    subkeys: RefCell<Vec<decaf377::Element>>,
}

impl ClueKey {
    /// Validate and expand this clue key encoding.
    ///
    /// # Errors
    ///
    /// Fails if the bytes don't encode a valid clue key.
    pub fn expand(&self) -> Result<ExpandedClueKey, Error> {
        ExpandedClueKey::new(self)
    }

    /// Expand this clue key encoding.
    ///
    /// This method always results in a valid clue key, though the clue key may not have
    /// a known detection key.
    pub fn expand_infallible(&self) -> ExpandedClueKey {
        let mut counter = 0u32;
        loop {
            counter += 1;
            let ck_fq_incremented = Fq::from_le_bytes_mod_order(&self.0) + Fq::from(counter);
            let ck = ClueKey(ck_fq_incremented.to_bytes());
            if let Ok(eck) = ck.expand() {
                return eck;
            }
        }
    }
}

impl ExpandedClueKey {
    pub fn new(clue_key: &ClueKey) -> Result<Self, Error> {
        let root_pub_enc = decaf377::Encoding(clue_key.0);
        let root_pub = root_pub_enc
            .vartime_decompress()
            .map_err(|_| Error::InvalidAddress)?;

        Ok(ExpandedClueKey {
            root_pub,
            root_pub_enc,
            subkeys: RefCell::new(Vec::new()),
        })
    }

    /// Checks that the expanded clue key has at least `precision` subkeys
    fn ensure_at_least(&self, precision: usize) -> Result<(), Error> {
        let current_precision = self.subkeys.borrow().len();

        // The cached expansion is large enough to accommodate the specified precision.
        if precision <= current_precision {
            return Ok(());
        }

        let mut expanded_keys = (current_precision..precision)
            .map(|i| hkd::derive_public(&self.root_pub, &self.root_pub_enc, i as u8))
            .collect::<Vec<_>>();

        self.subkeys.borrow_mut().append(&mut expanded_keys);

        Ok(())
    }

    /// Create a [`Clue`] intended for the [`DetectionKey`](crate::DetectionKey)
    /// corresponding to this clue key, deterministically, using the provided
    /// random seed.
    ///
    /// The clue will be detected by the intended detection key with probability
    /// 1, and by other detection keys with probability `2^{-precision_bits}`.
    ///
    /// # Errors
    ///
    /// `precision_bits` must be smaller than [`MAX_PRECISION`].
    #[allow(non_snake_case)]
    pub fn create_clue_deterministic(
        &self,
        precision: Precision,
        rseed: [u8; 32],
    ) -> Result<Clue, Error> {
        let precision_bits = precision.bits() as usize;
        // Ensure that at least `precision_bits` subkeys are available.
        self.ensure_at_least(precision_bits)?;

        let r = {
            let hash = blake2b_simd::Params::default()
                .personal(b"decaf377-fmd.rdv")
                .to_state()
                .update(&self.root_pub_enc.0)
                .update(&rseed)
                .finalize();

            Fr::from_le_bytes_mod_order(hash.as_bytes())
        };
        let z = {
            let hash = blake2b_simd::Params::default()
                .personal(b"decaf377-fmd.zdv")
                .to_state()
                .update(&self.root_pub_enc.0)
                .update(&rseed)
                .finalize();

            Fr::from_le_bytes_mod_order(hash.as_bytes())
        };

        let P = r * decaf377::Element::GENERATOR;
        let P_encoding = P.vartime_compress();
        let Q = z * decaf377::Element::GENERATOR;
        let Q_encoding = Q.vartime_compress();

        let mut ctxts = BitArray::<[u8; 3], order::Lsb0>::ZERO;
        let Xs = self.subkeys.borrow();

        for i in 0..precision_bits {
            let rXi = (r * Xs[i]).vartime_compress();
            let key_i = hash::to_bit(&P_encoding.0, &rXi.0, &Q_encoding.0);
            let ctxt_i = key_i ^ 1u8;
            ctxts.set(i, ctxt_i != 0);
        }

        let m = hash::to_scalar(&P_encoding.0, precision_bits as u8, ctxts.as_raw_slice());
        let y = (z - m) * r.inverse().expect("random element is nonzero");

        let mut buf = [0u8; 68];
        buf[0..32].copy_from_slice(&P_encoding.0[..]);
        buf[32..64].copy_from_slice(&y.to_bytes()[..]);
        buf[64] = precision_bits as u8;
        buf[65..68].copy_from_slice(ctxts.as_raw_slice());

        Ok(Clue(buf))
    }

    /// Create a [`Clue`] intended for the [`DetectionKey`](crate::DetectionKey)
    /// corresponding to this clue key.
    ///
    /// The clue will be detected by the intended detection key with probability
    /// 1, and by other detection keys with probability `2^{-precision_bits}`.
    ///
    /// # Errors
    ///
    /// `precision_bits` must be smaller than [`MAX_PRECISION`].
    #[allow(non_snake_case)]
    pub fn create_clue<R: RngCore + CryptoRng>(
        &self,
        precision: Precision,
        mut rng: R,
    ) -> Result<Clue, Error> {
        let mut rseed = [0u8; 32];
        rng.fill_bytes(&mut rseed);
        self.create_clue_deterministic(precision, rseed)
    }
}

impl TryFrom<&[u8]> for ClueKey {
    type Error = Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes[0..32]);
            Ok(ClueKey(arr))
        } else {
            Err(Error::InvalidClueKey)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clue_key_infallible_expand() {
        let valid_ck = ClueKey(decaf377::Element::GENERATOR.vartime_compress().0);
        let ck_fq_invalid = Fq::from_le_bytes_mod_order(&valid_ck.0) + Fq::from(1u64);
        let invalid_ck = ClueKey(ck_fq_invalid.to_bytes());
        let _eck = invalid_ck.expand_infallible();
    }
}
