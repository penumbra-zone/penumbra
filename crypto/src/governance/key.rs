use penumbra_proto::{
    core::crypto::v1alpha1 as pb,
    serializers::bech32str::{self, validator_governance_key::BECH32_PREFIX},
    DomainType,
};
use serde::{Deserialize, Serialize};

use crate::rdsa::{SpendAuth, VerificationKey};

/// The root of a validator's governance identity (which may be distinct from its main identity, to
/// allow cold storage of validator keys).
///
/// This key is a [`SpendAuth`] [`VerificationKey`]; currently, the wallet software reuses an
/// account's spend authorization key as the identity key and also as the governance key, but there
/// is no real requirement that it must be generated that way.
///
/// Using a [`SpendAuth`] key means that validators can reuse code and processes designed for
/// custodying funds to protect their identity.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "pb::GovernanceKey", into = "pb::GovernanceKey")]
pub struct GovernanceKey(pub VerificationKey<SpendAuth>);

impl std::str::FromStr for GovernanceKey {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        pb::GovernanceKey {
            gk: bech32str::decode(s, BECH32_PREFIX, bech32str::Bech32m)?,
        }
        .try_into()
    }
}

impl std::fmt::Display for GovernanceKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&bech32str::encode(
            &self.0.to_bytes(),
            BECH32_PREFIX,
            bech32str::Bech32m,
        ))
    }
}

impl std::fmt::Debug for GovernanceKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <GovernanceKey as std::fmt::Display>::fmt(self, f)
    }
}

impl DomainType for GovernanceKey {
    type Proto = pb::GovernanceKey;
}

impl From<GovernanceKey> for pb::GovernanceKey {
    fn from(gk: GovernanceKey) -> Self {
        pb::GovernanceKey {
            gk: gk.0.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::GovernanceKey> for GovernanceKey {
    type Error = anyhow::Error;
    fn try_from(gk: pb::GovernanceKey) -> Result<Self, Self::Error> {
        Ok(Self(gk.gk.as_slice().try_into()?))
    }
}
