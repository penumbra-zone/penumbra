//! Asset types and identifiers.
use once_cell::sync::Lazy;
use std::convert::{TryFrom, TryInto};

use regex::Regex;
use subtle_encoding::bech32;
use tendermint::PublicKey;

use ark_ff::fields::PrimeField;
use decaf377::FieldExt;

use crate::Fq;

pub const NATIVE_ASSET_DENOM: &str = "penumbra";
pub const PENUMBRA_BECH32_VALIDATOR_PREFIX: &str = "penumbravalpub";
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

/// Validator-specific unique assets, used for identifying staking tokens
/// associated with a particular validator.
pub struct ValidatorAsset {
    validator_pubkey: PublicKey,
}

impl ValidatorAsset {
    pub fn new(pk: PublicKey) -> Self {
        ValidatorAsset {
            validator_pubkey: pk,
        }
    }

    pub fn denom(&self) -> Denom {
        Denom(format!(
            "d{}-{}",
            NATIVE_ASSET_DENOM,
            self.validator_pubkey
                .to_bech32(PENUMBRA_BECH32_VALIDATOR_PREFIX)
        ))
    }

    pub fn id(&self) -> Id {
        Id::from(self.denom())
    }
}

pub fn is_validator_denom(d: &Denom) -> bool {
    let re = Regex::new(&format!(
        r"^d{}-({}{}\w+)$",
        NATIVE_ASSET_DENOM,
        PENUMBRA_BECH32_VALIDATOR_PREFIX,
        // Note, if tendermint-rs ever changes the bech32 separator from default,
        // this will break
        bech32::DEFAULT_SEPARATOR
    ))
    .unwrap();
    let caps = re.captures(&d.0);
    if caps.is_none() {
        // There was no bech32 address captured
        return false;
    }

    let bech32_address = caps.unwrap().get(1).map_or("", |m| m.as_str());

    re.is_match(&d.0) && bech32::decode(bech32_address.to_lowercase()).is_ok()
}

pub struct Denom(pub String);

impl std::fmt::Display for Denom {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.denom().0)
    }
}

impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ark_ff::BigInteger;
        let bytes = self.0.into_repr().to_bytes_le();
        f.write_fmt(format_args!("asset::Id({})", hex::encode(&bytes)))
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use ark_ff::BigInteger;
        let bytes = self.0.into_repr().to_bytes_le();
        f.write_fmt(format_args!("asset::Id({})", hex::encode(&bytes)))
    }
}

impl std::str::FromStr for Id {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes: [u8; 32] = s.as_bytes().try_into().map_err(|_| {
            anyhow::anyhow!("could not deserialize Asset ID: input vec is not 32 bytes")
        })?;
        let inner = Fq::from_bytes(bytes)?;
        Ok(Id(inner))
    }
}

impl From<&str> for Denom {
    fn from(strin: &str) -> Denom {
        Denom(strin.to_string())
    }
}

impl Denom {
    /// Returns the asset ID corresponding to this denomination.
    pub fn id(&self) -> Id {
        // Convert an asset name to an asset ID by hashing to a scalar
        Id(Fq::from_le_bytes_mod_order(
            // XXX choice of hash function?
            blake2b_simd::Params::default()
                .personal(b"Penumbra_AssetID")
                .hash(self.0.as_ref())
                .as_bytes(),
        ))
    }
}

impl From<Denom> for Id {
    fn from(denom: Denom) -> Id {
        // Putting the impl in id() rather than here means
        // id() doesn't need to clone the string
        denom.id()
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

    #[test]
    fn denom_type_inference() {
        // basic test around denomination type inference

        let bc = Denom("bottlecaps".to_string());
        let atom = Denom("HubPort/HubChannel/atom".to_string());
        let delegated = Denom(format!(
            "d{}-{}{}zcjduepq5nvpxvunnsvmcu499qxewr8pzxjuq9x5f868rp7dqnzgd59z9nqqc7hg9k",
            NATIVE_ASSET_DENOM,
            PENUMBRA_BECH32_VALIDATOR_PREFIX,
            bech32::DEFAULT_SEPARATOR
        ));
        let not_delegated = Denom(
            "dIWishIWasADelegatedToken-SomeStuff15zcu95aqk8pd8g93ctf6pvwz6wstrsknmrfr5t"
                .to_string(),
        );

        assert!(!is_validator_denom(&bc));
        assert!(!is_validator_denom(&atom));
        assert!(is_validator_denom(&delegated));
        assert!(!is_validator_denom(&not_delegated));
    }
}
