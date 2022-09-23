use crate::ibc::ibc_handler::{AppHandler, AppHandlerCheck, AppHandlerExecute};
use crate::Context;
use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics04_channel::channel::Order as ChannelOrder;
use ibc::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
use ibc::core::ics04_channel::msgs::chan_close_confirm::MsgChannelCloseConfirm;
use ibc::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
use ibc::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
use ibc::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
use ibc::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use ibc::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
use ibc::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use ibc::core::ics04_channel::msgs::timeout::MsgTimeout;
use ibc::core::ics04_channel::Version;
use penumbra_proto::core::ibc::v1alpha1::FungibleTokenPacketData;
use penumbra_storage::{State, StateExt};
use prost::Message;
use tracing::instrument;

#[allow(dead_code)]
pub struct ICS20Transfer {
    state: State,
}

impl ICS20Transfer {
    #[instrument(name = "ics20_transfer", skip(state))]
    pub fn new(state: State) -> Self {
        Self { state }
    }
}

// TODO: ICS20 implementation.
// see: https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer
// TODO (ava): add versioning to AppHandlers
#[async_trait]
impl AppHandlerCheck for ICS20Transfer {
    async fn chan_open_init_check(&self, _ctx: Context, msg: &MsgChannelOpenInit) -> Result<()> {
        if msg.channel.ordering != ChannelOrder::Unordered {
            return Err(anyhow::anyhow!(
                "channel order must be unordered for ICS20 transfer"
            ));
        }

        if msg.channel.version != Version::ics20() {
            return Err(anyhow::anyhow!(
                "channel version must be ics20 for ICS20 transfer"
            ));
        }

        Ok(())
    }
    async fn chan_open_try_check(&self, _ctx: Context, msg: &MsgChannelOpenTry) -> Result<()> {
        if msg.channel.ordering != ChannelOrder::Unordered {
            return Err(anyhow::anyhow!(
                "channel order must be unordered for ICS20 transfer"
            ));
        }

        if msg.counterparty_version != Version::ics20() {
            return Err(anyhow::anyhow!(
                "counterparty version must be ics20-1 for ICS20 transfer"
            ));
        }

        Ok(())
    }
    async fn chan_open_ack_check(&self, _ctx: Context, msg: &MsgChannelOpenAck) -> Result<()> {
        if msg.counterparty_version != Version::ics20() {
            return Err(anyhow::anyhow!(
                "counterparty version must be ics20-1 for ICS20 transfer"
            ));
        }

        Ok(())
    }
    async fn chan_open_confirm_check(
        &self,
        _ctx: Context,
        _msg: &MsgChannelOpenConfirm,
    ) -> Result<()> {
        // accept channel confirmations, port has already been validated, version has already been validated
        Ok(())
    }
    async fn chan_close_confirm_check(
        &self,
        _ctx: Context,
        _msg: &MsgChannelCloseConfirm,
    ) -> Result<()> {
        // no action necessary
        Ok(())
    }
    async fn chan_close_init_check(&self, _ctx: Context, _msg: &MsgChannelCloseInit) -> Result<()> {
        // always abort transaction
        return Err(anyhow::anyhow!("ics20 always aborts on close init"));
    }
    async fn recv_packet_check(&self, _ctx: Context, msg: &MsgRecvPacket) -> Result<()> {
        // 1. parse a FungibleTokenPacketData from msg.packet.data
        let packet_data = FungibleTokenPacketData::decode(msg.packet.data.as_slice())?;

        // 2. check if we are the source chain for the denom. (check packet path to see if it is a penumbra path)
        let prefix = format!("{}/{}/", msg.packet.source_port, msg.packet.source_channel);
        let is_source = packet_data.denom.starts_with(&prefix);

        if is_source {
            // check if we have enough balance to unescrow tokens to receiver
            let value_balance: u64 = self
                .state
                .get_proto::<u64>(
                    format!("ics20-value-balance/{}", msg.packet.destination_channel).into(),
                )
                .await?
                .ok_or_else(|| anyhow::anyhow!("value balance not found"))?;

            // convert the amount to a u64 from u256.
            //  TODO: the amount is given by the ICS20 spec to be a u256, but we parse it to u64
            //  for now. should we round, or error, or something else in this conversion?
            let amount_penumbra = packet_data.amount.parse::<u64>()?;
            if value_balance < amount_penumbra {
                return Err(anyhow::anyhow!(
                    "insufficient balance to unescrow tokens to receiver"
                ));
            }
        }

        Ok(())
    }
    async fn timeout_packet_check(&self, _ctx: Context, msg: &MsgTimeout) -> Result<()> {
        let packet_data = FungibleTokenPacketData::decode(msg.packet.data.as_slice())?;

        let prefix = format!("{}/{}/", msg.packet.source_port, msg.packet.source_channel);
        let is_source = packet_data.denom.starts_with(&prefix);
        if is_source {
            // check if we have enough balance to refund tokens to sender
            let value_balance: u64 = self
                .state
                .get_proto::<u64>(
                    format!("ics20-value-balance/{}", msg.packet.destination_channel).into(),
                )
                .await?
                .ok_or_else(|| anyhow::anyhow!("value balance not found"))?;

            // convert the amount to a u64 from u256.
            //  TODO: the amount is given by the ICS20 spec to be a u256, but we parse it to u64
            //  for now. should we round, or error, or something else in this conversion?
            let amount_penumbra = packet_data.amount.parse::<u64>()?;
            if value_balance < amount_penumbra {
                return Err(anyhow::anyhow!(
                    "insufficient balance to refund tokens to sender"
                ));
            }
        }

        Ok(())
    }
    async fn acknowledge_packet_check(
        &self,
        _ctx: Context,
        _msg: &MsgAcknowledgement,
    ) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl AppHandlerExecute for ICS20Transfer {
    async fn chan_open_init_execute(&mut self, _ctx: Context, _msg: &MsgChannelOpenInit) {}
    async fn chan_open_try_execute(&mut self, _ctx: Context, _msg: &MsgChannelOpenTry) {}
    async fn chan_open_ack_execute(&mut self, _ctx: Context, msg: &MsgChannelOpenAck) {
        self.state
            .put_proto::<u64>(format!("ics20-value-balance/{}", msg.channel_id).into(), 0)
            .await;
    }
    async fn chan_open_confirm_execute(&mut self, _ctx: Context, msg: &MsgChannelOpenConfirm) {
        self.state
            .put_proto::<u64>(format!("ics20-value-balance/{}", msg.channel_id).into(), 0)
            .await;
    }
    async fn chan_close_confirm_execute(&mut self, _ctx: Context, _msg: &MsgChannelCloseConfirm) {}
    async fn chan_close_init_execute(&mut self, _ctx: Context, _msg: &MsgChannelCloseInit) {}
    async fn recv_packet_execute(&mut self, _ctx: Context, _msg: &MsgRecvPacket) {
        // parse if we are source or dest, and mint or burn accordingly
    }
    async fn timeout_packet_execute(&mut self, _ctx: Context, _msg: &MsgTimeout) {}
    async fn acknowledge_packet_execute(&mut self, _ctx: Context, _msg: &MsgAcknowledgement) {}
}

impl AppHandler for ICS20Transfer {}
