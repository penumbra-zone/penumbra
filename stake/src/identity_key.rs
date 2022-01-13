use penumbra_crypto::rdsa::{SpendAuth, VerificationKey};
use penumbra_proto::{serializers::bech32str, stake as pb, Protobuf};
use serde::{Deserialize, Serialize};

use crate::DelegationToken;

/// A [`SpendAuth`] [`VerificationKey`] used as a validator's identity key.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "pb::IdentityKey", into = "pb::IdentityKey")]
pub struct IdentityKey(pub VerificationKey<SpendAuth>);

impl IdentityKey {
    pub fn delegation_token(&self) -> DelegationToken {
        DelegationToken::new(self.clone())
    }
}

impl std::str::FromStr for IdentityKey {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        pb::IdentityKey {
            ik: bech32str::decode(
                s,
                crate::VALIDATOR_IDENTITY_BECH32_PREFIX,
                bech32str::Bech32m,
            )?,
        }
        .try_into()
    }
}

impl std::fmt::Display for IdentityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&bech32str::encode(
            &self.0.to_bytes(),
            crate::VALIDATOR_IDENTITY_BECH32_PREFIX,
            bech32str::Bech32m,
        ))
    }
}

impl std::fmt::Debug for IdentityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <IdentityKey as std::fmt::Display>::fmt(self, f)
    }
}

impl Protobuf<pb::IdentityKey> for IdentityKey {}

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
