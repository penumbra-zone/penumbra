use ibc::core::ics03_connection::connection::ConnectionEnd;
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

impl Protobuf<pb::Connection> for Connection {}

impl TryFrom<pb::Connection> for Connection {
    type Error = anyhow::Error;

    fn try_from(p: pb::Connection) -> Result<Self, Self::Error> {
        let end = p
            .connection_end
            .ok_or_else(|| anyhow::anyhow!("connection end not set"))?;
        let connection_end = ConnectionEnd::try_from(end)?;
        Ok(Connection(connection_end))
    }
}

impl From<Connection> for pb::Connection {
    fn from(c: Connection) -> Self {
        pb::Connection {
            connection_end: Some(c.0.into()),
        }
    }
}

impl From<ConnectionEnd> for Connection {
    fn from(c: ConnectionEnd) -> Self {
        Connection(c)
    }
}
