use super::*;

pub fn serialize<S>(fq: &Fq, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_bytes(&fq.to_bytes())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Fq, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_bytes(FqVisitor)
}

struct FqVisitor;

impl<'de> Visitor<'de> for FqVisitor {
    type Value = Fq;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a 32-byte array representing a field element")
    }

    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| serde::de::Error::invalid_length(bytes.len(), &"exactly 32 bytes"))?;
        let fq = Fq::from_bytes_checked(&bytes).map_err(|e| serde::de::Error::custom(e))?;
        Ok(fq)
    }
}
