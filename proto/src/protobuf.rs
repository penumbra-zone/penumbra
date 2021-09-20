/// A marker trait that captures the relationships between a domain type (`Self`) and a protobuf type (`P`).
pub trait Protobuf<P>: Sized
where
    P: prost::Message,
    P: std::convert::From<Self>,
    Self: std::convert::TryFrom<P>,
{
}
