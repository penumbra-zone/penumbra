use penumbra_proto::{
    client::v1alpha1::NextValidatorRateRequest,
    core::crypto::v1alpha1 as pb,
    serializers::bech32str::{self, validator_identity_key::BECH32_PREFIX},
    DomainType,
};
use serde::{Deserialize, Serialize};

use crate::rdsa::{SpendAuth, VerificationKey};

/// The root of a validator's identity.
///
/// This key is a [`SpendAuth`] [`VerificationKey`]; currently, the wallet
/// software reuses an account's spend authorization key as the validator
/// identity, but there is no real requirement that it must be generated that
/// way.
///
/// Using a [`SpendAuth`] key means that validators can reuse code and processes
/// designed for custodying funds to protect their identity.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "pb::IdentityKey", into = "pb::IdentityKey")]
pub struct IdentityKey(pub VerificationKey<SpendAuth>);

impl std::str::FromStr for IdentityKey {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        pb::IdentityKey {
            ik: bech32str::decode(s, BECH32_PREFIX, bech32str::Bech32m)?,
        }
        .try_into()
    }
}

impl std::fmt::Display for IdentityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&bech32str::encode(
            &self.0.to_bytes(),
            BECH32_PREFIX,
            bech32str::Bech32m,
        ))
    }
}

impl std::fmt::Debug for IdentityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <IdentityKey as std::fmt::Display>::fmt(self, f)
    }
}

impl DomainType for IdentityKey {
    type Proto = pb::IdentityKey;
}

impl From<IdentityKey> for pb::IdentityKey {
    fn from(ik: IdentityKey) -> Self {
        pb::IdentityKey {
            ik: ik.0.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::IdentityKey> for IdentityKey {
    type Error = anyhow::Error;
    fn try_from(ik: pb::IdentityKey) -> Result<Self, Self::Error> {
        Ok(Self(ik.ik.as_slice().try_into()?))
    }
}

impl From<IdentityKey> for NextValidatorRateRequest {
    fn from(k: IdentityKey) -> Self {
        NextValidatorRateRequest {
            identity_key: Some(k.into()),
            chain_id: Default::default(),
        }
    }
}

impl TryFrom<NextValidatorRateRequest> for IdentityKey {
    type Error = anyhow::Error;
    fn try_from(value: NextValidatorRateRequest) -> Result<Self, Self::Error> {
        value
            .identity_key
            .ok_or_else(|| anyhow::anyhow!("empty NextValidatorRateRequest message"))?
            .try_into()
    }
}
