use ark_ff::ToConstraintField;
use ark_serialize::CanonicalDeserialize;
use decaf377::Fq;
use once_cell::sync::Lazy;
use penumbra_num::Amount;
use penumbra_proto::{penumbra::core::asset::v1 as pb, serializers::bech32str, DomainType};
use serde::{Deserialize, Serialize};

use crate::Value;

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
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(try_from = "pb::AssetId", into = "pb::AssetId")]
pub struct Id(pub Fq);

impl From<Id> for pb::AssetId {
    fn from(id: Id) -> Self {
        pb::AssetId {
            inner: id.0.to_bytes().to_vec(),
            // Never produce a proto encoding with the alt string encoding.
            alt_bech32m: String::new(),
            // Never produce a proto encoding with the alt base denom.
            alt_base_denom: String::new(),
        }
    }
}

impl TryFrom<pb::AssetId> for Id {
    type Error = anyhow::Error;
    fn try_from(value: pb::AssetId) -> Result<Self, Self::Error> {
        if !value.inner.is_empty() {
            if !value.alt_base_denom.is_empty() || !value.alt_bech32m.is_empty() {
                anyhow::bail!(
                    "AssetId proto has both inner and alt_bech32m or alt_base_denom fields set"
                );
            }
            value.inner.as_slice().try_into()
        } else if !value.alt_bech32m.is_empty() {
            value.alt_bech32m.parse()
        } else if !value.alt_base_denom.is_empty() {
            Ok(Self::from_raw_denom(&value.alt_base_denom))
        } else {
            Err(anyhow::anyhow!(
                "AssetId proto has neither inner nor alt_bech32m nor alt_base_denom fields set"
            ))
        }
    }
}

impl DomainType for Id {
    type Proto = pb::AssetId;
}

impl TryFrom<&[u8]> for Id {
    type Error = anyhow::Error;

    fn try_from(slice: &[u8]) -> Result<Id, Self::Error> {
        Ok(Id(Fq::deserialize_compressed(slice)?))
    }
}

impl TryFrom<[u8; 32]> for Id {
    type Error = anyhow::Error;

    fn try_from(bytes: [u8; 32]) -> Result<Id, Self::Error> {
        Ok(Id(Fq::from_bytes_checked(&bytes).expect("convert to bytes")))
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
        pb::AssetId {
            inner,
            alt_bech32m: String::new(),
            alt_base_denom: String::new(),
        }
        .try_into()
    }
}

impl ToConstraintField<Fq> for Id {
    fn to_field_elements(&self) -> Option<Vec<Fq>> {
        let mut elements = Vec::new();
        elements.extend_from_slice(&[self.0]);
        Some(elements)
    }
}

/// The domain separator used to hash asset ids to value generators.
pub static VALUE_GENERATOR_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
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
            asset_id: *self,
        }
    }

    pub(super) fn from_raw_denom(base_denom: &str) -> Self {
        Id(Fq::from_le_bytes_mod_order(
            // XXX choice of hash function?
            blake2b_simd::Params::default()
                .personal(b"Penumbra_AssetID")
                .hash(base_denom.as_bytes())
                .as_bytes(),
        ))
    }
}
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn asset_id_encoding() {
        let id = Id::from_raw_denom("upenumbra");

        let bech32m_id = format!("{id}");

        let id2 = Id::from_str(&bech32m_id).expect("can decode valid asset id");

        use penumbra_proto::Message;

        let proto = id.encode_to_vec();
        let proto2 = pb::AssetId {
            alt_bech32m: bech32m_id,
            ..Default::default()
        }
        .encode_to_vec();
        let proto3 = pb::AssetId {
            alt_base_denom: "upenumbra".to_owned(),
            ..Default::default()
        }
        .encode_to_vec();

        let id3 = Id::decode(proto.as_ref()).expect("can decode valid asset id");
        let id4 = Id::decode(proto2.as_ref()).expect("can decode valid asset id");
        let id5 = Id::decode(proto3.as_ref()).expect("can decode valid asset id");

        assert_eq!(id2, id);
        assert_eq!(id3, id);
        assert_eq!(id4, id);
        assert_eq!(id5, id);
    }
}
