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
        self.to_proto().encode_to_vec()
    }

    /// Convert this domain type to the associated proto type.
    ///
    /// This uses the `From` impl internally, so it works exactly
    /// like `.into()`, but does not require type inference.
    fn to_proto(&self) -> P {
        P::from(self.clone())
    }

    /// Decode this domain type from a byte buffer, via proto type `P`.
    fn decode<B: bytes::Buf>(buf: B) -> Result<Self, anyhow::Error> {
        <P as prost::Message>::decode(buf)?
            .try_into()
            .map_err(Into::into)
    }
}
