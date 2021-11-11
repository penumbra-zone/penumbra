use ark_ff::PrimeField;
use decaf377::FieldExt;
use once_cell::sync::Lazy;

use crate::{
    ka, prf,
    rdsa::{SpendAuth, VerificationKey},
    Fq, Fr,
};

use super::{DiversifierKey, IncomingViewingKey, NullifierKey, OutgoingViewingKey};

static IVK_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| Fq::from_le_bytes_mod_order(b"penumbra.derive.ivk"));

/// The `FullViewingKey` allows one to identify incoming and outgoing notes only.
pub struct FullViewingKey {
    ak: VerificationKey<SpendAuth>,
    nk: NullifierKey,
    ovk: OutgoingViewingKey,
    ivk: IncomingViewingKey,
}

impl FullViewingKey {
    /// Construct a full viewing key from its components.
    pub fn from_components(ak: VerificationKey<SpendAuth>, nk: NullifierKey) -> Self {
        let (ovk, dk) = {
            let hash_result = prf::expand(b"Penumbra_ExpndVK", &nk.0.to_bytes(), ak.as_ref());

            let mut ovk = [0; 32];
            let mut dk = [0; 32];
            ovk.copy_from_slice(&hash_result.as_bytes()[0..32]);
            dk.copy_from_slice(&hash_result.as_bytes()[32..64]);

            (ovk, dk)
        };

        let ivk = {
            let ak_s = Fq::from_bytes(*ak.as_ref())
                .expect("verification key is valid, so its byte encoding is a decaf377 s value");
            let ivk_mod_q = poseidon377::hash_2(&IVK_DOMAIN_SEP, (nk.0, ak_s));
            ka::Secret::new_from_field(Fr::from_le_bytes_mod_order(&ivk_mod_q.to_bytes()))
        };

        let dk = DiversifierKey(dk);
        let ovk = OutgoingViewingKey(ovk);
        let ivk = IncomingViewingKey { ivk, dk };

        Self { ak, nk, ovk, ivk }
    }

    /// Returns the incoming viewing key for this full viewing key.
    pub fn incoming(&self) -> &IncomingViewingKey {
        &self.ivk
    }

    /// Returns the outgoing viewing key for this full viewing key.
    pub fn outgoing(&self) -> &OutgoingViewingKey {
        &self.ovk
    }

    /// Returns the nullifier key contained in this full viewing key.
    pub fn nullifier_key(&self) -> &NullifierKey {
        &self.nk
    }

    /// Returns the spend verification key contained in this full viewing key.
    pub fn spend_verification_key(&self) -> &VerificationKey<SpendAuth> {
        &self.ak
    }
}
