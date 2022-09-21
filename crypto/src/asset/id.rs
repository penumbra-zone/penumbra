use crate::Amount;
use ark_ff::fields::PrimeField;
use ark_serialize::CanonicalDeserialize;
use decaf377::FieldExt;
use once_cell::sync::Lazy;
use penumbra_proto::{crypto as pb, serializers::bech32str, Protobuf};
use serde::{Deserialize, Serialize};

use crate::{Fq, Value};

/// An identifier for an IBC asset type.
///
/// This is similar to, but different from, the design in [ADR001].  As in
/// ADR001, a denomination trace is hashed to a fixed-size identifier, but
/// unlike ADR001, we hash to a field element rather than a byte string.
///
/// A denomination trace looks like
///
/// - `denom` (native chain A asset)
/// - `transfer/channelToA/denom` (chain B representation of chain A asset)
/// - `transfer/channelToB/transfer/channelToA/denom` (chain C representation of chain B representation of chain A asset)
///
/// ADR001 defines the IBC asset ID as the SHA-256 hash of the denomination
/// trace.  Instead, Penumbra hashes to a field element, so that asset IDs can
/// be more easily used inside of a circuit.
///
/// [ADR001]:
/// https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-001-coin-source-tracing.md
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "pb::AssetId", into = "pb::AssetId")]
pub struct Id(pub Fq);

impl From<Id> for pb::AssetId {
    fn from(id: Id) -> Self {
        pb::AssetId {
            inner: id.0.to_bytes().to_vec(),
        }
    }
}

impl TryFrom<pb::AssetId> for Id {
    type Error = anyhow::Error;
    fn try_from(value: pb::AssetId) -> Result<Self, Self::Error> {
        let bytes: [u8; 32] = value.inner.try_into().map_err(|_| {
            anyhow::anyhow!("could not deserialize Asset ID: input vec is not 32 bytes")
        })?;
        let inner = Fq::from_bytes(bytes)?;
        Ok(Id(inner))
    }
}

impl Protobuf<pb::AssetId> for Id {}

impl TryFrom<&[u8]> for Id {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<Id, Self::Error> {
        Ok(Id(Fq::deserialize(slice)?))
    }
}

impl TryFrom<[u8; 32]> for Id {
    type Error = anyhow::Error;

    fn try_from(bytes: [u8; 32]) -> Result<Id, Self::Error> {
        Ok(Id(Fq::from_bytes(bytes)?))
    }
}

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&bech32str::encode(
            &self.0.to_bytes(),
            bech32str::asset_id::BECH32_PREFIX,
            bech32str::Bech32m,
        ))
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&bech32str::encode(
            &self.0.to_bytes(),
            bech32str::asset_id::BECH32_PREFIX,
            bech32str::Bech32m,
        ))
    }
}

impl std::str::FromStr for Id {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = bech32str::decode(s, bech32str::asset_id::BECH32_PREFIX, bech32str::Bech32m)?;
        pb::AssetId { inner }.try_into()
    }
}

/// The domain separator used to hash asset ids to value generators.
static VALUE_GENERATOR_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.value.generator").as_bytes())
});

impl Id {
    /// Compute the value commitment generator for this asset.
    pub fn value_generator(&self) -> decaf377::Element {
        decaf377::Element::encode_to_curve(&poseidon377::hash_1(
            &VALUE_GENERATOR_DOMAIN_SEP,
            self.0,
        ))
    }

    /// Convert the asset ID to bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }

    /// Create a value of this denomination.
    pub fn value(&self, amount: Amount) -> Value {
        Value {
            amount,
            asset_id: self.clone(),
        }
    }
}
