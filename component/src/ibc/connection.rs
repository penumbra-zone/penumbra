use ibc::core::ics03_connection::version::Version;
use once_cell::sync::Lazy;
use penumbra_proto::{core::ibc::v1alpha1 as pb, Protobuf};

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

pub static SUPPORTED_VERSIONS: Lazy<Vec<Version>> = Lazy::new(|| vec![Version::default()]);
