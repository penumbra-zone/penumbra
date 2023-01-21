use anyhow::Context;
use ark_ff::PrimeField;
use ark_serialize::CanonicalDeserialize;
use decaf377::FieldExt;
use once_cell::sync::Lazy;
use penumbra_proto::{core::crypto::v1alpha1 as pb, serializers::bech32str, Protobuf};
use poseidon377::hash_2;
use rand_core::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use super::{AddressIndex, DiversifierKey, IncomingViewingKey, NullifierKey, OutgoingViewingKey};
use crate::{
    fmd, ka,
    note::Commitment,
    prf,
    rdsa::{SpendAuth, VerificationKey},
    Address, Fq, Fr, Note, Nullifier,
};

pub(crate) static IVK_DOMAIN_SEP: Lazy<Fq> =
    Lazy::new(|| Fq::from_le_bytes_mod_order(b"penumbra.derive.ivk"));

static ACCOUNT_ID_DOMAIN_SEP: Lazy<Fq> =
    Lazy::new(|| Fq::from_le_bytes_mod_order(b"Penumbra_HashFVK"));

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
#[serde(try_from = "pb::AccountId", into = "pb::AccountId")]
pub struct AccountID(pub [u8; 32]);

impl FullViewingKey {
    pub fn controls(&self, note: &Note) -> bool {
        *note.transmission_key()
            == self
                .incoming()
                .diversified_public(&note.diversified_generator())
    }

    /// Derive a shielded payment address with the given [`AddressIndex`].
    pub fn payment_address(&self, index: AddressIndex) -> (Address, fmd::DetectionKey) {
        self.incoming().payment_address(index)
    }

    /// Derive a random ephemeral address.
    pub fn ephemeral_address<R: RngCore + CryptoRng>(
        &self,
        rng: R,
    ) -> (Address, fmd::DetectionKey) {
        self.incoming().ephemeral_address(rng)
    }

    /// Returns the index of the given address, if the address is viewed by this
    /// viewing key; otherwise, returns `None`.
    pub fn address_index(&self, address: &Address) -> Option<AddressIndex> {
        self.incoming().address_index(address)
    }

    /// Construct a full viewing key from its components.
    pub fn from_components(ak: VerificationKey<SpendAuth>, nk: NullifierKey) -> Self {
        let ovk = {
            let hash_result = prf::expand(b"Penumbra_DeriOVK", &nk.0.to_bytes(), ak.as_ref());
            let mut ovk = [0; 32];
            ovk.copy_from_slice(&hash_result.as_bytes()[0..32]);
            ovk
        };

        let dk = {
            let hash_result = prf::expand(b"Penumbra_DerivDK", &nk.0.to_bytes(), ak.as_ref());
            let mut dk = [0; 16];
            dk.copy_from_slice(&hash_result.as_bytes()[0..16]);
            dk
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

    /// Derive the [`Nullifier`] for a positioned note or swap given its [`merkle::Position`]
    /// and [`Commitment`].
    pub fn derive_nullifier(
        &self,
        pos: penumbra_tct::Position,
        state_commitment: &Commitment,
    ) -> Nullifier {
        self.nk.derive_nullifier(pos, state_commitment)
    }

    /// Returns the spend verification key contained in this full viewing key.
    pub fn spend_verification_key(&self) -> &VerificationKey<SpendAuth> {
        &self.ak
    }

    /// Hashes the full viewing key into an [`AccountID`].
    pub fn hash(&self) -> AccountID {
        let hash_result = hash_2(
            &ACCOUNT_ID_DOMAIN_SEP,
            (
                self.nk.0,
                Fq::from_le_bytes_mod_order(&self.ak.to_bytes()[..]),
            ),
        );
        let hash = hash_result.to_bytes()[..32]
            .try_into()
            .expect("hash is 32 bytes");
        AccountID(hash)
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

impl TryFrom<pb::AccountId> for AccountID {
    type Error = anyhow::Error;

    fn try_from(value: pb::AccountId) -> Result<Self, Self::Error> {
        Ok(AccountID(
            value
                .inner
                .try_into()
                .map_err(|_| anyhow::anyhow!("expected 32 byte array"))?,
        ))
    }
}

impl From<AccountID> for pb::AccountId {
    fn from(value: AccountID) -> pb::AccountId {
        pb::AccountId {
            inner: value.0.to_vec(),
        }
    }
}

impl std::fmt::Debug for AccountID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("AccountID")
            .field(&hex::encode(self.0))
            .finish()
    }
}

impl std::fmt::Display for AccountID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}
