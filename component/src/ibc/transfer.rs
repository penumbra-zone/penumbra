use crate::ibc::component::state_key;
use crate::ibc::ibc_handler::{AppHandler, AppHandlerCheck, AppHandlerExecute};
use crate::ibc::packet::{IBCPacket, Unchecked};
use crate::Component;
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
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{State, StateRead, StateTransaction, StateWrite};
use penumbra_transaction::action::Ics20Withdrawal;
use prost::Message;
use std::sync::Arc;
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
pub struct Ics20Transfer {}

#[async_trait]
pub trait Ics20TransferReadExt: StateRead {
    async fn withdrawal_check(state: Arc<State>, withdrawal: &Ics20Withdrawal) -> Result<()> {
        // create packet
        let packet: IBCPacket<Unchecked> = withdrawal.clone().into();

        // send packet
        use crate::ibc::packet::SendPacketRead as _;
        state.send_packet_check(packet).await?;

        Ok(())
    }
}

impl<T: StateRead> Ics20TransferReadExt for T {}

#[async_trait]
pub trait Ics20TransferWriteExt: StateWrite {
    async fn withdrawal_execute(state: &mut StateTransaction<'_>, withdrawal: &Ics20Withdrawal) {
        // create packet, assume it's already checked since the component caller contract calls `check` before `execute`
        let checked_packet = IBCPacket::<Unchecked>::from(withdrawal.clone()).assume_checked();

        if is_source(
            &withdrawal.source_port,
            &withdrawal.source_channel,
            &withdrawal.denom,
        ) {
            // we are the source. add the value balance to the escrow channel.
            let existing_value_balance: Amount = state
                .get(&state_key::ics20_value_balance(
                    &withdrawal.source_channel,
                    &withdrawal.denom.id(),
                ))
                .await
                .unwrap()
                .unwrap_or_else(Amount::zero);

            let new_value_balance = existing_value_balance + withdrawal.amount;
            state.put(
                state_key::ics20_value_balance(&withdrawal.source_channel, &withdrawal.denom.id()),
                new_value_balance,
            );
        } else {
            // receiver is the source, burn utxos

            // NOTE: this burning should already be accomplished by the value balance check from
            // the withdrawal's balance commitment, so nothing to do here.
        }

        use crate::ibc::packet::SendPacketWrite;
        state.send_packet_execute(checked_packet).await;
    }
}

impl<T: StateWrite> Ics20TransferWriteExt for T {}

// TODO: Ics20 implementation.
// see: https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer
// TODO (ava): add versioning to AppHandlers
#[async_trait]
impl AppHandlerCheck for Ics20Transfer {
    async fn chan_open_init_check(_state: Arc<State>, msg: &MsgChannelOpenInit) -> Result<()> {
        if msg.channel.ordering != ChannelOrder::Unordered {
            return Err(anyhow::anyhow!(
                "channel order must be unordered for Ics20 transfer"
            ));
        }

        if msg.channel.version != Version::ics20() {
            return Err(anyhow::anyhow!(
                "channel version must be ics20 for Ics20 transfer"
            ));
        }

        Ok(())
    }

    async fn chan_open_try_check(_state: Arc<State>, msg: &MsgChannelOpenTry) -> Result<()> {
        if msg.channel.ordering != ChannelOrder::Unordered {
            return Err(anyhow::anyhow!(
                "channel order must be unordered for Ics20 transfer"
            ));
        }

        if msg.counterparty_version != Version::ics20() {
            return Err(anyhow::anyhow!(
                "counterparty version must be ics20-1 for Ics20 transfer"
            ));
        }

        Ok(())
    }

    async fn chan_open_ack_check(_state: Arc<State>, msg: &MsgChannelOpenAck) -> Result<()> {
        if msg.counterparty_version != Version::ics20() {
            return Err(anyhow::anyhow!(
                "counterparty version must be ics20-1 for Ics20 transfer"
            ));
        }

        Ok(())
    }

    async fn chan_open_confirm_check(
        _state: Arc<State>,
        _msg: &MsgChannelOpenConfirm,
    ) -> Result<()> {
        // accept channel confirmations, port has already been validated, version has already been validated
        Ok(())
    }

