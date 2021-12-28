use serde::{Deserialize, Deserializer, Serializer};

/// Deserialize hexstring into Vec<u8>
pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = Option::<String>::deserialize(deserializer)?.unwrap_or_default();
    hex::decode(&string).map_err(serde::de::Error::custom)
}

/// Serialize from T into hexstring
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    serializer.serialize_str(&hex::encode(value.as_ref()))
}
