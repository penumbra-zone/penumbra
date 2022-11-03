use crate::ibc::component::state_key;
use crate::ibc::ibc_handler::{AppHandler, AppHandlerCheck, AppHandlerExecute};
use crate::ibc::packet::{IBCPacket, Unchecked};
use crate::{Component, Context};
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
use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use penumbra_chain::genesis;
use penumbra_crypto::asset::Denom;
use penumbra_crypto::{asset, Amount};
use penumbra_proto::core::ibc::v1alpha1::FungibleTokenPacketData;
use penumbra_storage2::State;
use penumbra_transaction::action::ICS20Withdrawal;
use penumbra_transaction::{Action, Transaction};
use prost::Message;
use tendermint::abci;
use tracing::instrument;

// returns a bool indicating if the provided denom was issued locally or if it was bridged in.
// this logic is a bit tricky, and adapted from https://github.com/cosmos/ibc/tree/main/spec/app/ics-020-fungible-token-transfer (sendFungibleTokens).
//
// what we want to do is to determine if the denom being withdrawn is a native token (one
// that originates from Penumbra) or a bridged token (one that was sent into penumbra from
// IBC).
//
// A simple way of doing this is by parsing the denom, looking for a prefix that is only
// appended in the case of a bridged token. That is what this logic does.
fn is_source(source_port: &PortId, source_channel: &ChannelId, denom: &Denom) -> bool {
    let prefix = format!("{}/{}/", source_port, source_channel);

    !denom.starts_with(&prefix)
}

#[derive(Clone)]
pub struct ICS20Transfer {}

impl ICS20Transfer {
    #[instrument(name = "ics20_transfer")]
    pub fn new() -> Self {
        Self {}
    }

    pub async fn withdrawal_check(&self, ctx: Context, withdrawal: &ICS20Withdrawal) -> Result<()> {
        // create packet
        let packet: IBCPacket<Unchecked> = withdrawal.clone().into();

        // send packet
        use crate::ibc::packet::SendPacket;
        self.state.send_packet_check(ctx.clone(), packet).await?;

        Ok(())
    }

    pub async fn withdrawal_execute(&mut self, ctx: Context, withdrawal: &ICS20Withdrawal) {
        // create packet, assume it's already checked since the component caller contract calls `check` before `execute`
        let checked_packet = IBCPacket::<Unchecked>::from(withdrawal.clone()).assume_checked();

        if is_source(
            &withdrawal.source_port,
            &withdrawal.source_channel,
            &withdrawal.denom,
        ) {
            // we are the source. add the value balance to the escrow channel.
            let existing_value_balance: Amount = self
                .state
                .get_domain(
                    state_key::ics20_value_balance(
                        &withdrawal.source_channel,
                        &withdrawal.denom.id(),
                    )
                    .into(),
                )
                .await
                .unwrap()
                .unwrap_or(Amount::zero());

            let new_value_balance = existing_value_balance + withdrawal.amount;
            self.state
                .put_domain(
                    state_key::ics20_value_balance(
                        &withdrawal.source_channel,
                        &withdrawal.denom.id().into(),
                    )
                    .into(),
                    new_value_balance,
                )
                .await;
        } else {
            // receiver is the source, burn utxos

            // NOTE: this burning should already be accomplished by the value balance check from
            // the withdrawal's balance commitment, so nothing to do here.
        }

        use crate::ibc::packet::SendPacket;
        self.state
            .send_packet_execute(ctx.clone(), checked_packet)
            .await;
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
        let denom: asset::Denom = packet_data.denom.as_str().try_into()?;

        // 2. check if we are the source chain for the denom.
        if is_source(&msg.packet.source_port, &msg.packet.source_channel, &denom) {
            // check if we have enough balance to unescrow tokens to receiver
            let value_balance: Amount = self
                .state
                .get_domain(
                    state_key::ics20_value_balance(&msg.packet.source_channel, &denom.id()).into(),
                )
                .await?
                .unwrap_or(Amount::zero());

            let amount_penumbra: Amount = packet_data.amount.try_into()?;
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
        let denom: asset::Denom = packet_data.denom.as_str().try_into()?;

        if is_source(&msg.packet.source_port, &msg.packet.source_channel, &denom) {
            // check if we have enough balance to refund tokens to sender
            let value_balance: Amount = self
                .state
                .get_domain(
                    state_key::ics20_value_balance(&msg.packet.source_channel, &denom.id()).into(),
                )
                .await?
                .unwrap_or(Amount::zero());

            let amount_penumbra: Amount = packet_data.amount.try_into()?;
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

#[async_trait]
impl Component for ICS20Transfer {
    #[instrument(name = "ics20_transfer", skip(self, _app_state))]
    async fn init_chain(state: &mut StateTransaction, _app_state: &genesis::AppState) {}

    #[instrument(name = "ics20_transfer", skip(self, _ctx, _begin_block))]
    async fn begin_block(
        state: &mut StateTransaction,
        _ctx: Context,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "ics20_transfer", skip(_ctx, tx))]
    fn check_tx_stateless(_ctx: Context, tx: &Transaction) -> Result<()> {
        for action in tx.actions() {
            match action {
                Action::ICS20Withdrawal(withdrawal) => {
                    withdrawal.validate()?;
                }

                _ => {}
            }
        }
        Ok(())
    }
    #[instrument(name = "ics20_transfer", skip(self, ctx, tx))]
    async fn check_tx_stateful(ctx: Context, tx: &Transaction) -> Result<()> {
        for action in tx.actions() {
            match action {
                Action::ICS20Withdrawal(withdrawal) => {
                    self.withdrawal_check(ctx.clone(), withdrawal).await?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    #[instrument(name = "ics20_transfer", skip(self, ctx, tx))]
    async fn execute_tx(ctx: Context, tx: &Transaction) {
        for action in tx.actions() {
            match action {
                Action::ICS20Withdrawal(withdrawal) => {
                    self.withdrawal_execute(ctx.clone(), withdrawal).await;
                }
                _ => {}
            }
        }
    }

    #[instrument(name = "ics20_channel", skip(self, _ctx, _end_block))]
    async fn end_block(_ctx: Context, _end_block: &abci::request::EndBlock) {}
}
