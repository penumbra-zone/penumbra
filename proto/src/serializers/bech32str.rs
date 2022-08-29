//! Bech32 serializers. Because Bech32 is parameterized by the HRP, this module
//! implements (internal) helper functions that are used in submodules that fill
//! in parameters for various key types.

use bech32::{FromBase32, ToBase32};
// re-exporting allows the convenience methods to be used without referencing
// the underlying bech32 crate.
pub use bech32::{Variant, Variant::*};
use serde::{Deserialize, Deserializer, Serializer};

/// Convenience method for (general-purpose) Bech32 decoding.
///
/// Works around a bit of awkwardness in the [`bech32`] API.
pub fn decode(
    string: &str,
    expected_hrp: &str,
    expected_variant: Variant,
) -> anyhow::Result<Vec<u8>> {
    let (hrp, data, variant) = bech32::decode(string)?;

    if variant != expected_variant {
        return Err(anyhow::anyhow!(
            "wrong bech32 variant {:?}, expected {:?}",
            variant,
            expected_variant
        ));
    }
    if hrp != expected_hrp {
        return Err(anyhow::anyhow!(
            "wrong bech32 human readable part {}, expected {}",
            hrp,
            expected_hrp
        ));
    }

    Ok(Vec::from_base32(&data).expect("bech32 decoding produces valid base32"))
}

/// Convenience method for (general-purpose) Bech32 encoding.
///
/// Works around a bit of awkwardness in the [`bech32`] API.
/// Panics if the HRP is invalid.
pub fn encode(data: &[u8], hrp: &str, variant: Variant) -> String {
    bech32::encode(hrp, data.to_base32(), variant).expect("HRP should be valid")
}

fn deserialize_bech32<'de, D>(
    deserializer: D,
    expected_hrp: &str,
    expected_variant: Variant,
) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    decode(
        &Option::<String>::deserialize(deserializer)?.unwrap_or_default(),
        expected_hrp,
        expected_variant,
    )
    .map_err(serde::de::Error::custom)
}

fn serialize_bech32<S, T>(
    value: &T,
    serializer: S,
    hrp: &str,
    variant: Variant,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    serializer.serialize_str(&encode(value.as_ref(), hrp, variant))
}

pub mod validator_identity_key {
    use super::*;

    /// The Bech32 prefix used for validator identity keys.
    pub const BECH32_PREFIX: &str = "penumbravalid";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_bech32(deserializer, BECH32_PREFIX, Variant::Bech32m)
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        serialize_bech32(value, serializer, BECH32_PREFIX, Variant::Bech32m)
    }
}

pub mod validator_governance_key {
    use super::*;

    /// The Bech32 prefix used for validator governance keys.
    pub const BECH32_PREFIX: &str = "penumbragovern";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_bech32(deserializer, BECH32_PREFIX, Variant::Bech32m)
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        serialize_bech32(value, serializer, BECH32_PREFIX, Variant::Bech32m)
    }
}

pub mod address {
    use super::*;

    /// The Bech32 prefix used for addresses.
    pub const BECH32_PREFIX: &str = "penumbrav2t";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_bech32(deserializer, BECH32_PREFIX, Variant::Bech32m)
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        serialize_bech32(value, serializer, BECH32_PREFIX, Variant::Bech32m)
    }
}

pub mod asset_id {
    use super::*;

    /// The Bech32 prefix used for asset IDs.
    pub const BECH32_PREFIX: &str = "passet";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_bech32(deserializer, BECH32_PREFIX, Variant::Bech32m)
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        serialize_bech32(value, serializer, BECH32_PREFIX, Variant::Bech32m)
    }
}

pub mod full_viewing_key {
    use super::*;

    /// The Bech32 prefix used for full viewing keys.
    pub const BECH32_PREFIX: &str = "penumbrafullviewingkey";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_bech32(deserializer, BECH32_PREFIX, Variant::Bech32m)
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        serialize_bech32(value, serializer, BECH32_PREFIX, Variant::Bech32m)
    }
}

pub mod spend_key {
    use super::*;

    /// The Bech32 prefix used for spend keys.
    pub const BECH32_PREFIX: &str = "penumbraspendkey";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_bech32(deserializer, BECH32_PREFIX, Variant::Bech32m)
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        serialize_bech32(value, serializer, BECH32_PREFIX, Variant::Bech32m)
    }
}

pub mod lp_id {
    use super::*;

    /// The Bech32 prefix used for LP IDs.
    pub const BECH32_PREFIX: &str = "plpid";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_bech32(deserializer, BECH32_PREFIX, Variant::Bech32m)
    }

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        serialize_bech32(value, serializer, BECH32_PREFIX, Variant::Bech32m)
    }
}
