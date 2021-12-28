use serde::{Deserialize, Deserializer, Serializer};
use subtle_encoding::base64;

/// Deserialize base64string into Vec<u8>
pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = Option::<String>::deserialize(deserializer)?.unwrap_or_default();
    base64::decode(&string).map_err(serde::de::Error::custom)
}

/// Serialize from T into base64string
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    let base64_bytes = base64::encode(value.as_ref());
    let base64_string = String::from_utf8(base64_bytes).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&base64_string)
}