    async fn chan_close_confirm_check(
        _state: Arc<State>,
        _msg: &MsgChannelCloseConfirm,
    ) -> Result<()> {
        // no action necessary
        Ok(())
    }

    async fn chan_close_init_check(_state: Arc<State>, _msg: &MsgChannelCloseInit) -> Result<()> {
        // always abort transaction
        return Err(anyhow::anyhow!("ics20 always aborts on close init"));
    }

    async fn recv_packet_check(state: Arc<State>, msg: &MsgRecvPacket) -> Result<()> {
        // 1. parse a FungibleTokenPacketData from msg.packet.data
        let packet_data = FungibleTokenPacketData::decode(msg.packet.data.as_slice())?;
        let denom: asset::Denom = packet_data.denom.as_str().try_into()?;

        // 2. check if we are the source chain for the denom.
        if is_source(&msg.packet.source_port, &msg.packet.source_channel, &denom) {
            // check if we have enough balance to unescrow tokens to receiver
            let value_balance: Amount = state
                .get(&state_key::ics20_value_balance(
                    &msg.packet.source_channel,
                    &denom.id(),
                ))
                .await?
                .unwrap_or_else(Amount::zero);

            let amount_penumbra: Amount = packet_data.amount.try_into()?;
            if value_balance < amount_penumbra {
                return Err(anyhow::anyhow!(
                    "insufficient balance to unescrow tokens to receiver"
                ));
            }
        }

        Ok(())
    }

    async fn timeout_packet_check(state: Arc<State>, msg: &MsgTimeout) -> Result<()> {
        let packet_data = FungibleTokenPacketData::decode(msg.packet.data.as_slice())?;
        let denom: asset::Denom = packet_data.denom.as_str().try_into()?;

        if is_source(&msg.packet.source_port, &msg.packet.source_channel, &denom) {
            // check if we have enough balance to refund tokens to sender
            let value_balance: Amount = state
                .get(&state_key::ics20_value_balance(
                    &msg.packet.source_channel,
                    &denom.id(),
                ))
                .await?
                .unwrap_or_else(Amount::zero);

            let amount_penumbra: Amount = packet_data.amount.try_into()?;
            if value_balance < amount_penumbra {
                return Err(anyhow::anyhow!(
                    "insufficient balance to refund tokens to sender"
                ));
            }
        }

        Ok(())
    }

    async fn acknowledge_packet_check(_state: Arc<State>, _msg: &MsgAcknowledgement) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl AppHandlerExecute for Ics20Transfer {
    async fn chan_open_init_execute(_state: &mut StateTransaction, _msg: &MsgChannelOpenInit) {}
    async fn chan_open_try_execute(_state: &mut StateTransaction, _msg: &MsgChannelOpenTry) {}
    async fn chan_open_ack_execute(_state: &mut StateTransaction, _msg: &MsgChannelOpenAck) {}
    async fn chan_open_confirm_execute(
        _state: &mut StateTransaction,
        _msg: &MsgChannelOpenConfirm,
    ) {
    }
    async fn chan_close_confirm_execute(
        _state: &mut StateTransaction,
        _msg: &MsgChannelCloseConfirm,
    ) {
    }
    async fn chan_close_init_execute(_state: &mut StateTransaction, _msg: &MsgChannelCloseInit) {}
    async fn recv_packet_execute(_state: &mut StateTransaction, _msg: &MsgRecvPacket) {
        // parse if we are source or dest, and mint or burn accordingly
    }
    async fn timeout_packet_execute(_state: &mut StateTransaction, _msg: &MsgTimeout) {}
    async fn acknowledge_packet_execute(_state: &mut StateTransaction, _msg: &MsgAcknowledgement) {}
}

impl AppHandler for Ics20Transfer {}

#[async_trait]
impl Component for Ics20Transfer {
    #[instrument(name = "ics20_transfer", skip(_state, _app_state))]
    async fn init_chain(_state: &mut StateTransaction, _app_state: &genesis::AppState) {}

    #[instrument(name = "ics20_transfer", skip(_state, _begin_block))]
    async fn begin_block(_state: &mut StateTransaction, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "ics20_channel", skip(_state, _end_block))]
    async fn end_block(_state: &mut StateTransaction, _end_block: &abci::request::EndBlock) {}
}
