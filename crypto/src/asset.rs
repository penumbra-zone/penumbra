//! Asset types and identifiers.
use std::convert::{TryFrom, TryInto};

use ark_ff::fields::PrimeField;
use decaf377::FieldExt;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::Fq;

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
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub Fq);

pub struct Denom(pub String);

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ark_ff::BigInteger;
        let bytes = self.0.into_repr().to_bytes_le();
        f.write_fmt(format_args!("asset::Id({})", hex::encode(&bytes)))
    }
}

impl From<&str> for Denom {
    fn from(strin: &str) -> Denom {
        Denom(strin.to_string())
    }
}

impl From<Denom> for Id {
    fn from(denom: Denom) -> Id {
        // Convert an asset name to an asset ID by hashing to a scalar
        Id(Fq::from_le_bytes_mod_order(
            // XXX choice of hash function?
            blake2b_simd::Params::default()
                .personal(b"Penumbra_AssetID")
                .hash(denom.0.as_ref())
                .as_bytes(),
        ))
    }
}

impl TryFrom<Vec<u8>> for Id {
    type Error = anyhow::Error;

    fn try_from(vec: Vec<u8>) -> Result<Id, Self::Error> {
        let bytes: [u8; 32] = vec.try_into().map_err(|_| {
            anyhow::anyhow!("could not deserialize Asset ID: input vec is not 32 bytes")
        })?;
        let inner = Fq::from_bytes(bytes)?;
        Ok(Id(inner))
    }
}

/// The domain separator used to hash asset ids to value generators.
static VALUE_GENERATOR_DOMAIN_SEP: Lazy<Fq> = Lazy::new(|| {
    Fq::from_le_bytes_mod_order(blake2b_simd::blake2b(b"penumbra.value.generator").as_bytes())
});

impl Id {
    /// Compute the value commitment generator for this asset.
    pub fn value_generator(&self) -> decaf377::Element {
        decaf377::Element::map_to_group_cdh(&poseidon377::hash_1(
            &VALUE_GENERATOR_DOMAIN_SEP,
            self.0,
        ))
    }

    /// Convert the asset ID to bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_bytes()
    }
}

#[derive(Serialize, Deserialize)]
pub struct DenomUnit {
    /// The name of this denomination, e.g. `upenumbra`.
    pub denom: String,
    /// The exponent of this denomination, if the minimum denomination, `0`.
    pub exponent: u8,
    /// A list of alternative aliases for the denomination.
    pub aliases: Vec<String>,
}

/// Metadata about each asset including the minimum denomination.
///
/// Based on [ADR-024](https://docs.cosmos.network/master/architecture/adr-024-coin-metadata.html).
#[derive(Serialize, Deserialize)]
pub struct AssetList {
    /// The description of this asset, e.g. `The native token for the Penumbra zone.`
    pub description: String,
    /// A list of alternative denominations that can be used, e.g. `upenumbra`, `penumbra`.
    pub denom_unit: Vec<DenomUnit>,
    /// The base unit of the asset, e.g. `upenumbra`. Calculations should be done with the base.
    pub base: String,
    /// The unit that should be used in displaying e.g. balances, e.g. `penumbra`.
    pub display: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_up_some_fake_asset_ids() {
        // marked for future deletion
        // not really a test, just a way to exercise the code

        let pen_trace = Denom("penumbra".to_string());
        let atom_trace = Denom("HubPort/HubChannel/atom".to_string());

        let pen_id = Id::from(pen_trace);
        let atom_id = Id::from(atom_trace);

        dbg!(pen_id);
        dbg!(atom_id);

        dbg!(pen_id.value_generator());
        dbg!(atom_id.value_generator());
    }
}
