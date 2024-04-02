// TODO(kate):
// decompressed: std::sync::OnceLock<VerificationKey<SpendAuth>>,

use decaf377_rdsa::{SigningKey, SpendAuth, VerificationKey, VerificationKeyBytes};
use penumbra_proto::{
    core::component::stake::v1::CurrentValidatorRateRequest,
    // TODO: why is this not in the keys crate?
    core::keys::v1 as pb,
    serializers::bech32str::{self, validator_identity_key::BECH32_PREFIX},
    DomainType,
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// The root of a validator's identity.
///
/// This key is a [`SpendAuth`] [`VerificationKey`]; currently, the wallet
/// software reuses an account's spend authorization key as the validator
/// identity, but there is no real requirement that it must be generated that
/// way.
///
/// Using a [`SpendAuth`] key means that validators can reuse code and processes
/// designed for custodying funds to protect their identity.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "pb::IdentityKey", into = "pb::IdentityKey")]
pub struct IdentityKey {
    /// The *compressed* bytes of the identity verification key.
    bytes: VerificationKeyBytes<SpendAuth>,
    // The *decompressed* identity verification key.
    key: OnceLock<DecompressionResult>,
}

type DecompressionResult = Result<VerificationKey<SpendAuth>, decaf377_rdsa::Error>;

impl PartialOrd for IdentityKey {
    /// [`IdentityKey`]s are partially ordered according to their compressed bytes.
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.bytes.partial_cmp(&other.bytes)
    }
}

impl Ord for IdentityKey {
    /// [`IdentityKey`]s are totally ordered according to their compressed bytes.
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl From<SigningKey<SpendAuth>> for IdentityKey {
    /// An [`IdentityKey`] can be constructed from a [`decaf377_rdsa::SigningKey`].
    fn from(sk: SigningKey<SpendAuth>) -> Self {
        let vk: VerificationKey<SpendAuth> = sk.into();
        Self::from(vk)
    }
}

impl<'sk> From<&'sk SigningKey<SpendAuth>> for IdentityKey {
    /// An [`IdentityKey`] can be constructed from a reference to a [`decaf377_rdsa::SigningKey`].
    fn from(sk: &'sk SigningKey<SpendAuth>) -> Self {
        Self::from(*sk)
    }
}

impl From<VerificationKey<SpendAuth>> for IdentityKey {
    /// An [`IdentityKey`] can be constructed from a [`decaf377_rdsa::VerificationKey`].
    fn from(vk: VerificationKey<SpendAuth>) -> Self {
        let bytes: VerificationKeyBytes<SpendAuth> = vk.into();
        let key: OnceLock<_> = Ok(vk).into();

        Self { bytes, key }
    }
}

impl<'vk> From<&'vk VerificationKey<SpendAuth>> for IdentityKey {
    /// An [`IdentityKey`] can be constructed from a reference to a [`decaf377_rdsa::VerificationKey`].
    fn from(vk: &'vk VerificationKey<SpendAuth>) -> Self {
        Self::from(*vk)
    }
}

impl IdentityKey {
    /// Decompresses this identity key, returning a [`VerificationKey`].
    ///
    /// This is idempotent and can be called repeatedly, and the bytes will only decompressed
    /// once.
    pub fn key(&self) -> Result<VerificationKey<SpendAuth>, decaf377_rdsa::Error> {
        let Self { bytes, ref key } = *self;
        key.get_or_init(|| VerificationKey::try_from(bytes)).clone()
    }

    /// Returns the compressed bytes of this identity key.
    ///
    /// This can be used as an alternative to [`AsRef`] when type inference fails.
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.as_ref()
    }
}

// IMPORTANT: Changing this implementation is state-breaking.
// TODO(kate): lazily decoding bytes may be a state-breaking change.
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
// TODO(kate): lazily decoding bytes may be a state-breaking change.
impl std::fmt::Display for IdentityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&bech32str::encode(
            &self.bytes.as_ref().as_slice(),
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
            ik: ik.bytes.as_ref().to_vec(),
        }
    }
}

impl TryFrom<pb::IdentityKey> for IdentityKey {
    type Error = anyhow::Error;
    fn try_from(ik: pb::IdentityKey) -> Result<Self, Self::Error> {
        Self::try_from(ik.ik)
    }
}

impl TryFrom<Vec<u8>> for IdentityKey {
    type Error = anyhow::Error;
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&[u8]> for IdentityKey {
    type Error = anyhow::Error;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        bytes
            .try_into()
            .map(|bytes| Self {
                bytes,
                key: OnceLock::new(),
            })
            .map_err(Into::into)
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

impl AsRef<VerificationKeyBytes<SpendAuth>> for IdentityKey {
    fn as_ref(&self) -> &VerificationKeyBytes<SpendAuth> {
        &self.bytes
    }
}

// TODO(kate): define this 32 as a constant upstream in decaf377_rdsa.
impl AsRef<[u8; 32]> for IdentityKey {
    fn as_ref(&self) -> &[u8; 32] {
        self.bytes.as_ref()
    }
}

impl AsRef<[u8]> for IdentityKey {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}
