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
use penumbra_crypto::Value;
use penumbra_storage::State;
use tracing::instrument;

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
        // 2. check of we are the source chain for the denom. (check packet path to see if it is a penumbra path)
        // 3. implement logic to check value balance if we are the source chain for the denom.

        /*
        let value_balance = self
            .state
            .get_domain::<Value>(format!(
                "ics20-value-balance/{}",
                msg.packet.destination_channel
            ))
            .await?;
        */

        /*
                from ICS-20 spec
                function onRecvPacket(packet: Packet) {
          FungibleTokenPacketData data = packet.data
          // construct default acknowledgement of success
          FungibleTokenPacketAcknowledgement ack = FungibleTokenPacketAcknowledgement{true, null}
          prefix = "{packet.sourcePort}/{packet.sourceChannel}/"
          // we are the source if the packets were prefixed by the sending chain
          source = data.denom.slice(0, len(prefix)) === prefix
          if source {
            // receiver is source chain: unescrow tokens
            // determine escrow account
            escrowAccount = channelEscrowAddresses[packet.destChannel]
            // unescrow tokens to receiver (assumed to fail if balance insufficient)
            err = bank.TransferCoins(escrowAccount, data.receiver, data.denom.slice(len(prefix)), data.amount)
            if (err !== nil)
              ack = FungibleTokenPacketAcknowledgement{false, "transfer coins failed"}
          } else {
            prefix = "{packet.destPort}/{packet.destChannel}/"
            prefixedDenomination = prefix + data.denom
            // sender was source, mint vouchers to receiver (assumed to fail if balance insufficient)
            err = bank.MintCoins(data.receiver, prefixedDenomination, data.amount)
            if (err !== nil)
              ack = FungibleTokenPacketAcknowledgement{false, "mint coins failed"}
          }
          return ack
        }
                 */
        Ok(())
    }
    async fn timeout_packet_check(&self, _ctx: Context, _msg: &MsgTimeout) -> Result<()> {
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
    async fn chan_open_init_execute(&mut self, _ctx: Context, _msg: &MsgChannelOpenInit) {
        /*
        self
        .state
        .put_domain::<Value>(format!("ics20-value-balance/{}", msg.channel_id), 0)
        .await?
        */
    }
    async fn chan_open_try_execute(&mut self, _ctx: Context, _msg: &MsgChannelOpenTry) {
        /*
        self
        .state
        .put_domain::<Value>(format!("ics20-value-balance/{}", msg.channel_id), 0)
        .await?
        */
    }
    async fn chan_open_ack_execute(&mut self, _ctx: Context, _msg: &MsgChannelOpenAck) {}
    async fn chan_open_confirm_execute(&mut self, _ctx: Context, _msg: &MsgChannelOpenConfirm) {}
    async fn chan_close_confirm_execute(&mut self, _ctx: Context, _msg: &MsgChannelCloseConfirm) {}
    async fn chan_close_init_execute(&mut self, _ctx: Context, _msg: &MsgChannelCloseInit) {}
    async fn recv_packet_execute(&mut self, _ctx: Context, _msg: &MsgRecvPacket) {
        // parse if we are source or dest, and mint or burn accordingly
    }
    async fn timeout_packet_execute(&mut self, _ctx: Context, _msg: &MsgTimeout) {}
    async fn acknowledge_packet_execute(&mut self, _ctx: Context, _msg: &MsgAcknowledgement) {}
}

impl AppHandler for ICS20Transfer {}
