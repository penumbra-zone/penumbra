use ibc::core::ics03_connection::connection::ConnectionEnd;
use ibc::core::ics03_connection::version::Version;
use ibc_proto::ibc::core::connection::v1::ConnectionEnd as RawConnectionEnd;
use once_cell::sync::Lazy;
use penumbra_proto::{ibc as pb, Protobuf};

#[derive(Debug, Clone)]
pub struct ConnectionCounter(pub u64);

impl Protobuf<pb::ConnectionCounter> for ConnectionCounter {}

impl TryFrom<pb::ConnectionCounter> for ConnectionCounter {
    type Error = anyhow::Error;

    fn try_from(p: pb::ConnectionCounter) -> Result<Self, Self::Error> {
        Ok(ConnectionCounter(p.counter))
    }
}

impl From<ConnectionCounter> for pb::ConnectionCounter {
    fn from(c: ConnectionCounter) -> Self {
        pb::ConnectionCounter { counter: c.0 }
    }
}

#[derive(Debug, Clone)]
pub struct Connection(pub ConnectionEnd);

impl Protobuf<RawConnectionEnd> for Connection {}

impl TryFrom<RawConnectionEnd> for Connection {
    type Error = anyhow::Error;

    fn try_from(p: RawConnectionEnd) -> Result<Self, Self::Error> {
        let connection_end = ConnectionEnd::try_from(p)?;
        Ok(Connection(connection_end))
    }
}

impl From<Connection> for RawConnectionEnd {
    fn from(c: Connection) -> Self {
        c.0.into()
    }
}

impl From<ConnectionEnd> for Connection {
    fn from(c: ConnectionEnd) -> Self {
        Connection(c)
    }
}

pub static SUPPORTED_VERSIONS: Lazy<Vec<Version>> = Lazy::new(|| vec![Version::default()]);
