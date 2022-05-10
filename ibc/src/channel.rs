use ibc::core::ics04_channel::channel::ChannelEnd;
use ibc_proto::ibc::core::channel::v1::Channel as RawChannel;
use penumbra_proto::{ibc as pb, Protobuf};

#[derive(Clone, Debug)]
pub struct Channel(pub ChannelEnd);

impl Protobuf<RawChannel> for Channel{}

impl TryFrom<RawChannel> for Channel {
    type Error = anyhow::Error;

    fn try_from(p: RawChannel) -> Result<Self, Self::Error> {
        let connection_end = ConnectionEnd::try_from(p)?;
        Ok(Connection(connection_end))
    }
}

impl From<Channel> for RawChannel {
    fn from(c: Channel) -> Self {
        c.0.into()
    }
}

impl From<ChannelEnd> for Channel{
    fn from(c: ChannelEnd) -> Self {
        Channel(c)
    }
}

