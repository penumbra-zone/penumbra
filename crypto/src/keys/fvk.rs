use anyhow::Context;
use ark_ff::PrimeField;
use ark_serialize::CanonicalDeserialize;
use decaf377::FieldExt;
use once_cell::sync::Lazy;
use penumbra_proto::{crypto as pb, serializers::bech32str, Protobuf};
use serde::{Deserialize, Serialize};

use super::{DiversifierKey, IncomingViewingKey, NullifierKey, OutgoingViewingKey};
use crate::{
    ka, note, prf,
    rdsa::{SpendAuth, VerificationKey},
    Fq, Fr, Note, Nullifier,
};

static IVK_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| Fq::from_le_bytes_mod_order(b"penumbra.derive.ivk"));

/// The root viewing capability for all data related to a given spend authority.
#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "pb::FullViewingKey", into = "pb::FullViewingKey")]
pub struct FullViewingKey {
    ak: VerificationKey<SpendAuth>,
    nk: NullifierKey,
    ovk: OutgoingViewingKey,
    ivk: IncomingViewingKey,
}

/// The hash of a full viewing key, used as an account identifier.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "pb::FullViewingKeyHash", into = "pb::FullViewingKeyHash")]
pub struct FullViewingKeyHash(pub [u8; 32]);

impl FullViewingKey {
    pub fn controls(&self, note: &Note) -> bool {
        note.transmission_key()
            == self
                .incoming()
                .diversified_public(&note.diversified_generator())
    }

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

    pub fn nullifier_key(&self) -> &NullifierKey {
        &self.nk
    }

    /// Derive the [`Nullifier`] for a positioned note given its [`merkle::Position`] and
    /// [`note::Commitment`].
    pub fn derive_nullifier(
        &self,
        pos: penumbra_tct::Position,
        note_commitment: &note::Commitment,
    ) -> Nullifier {
        self.nk.derive_nullifier(pos, note_commitment)
    }

    /// Returns the spend verification key contained in this full viewing key.
    pub fn spend_verification_key(&self) -> &VerificationKey<SpendAuth> {
        &self.ak
    }

    /// Hashes the full viewing key into a [`FullViewingKeyHash`].
    pub fn hash(&self) -> FullViewingKeyHash {
        let hash_result = prf::expand(b"Penumbra_HashFVK", &self.nk.0.to_bytes(), self.ak.as_ref());
        let hash = hash_result.as_bytes()[..32]
            .try_into()
            .expect("hash is 32 bytes");
        FullViewingKeyHash(hash)
    }
}

impl Protobuf<pb::FullViewingKey> for FullViewingKey {}

impl TryFrom<pb::FullViewingKey> for FullViewingKey {
    type Error = anyhow::Error;

    fn try_from(value: pb::FullViewingKey) -> Result<Self, Self::Error> {
        if value.inner.len() != 64 {
            return Err(anyhow::anyhow!(
                "Wrong byte length, expected 64 but found {}",
                value.inner.len()
            ));
        }

        let ak_bytes: [u8; 32] = value.inner[0..32].try_into().unwrap();
        let nk_bytes: [u8; 32] = value.inner[32..64].try_into().unwrap();

        let ak = ak_bytes.try_into()?;
        let nk = NullifierKey(
            Fq::deserialize(&nk_bytes[..]).context("could not deserialize nullifier key")?,
        );

        Ok(FullViewingKey::from_components(ak, nk))
    }
}

impl From<FullViewingKey> for pb::FullViewingKey {
    fn from(value: FullViewingKey) -> pb::FullViewingKey {
        let mut inner = Vec::with_capacity(64);
        inner.extend_from_slice(&value.ak.to_bytes());
        inner.extend_from_slice(&value.nk.0.to_bytes());
        pb::FullViewingKey { inner }
    }
}

impl std::fmt::Display for FullViewingKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let proto = pb::FullViewingKey::from(self.clone());
        f.write_str(&bech32str::encode(
            &proto.inner,
            bech32str::full_viewing_key::BECH32_PREFIX,
            bech32str::Bech32m,
        ))
    }
}

impl std::fmt::Debug for FullViewingKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

impl std::str::FromStr for FullViewingKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        pb::FullViewingKey {
            inner: bech32str::decode(
                s,
                bech32str::full_viewing_key::BECH32_PREFIX,
                bech32str::Bech32m,
            )?,
        }
        .try_into()
    }
}

impl TryFrom<pb::FullViewingKeyHash> for FullViewingKeyHash {
    type Error = anyhow::Error;

    fn try_from(value: pb::FullViewingKeyHash) -> Result<Self, Self::Error> {
        Ok(FullViewingKeyHash(
            value
                .inner
                .try_into()
                .map_err(|_| anyhow::anyhow!("expected 32 byte array"))?,
        ))
    }
}

impl From<FullViewingKeyHash> for pb::FullViewingKeyHash {
    fn from(value: FullViewingKeyHash) -> pb::FullViewingKeyHash {
        pb::FullViewingKeyHash {
            inner: value.0.to_vec(),
        }
    }
}

impl std::fmt::Debug for FullViewingKeyHash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("FullViewingKeyHash")
            .field(&hex::encode(&self.0))
            .finish()
    }
}

impl std::fmt::Display for FullViewingKeyHash {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&hex::encode(&self.0))
    }
}
