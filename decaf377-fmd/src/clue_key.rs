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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ClueKey(pub [u8; 32]);

/// An expanded and validated clue key that can be used to create [`Clue`]s
/// intended for the corresponding [`DetectionKey`](crate::DetectionKey).
#[allow(non_snake_case)]
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
    #[allow(non_snake_case)]
    pub fn expand(&self) -> Result<ExpandedClueKey, Error> {
        ExpandedClueKey::new(&self)
    }
}

impl ExpandedClueKey {
    pub fn new(clue_key: &ClueKey) -> Result<Self, Error> {
        let root_pub_enc = decaf377::Encoding(clue_key.0);
        let root_pub = root_pub_enc
            .decompress()
            .map_err(|_| Error::InvalidAddress)?;

        Ok(ExpandedClueKey {
            root_pub,
            root_pub_enc,
            subkeys: RefCell::new(Vec::new()),
        })
    }

    /// Checks that the expanded clue key has at least `precision` subkeys
    fn ensure_at_least(&self, precision: usize) -> Result<(), Error> {
        if precision > MAX_PRECISION {
            return Err(Error::PrecisionTooLarge(precision));
        }

        let current_precision = self.subkeys.try_borrow()?.len();

        // The cached expansion is large enough to accomodate the specified precision.
        if precision <= current_precision {
            return Ok(());
        }

        let mut expanded_keys = (current_precision..precision)
            .into_iter()
            .map(|i| hkd::derive_public(&self.root_pub, &self.root_pub_enc, i as u8))
            .collect::<Vec<_>>();

        self.subkeys.try_borrow_mut()?.append(&mut expanded_keys);

        return Ok(());
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

        // Ensure that at least `precision_bits` subkeys are available.
        self.ensure_at_least(precision_bits)?;

        let r = Fr::rand(&mut rng);
        let z = Fr::rand(&mut rng);

        let P = r * decaf377::basepoint();
        let P_encoding = P.compress();
        let Q = z * decaf377::basepoint();
        let Q_encoding = Q.compress();

        let mut ctxts = BitArray::<order::Lsb0, [u8; 3]>::zeroed();
        let Xs = self.subkeys.try_borrow()?;

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
}
