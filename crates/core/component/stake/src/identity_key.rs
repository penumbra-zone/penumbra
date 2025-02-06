use penumbra_sdk_proto::{
    core::component::stake::v1::CurrentValidatorRateRequest,
    // TODO: why is this not in the keys crate?
    core::keys::v1 as pb,
    serializers::bech32str::{self, validator_identity_key::BECH32_PREFIX},
    DomainType,
};
use serde::{Deserialize, Serialize};

use decaf377_rdsa::{SpendAuth, VerificationKeyBytes};

/// The length of an identity key in bytes.
/// TODO(erwan): move this to the keys crate, one day.
pub const IDENTITY_KEY_LEN_BYTES: usize = 32;

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
pub struct IdentityKey(pub VerificationKeyBytes<SpendAuth>);

impl IdentityKey {
    pub fn to_bytes(&self) -> [u8; IDENTITY_KEY_LEN_BYTES] {
        self.0.into()
    }
}

// IMPORTANT: Changing this implementation is state-breaking.
impl std::str::FromStr for IdentityKey {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        pb::IdentityKey {
            ik: bech32str::decode(s, BECH32_PREFIX, bech32str::Bech32m)?,
        }
        .try_into()
    }
}

// IMPORTANT: Changing this implementation is state-breaking.
impl std::fmt::Display for IdentityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&bech32str::encode(
            self.0.as_ref(),
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
            ik: ik.0.as_ref().to_vec(),
        }
    }
}

impl TryFrom<pb::IdentityKey> for IdentityKey {
    type Error = anyhow::Error;
    fn try_from(ik: pb::IdentityKey) -> Result<Self, Self::Error> {
        Ok(Self(ik.ik.as_slice().try_into()?))
    }
}

impl From<IdentityKey> for CurrentValidatorRateRequest {
    fn from(k: IdentityKey) -> Self {
        CurrentValidatorRateRequest {
            identity_key: Some(k.into()),
        }
    }
}

impl TryFrom<CurrentValidatorRateRequest> for IdentityKey {
    type Error = anyhow::Error;
    fn try_from(value: CurrentValidatorRateRequest) -> Result<Self, Self::Error> {
        value
            .identity_key
            .ok_or_else(|| anyhow::anyhow!("empty CurrentValidatorRateRequest message"))?
            .try_into()
    }
}
