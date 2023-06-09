use ibc_types2::core::ics03_connection::version::Version;
use once_cell::sync::Lazy;
use penumbra_proto::{core::ibc::v1alpha1 as pb, DomainType, TypeUrl};

#[derive(Debug, Clone)]
pub struct ConnectionCounter(pub u64);

impl TypeUrl for ConnectionCounter {
    const TYPE_URL: &'static str = "/penumbra.core.ibc.v1alpha1.ConnectionCounter";
}

impl DomainType for ConnectionCounter {
    type Proto = pb::ConnectionCounter;
}

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
