use serde::{Deserialize, Deserializer, Serialize, Serializer};
use subtle_encoding::base64;

/// Deserialize base64string into Vec<u8>
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<prost_types::Any>, D::Error>
where
    D: Deserializer<'de>,
{
    let our_any = Option::<Any>::deserialize(deserializer)?;

    our_any
        .map(|v| prost_types::Any::try_from(v).map_err(serde::de::Error::custom))
        .transpose()
}

pub fn serialize<S>(value: &Option<prost_types::Any>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let our_any: Option<Any> = value.as_ref().map(|v| Any::from(v.clone()));

    our_any.serialize(serializer)
}

#[derive(Deserialize, Serialize)]
struct Any {
    type_url: String,
    value: String,
}

impl TryFrom<Any> for prost_types::Any {
    type Error = subtle_encoding::Error;

    fn try_from(value: Any) -> Result<Self, Self::Error> {
        Ok(prost_types::Any {
            type_url: value.type_url,
            value: base64::decode(&value.value)?,
        })
    }
}

impl From<prost_types::Any> for Any {
    fn from(value: prost_types::Any) -> Self {
        Any {
            type_url: value.type_url,
            value: String::from_utf8(base64::encode(value.value)).unwrap(),
        }
    }
}
