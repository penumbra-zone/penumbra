use bytes::Bytes;
use serde::{Deserialize, Deserializer, Serializer};

/// Deserialize hexstring into Bytes
pub fn deserialize<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    hex::decode(&string)
        .map_err(serde::de::Error::custom)
        .map(Into::into)
}

/// Serialize from T into hexstring
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    serializer.serialize_str(&hex::encode(value.as_ref()))
}
