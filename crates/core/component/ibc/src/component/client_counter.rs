use ibc_types::core::client::Height;
use ibc_types::core::connection::ConnectionId;
use penumbra_proto::{penumbra::core::component::ibc::v1alpha1 as pb, DomainType};

#[derive(Clone, Debug)]
pub struct ClientCounter(pub u64);

impl DomainType for ClientCounter {
    type Proto = pb::ClientCounter;
}

impl TryFrom<pb::ClientCounter> for ClientCounter {
    type Error = anyhow::Error;

    fn try_from(p: pb::ClientCounter) -> Result<Self, Self::Error> {
        Ok(ClientCounter(p.counter))
    }
}

impl From<ClientCounter> for pb::ClientCounter {
    fn from(c: ClientCounter) -> Self {
        pb::ClientCounter { counter: c.0 }
    }
}

#[derive(Clone, Debug)]
pub struct VerifiedHeights {
    pub heights: Vec<Height>,
}

impl DomainType for VerifiedHeights {
    type Proto = pb::VerifiedHeights;
}

impl TryFrom<pb::VerifiedHeights> for VerifiedHeights {
    type Error = anyhow::Error;

    fn try_from(msg: pb::VerifiedHeights) -> Result<Self, Self::Error> {
        let heights = msg.heights.into_iter().map(TryFrom::try_from).collect();
        match heights {
            Ok(heights) => Ok(VerifiedHeights { heights }),
            Err(e) => anyhow::bail!(format!("invalid height: {e}")),
        }
    }
}

impl From<VerifiedHeights> for pb::VerifiedHeights {
    fn from(d: VerifiedHeights) -> Self {
        Self {
            heights: d.heights.into_iter().map(|h| h.into()).collect(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ClientConnections {
    pub connection_ids: Vec<ConnectionId>,
}

impl DomainType for ClientConnections {
    type Proto = pb::ClientConnections;
}

impl TryFrom<pb::ClientConnections> for ClientConnections {
    type Error = anyhow::Error;

    fn try_from(msg: pb::ClientConnections) -> Result<Self, Self::Error> {
        Ok(ClientConnections {
            connection_ids: msg
                .connections
                .into_iter()
                .map(|h| h.parse())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl From<ClientConnections> for pb::ClientConnections {
    fn from(d: ClientConnections) -> Self {
        Self {
            connections: d
                .connection_ids
                .into_iter()
                .map(|h| h.as_str().to_string())
                .collect(),
        }
    }
}
