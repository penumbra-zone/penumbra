/// A marker trait that captures the relationships between a domain type (`Self`) and a protobuf type (`P`).
pub trait Protobuf<P>: Sized
where
    P: prost::Message + Default,
    P: std::convert::From<Self>,
    Self: std::convert::TryFrom<P> + Clone,
    <Self as std::convert::TryFrom<P>>::Error: Into<anyhow::Error>,
{
    /// Encode this domain type to a byte vector, via proto type `P`.
    fn encode_to_vec(&self) -> Vec<u8> {
        P::from(self.clone()).encode_to_vec()
    }

    /// Decode this domain type from a byte buffer, via proto type `P`.
    fn decode<B: bytes::Buf>(buf: B) -> Result<Self, anyhow::Error> {
        <P as prost::Message>::decode(buf)?
            .try_into()
            .map_err(Into::into)
    }
}
