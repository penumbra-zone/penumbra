/// A marker trait that captures the relationships between a domain type (`Self`) and a protobuf type (`P`).
pub trait Protobuf: Sized
where
    Self::Protobuf: prost::Message + Default,
    Self::Protobuf: std::convert::From<Self>,
    Self: std::convert::TryFrom<Self::Protobuf> + Clone,
    <Self as std::convert::TryFrom<Self::Protobuf>>::Error: Into<anyhow::Error>,
{
    /// The protobuf type corresponding to the domain type `Self`.
    type Protobuf;

    /// Encode this domain type to a byte vector, via proto type `P`.
    fn encode_to_vec(&self) -> Vec<u8> {
        use prost::Message;
        Self::Protobuf::from(self.clone()).encode_to_vec()
    }

    /// Decode this domain type from a byte buffer, via proto type `P`.
    fn decode<B: bytes::Buf>(buf: B) -> Result<Self, anyhow::Error> {
        <Self::Protobuf as prost::Message>::decode(buf)?
            .try_into()
            .map_err(Into::into)
    }
}
