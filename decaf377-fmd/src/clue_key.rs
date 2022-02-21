use std::cell::RefCell;

use ark_ff::{Field, UniformRand};
use bitvec::{array::BitArray, order};
use decaf377::{FieldExt, Fr};
use rand_core::{CryptoRng, RngCore};

use crate::{hash, hkd, Clue, Error, MAX_PRECISION};

/// Bytes representing a clue key corresponding to some
/// [`DetectionKey`](crate::DetectionKey).
///
/// This type is a refinement type for plain bytes, and is suitable for use in
/// situations where clue key might or might not actually be used.  This saves
/// computation; at the point that a clue key will be used to create a [`Clue`],
/// it can be expanded to an [`ExpandedClueKey`].
/// TODO: use Zeroize?
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ClueKey(pub [u8; 32]);

/// An expanded and validated clue key that can be used to create [`Clue`]s
/// intended for the corresponding [`DetectionKey`](crate::DetectionKey).
#[allow(non_snake_case)]
pub struct ExpandedClueKey {
    subkeys: RefCell<Vec<decaf377::Element>>,
}

impl ClueKey {
    /// Validate and expand this clue key encoding.
    ///
    /// # Errors
    ///
    /// Fails if the bytes don't encode a valid clue key.
    #[allow(non_snake_case)]
    pub fn expand(&self, precision: usize) -> Result<ExpandedClueKey, Error> {
        if precision > MAX_PRECISION {
            return Err(Error::PrecisionTooLarge(precision));
        }

        let root_pub_enc = decaf377::Encoding(self.0);
        let root_pub = root_pub_enc
            .decompress()
            .map_err(|_| Error::InvalidAddress)?;

        // TOOD: generate subkeys between 0 and `precision`, should we instead handle an arbitrary
        //       range (e.g. K_i <> K_{i+k})
        let Xs = (0..precision)
            .into_iter()
            .map(|i| hkd::derive_public(&root_pub, &root_pub_enc, i as u8))
            .collect::<Vec<_>>();

        Ok(ExpandedClueKey {
            subkeys: RefCell::new(Xs),
        })
    }
}

impl ExpandedClueKey {
    /// Create a [`Clue`] intended for the [`DetectionKey`](crate::DetectionKey)
    /// corresponding to this clue key.
    ///
    /// The clue will be detected by the intended detection key with probability
    /// 1, and by other detection keys with probability `2^{-precision_bits}`.
    ///
    /// # Errors
    ///
    /// `precision_bits` must be smaller than [`MAX_PRECISION`].
    /// `precision_bits` must be smaller or equal to the `ExpandedClueKey` precision.
    #[allow(non_snake_case)]
    pub fn create_clue<R: RngCore + CryptoRng>(
        &self,
        precision_bits: usize,
        mut rng: R,
    ) -> Result<Clue, Error> {
        if precision_bits >= MAX_PRECISION {
            return Err(Error::PrecisionTooLarge(precision_bits));
        }

        if !self.ensure_at_least(precision_bits) {
            return Err(Error::PrecisionTooLarge(precision_bits));
        }

        let r = Fr::rand(&mut rng);
        let z = Fr::rand(&mut rng);

        let P = r * decaf377::basepoint();
        let P_encoding = P.compress();
        let Q = z * decaf377::basepoint();
        let Q_encoding = Q.compress();

        let mut ctxts = BitArray::<order::Lsb0, [u8; 3]>::zeroed();
        let Xs = self.subkeys.try_borrow().unwrap();

        for i in 0..precision_bits {
            let rXi = (r * Xs[i]).compress();
            let key_i = hash::to_bit(&P_encoding.0, &rXi.0, &Q_encoding.0);
            let ctxt_i = key_i ^ 1u8;
            ctxts.set(i, ctxt_i != 0);
        }

        let m = hash::to_scalar(&P_encoding.0, precision_bits as u8, ctxts.as_buffer());
        let y = (z - m) * r.inverse().expect("random element is nonzero");

        let mut buf = [0u8; 68];
        buf[0..32].copy_from_slice(&P_encoding.0[..]);
        buf[32..64].copy_from_slice(&y.to_bytes()[..]);
        buf[64] = precision_bits as u8;
        buf[65..68].copy_from_slice(ctxts.as_buffer());

        Ok(Clue(buf))
    }

    /// Checks that the expanded clue key has at least `precision` subkeys
    fn ensure_at_least(&self, precision: usize) -> bool {
        // TODO: handle try_borrow failure via Result
        return self.subkeys.try_borrow().unwrap().len() >= precision;
    }
}
