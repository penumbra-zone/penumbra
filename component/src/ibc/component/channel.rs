use super::state_key;
use crate::ibc::component::client::View as _;
use crate::ibc::component::connection::View as _;
use crate::ibc::event;
use crate::ibc::ibc_handler::AppHandler;
use crate::{Component, Context};
use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::client_consensus::AnyConsensusState;
use ibc::core::ics02_client::client_consensus::ConsensusState;
use ibc::core::ics02_client::client_def::AnyClient;
use ibc::core::ics02_client::client_def::ClientDef;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics03_connection::connection::{ConnectionEnd, State as ConnectionState};
use ibc::core::ics04_channel::channel::Order as ChannelOrder;
use ibc::core::ics04_channel::channel::State as ChannelState;
use ibc::core::ics04_channel::channel::{ChannelEnd, Counterparty};
use ibc::core::ics04_channel::msgs::acknowledgement::MsgAcknowledgement;
use ibc::core::ics04_channel::msgs::chan_close_confirm::MsgChannelCloseConfirm;
use ibc::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
use ibc::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
use ibc::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
use ibc::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use ibc::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
use ibc::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use ibc::core::ics04_channel::msgs::timeout::MsgTimeout;
use ibc::core::ics04_channel::packet::Packet;
use ibc::core::ics24_host::identifier::ChannelId;
use ibc::core::ics24_host::identifier::PortId;
use penumbra_chain::genesis;
use penumbra_proto::core::ibc::v1alpha1::ibc_action::Action::{
    Acknowledgement, ChannelCloseConfirm, ChannelCloseInit, ChannelOpenAck, ChannelOpenConfirm,
    ChannelOpenInit, ChannelOpenTry, RecvPacket, Timeout,
};
use penumbra_storage2::State;
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

mod execution;
mod stateful;
mod stateless;

use stateful::proof_verification::commit_packet;

pub struct ICS4Channel {
    state: State,

    app_handler: Box<dyn AppHandler>,
}

impl ICS4Channel {
    #[instrument(name = "ics4_channel", skip(state, app_handler))]
    pub async fn new(state: State, app_handler: Box<dyn AppHandler>) -> Self {
        Self { state, app_handler }
    }
}

#[async_trait]
impl Component for ICS4Channel {
    #[instrument(name = "ics4_channel", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {}

    #[instrument(name = "ics4_channel", skip(self, _ctx, _begin_block))]
    async fn begin_block(&mut self, _ctx: Context, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "ics4_channel", skip(_ctx, tx))]
    fn check_tx_stateless(_ctx: Context, tx: &Transaction) -> Result<()> {
        // Each stateless check is a distinct function in an appropriate submodule,
        // so that we can easily add new stateless checks and see a birds' eye view
        // of all of the checks we're performing.

        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(ChannelOpenInit(msg)) => {
                    use stateless::channel_open_init::*;
                    let msg = MsgChannelOpenInit::try_from(msg.clone())?;

                    connection_hops_eq_1(&msg)?;
                }
                Some(ChannelOpenTry(msg)) => {
                    use stateless::channel_open_try::*;
                    let msg = MsgChannelOpenTry::try_from(msg.clone())?;

                    connection_hops_eq_1(&msg)?;
                }
                Some(ChannelOpenAck(msg)) => {
                    MsgChannelOpenAck::try_from(msg.clone())?;
                    // NOTE: no additional stateless validation is possible
                }
                Some(ChannelOpenConfirm(msg)) => {
                    MsgChannelOpenConfirm::try_from(msg.clone())?;
                    // NOTE: no additional stateless validation is possible
                }
                Some(ChannelCloseInit(msg)) => {
                    MsgChannelCloseInit::try_from(msg.clone())?;
                    // NOTE: no additional stateless validation is possible
                }
                Some(ChannelCloseConfirm(msg)) => {
                    MsgChannelCloseConfirm::try_from(msg.clone())?;
                    // NOTE: no additional stateless validation is possible
                }
                Some(RecvPacket(msg)) => {
                    MsgRecvPacket::try_from(msg.clone())?;

                    // NOTE: no additional stateless validation is possible
                }
                Some(Acknowledgement(msg)) => {
                    MsgAcknowledgement::try_from(msg.clone())?;
                    // NOTE: no additional stateless validation is possible
                }
                Some(Timeout(msg)) => {
                    MsgTimeout::try_from(msg.clone())?;
                    // NOTE: no additional stateless validation is possible
                }

                // Other IBC messages are not handled by this component.
                _ => {}
            }
        }

        Ok(())
    }

    #[instrument(name = "ics4_channel", skip(self, ctx, tx))]
    async fn check_tx_stateful(&self, ctx: Context, tx: &Transaction) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(ChannelOpenInit(msg)) => {
                    use stateful::channel_open_init::ChannelOpenInitCheck;
                    let msg = MsgChannelOpenInit::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                    self.app_handler
                        .chan_open_init_check(ctx.clone(), &msg)
                        .await?;
                }
                Some(ChannelOpenTry(msg)) => {
                    use stateful::channel_open_try::ChannelOpenTryCheck;
                    let msg = MsgChannelOpenTry::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                    self.app_handler
                        .chan_open_try_check(ctx.clone(), &msg)
                        .await?;
                }
                Some(ChannelOpenAck(msg)) => {
                    use stateful::channel_open_ack::ChannelOpenAckCheck;
                    let msg = MsgChannelOpenAck::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                    self.app_handler
                        .chan_open_ack_check(ctx.clone(), &msg)
                        .await?;
                }
                Some(ChannelOpenConfirm(msg)) => {
                    use stateful::channel_open_confirm::ChannelOpenConfirmCheck;
                    let msg = MsgChannelOpenConfirm::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                    self.app_handler
                        .chan_open_confirm_check(ctx.clone(), &msg)
                        .await?;
                }
                Some(ChannelCloseInit(msg)) => {
                    use stateful::channel_close_init::ChannelCloseInitCheck;
                    let msg = MsgChannelCloseInit::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                    self.app_handler
                        .chan_close_init_check(ctx.clone(), &msg)
                        .await?;
                }
                Some(ChannelCloseConfirm(msg)) => {
                    use stateful::channel_close_confirm::ChannelCloseConfirmCheck;
                    let msg = MsgChannelCloseConfirm::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                    self.app_handler
                        .chan_close_confirm_check(ctx.clone(), &msg)
                        .await?;
                }
                Some(RecvPacket(msg)) => {
                    use stateful::recv_packet::RecvPacketCheck;
                    let msg = MsgRecvPacket::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                    self.app_handler
                        .recv_packet_check(ctx.clone(), &msg)
                        .await?;
                }
                Some(Acknowledgement(msg)) => {
                    use stateful::acknowledge_packet::AcknowledgePacketCheck;
                    let msg = MsgAcknowledgement::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                    self.app_handler
                        .acknowledge_packet_check(ctx.clone(), &msg)
                        .await?;
                }
                Some(Timeout(msg)) => {
                    use stateful::timeout::TimeoutCheck;
                    let msg = MsgTimeout::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                    self.app_handler
                        .timeout_packet_check(ctx.clone(), &msg)
                        .await?;
                }

                // Other IBC messages are not handled by this component.
                _ => {}
            }
        }
        Ok(())
    }

    #[instrument(name = "ics4_channel", skip(self, ctx, tx))]
    async fn execute_tx(&mut self, ctx: Context, tx: &Transaction) {
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(ChannelOpenInit(msg)) => {
                    use execution::channel_open_init::ChannelOpenInitExecute;
                    let msg = MsgChannelOpenInit::try_from(msg.clone()).unwrap();

                    self.state.execute(ctx.clone(), &msg).await;
                    self.app_handler
                        .chan_open_init_execute(ctx.clone(), &msg)
                        .await;
                }
                Some(ChannelOpenTry(msg)) => {
                    use execution::channel_open_try::ChannelOpenTryExecute;
                    let msg = MsgChannelOpenTry::try_from(msg.clone()).unwrap();

                    self.state.execute(ctx.clone(), &msg).await;
                    self.app_handler
                        .chan_open_try_execute(ctx.clone(), &msg)
                        .await;
                }
                Some(ChannelOpenAck(msg)) => {
                    use execution::channel_open_ack::ChannelOpenAckExecute;
                    let msg = MsgChannelOpenAck::try_from(msg.clone()).unwrap();

                    self.state.execute(ctx.clone(), &msg).await;
                    self.app_handler
                        .chan_open_ack_execute(ctx.clone(), &msg)
                        .await;
                }
                Some(ChannelOpenConfirm(msg)) => {
                    use execution::channel_open_confirm::ChannelOpenConfirmExecute;
                    let msg = MsgChannelOpenConfirm::try_from(msg.clone()).unwrap();

                    self.state.execute(ctx.clone(), &msg).await;
                    self.app_handler
                        .chan_open_confirm_execute(ctx.clone(), &msg)
                        .await;
                }
                Some(ChannelCloseInit(msg)) => {
                    use execution::channel_close_init::ChannelCloseInitExecute;
                    let msg = MsgChannelCloseInit::try_from(msg.clone()).unwrap();

                    self.state.execute(ctx.clone(), &msg).await;
                    self.app_handler
                        .chan_close_init_execute(ctx.clone(), &msg)
                        .await;
                }
                Some(ChannelCloseConfirm(msg)) => {
                    use execution::channel_close_confirm::ChannelCloseConfirmExecute;
                    let msg = MsgChannelCloseConfirm::try_from(msg.clone()).unwrap();

                    self.state.execute(ctx.clone(), &msg).await;
                    self.app_handler
                        .chan_close_confirm_execute(ctx.clone(), &msg)
                        .await;
                }
                Some(RecvPacket(msg)) => {
                    use execution::recv_packet::RecvPacketExecute;
                    let msg = MsgRecvPacket::try_from(msg.clone()).unwrap();

                    self.state.execute(ctx.clone(), &msg).await;
                    self.app_handler
                        .recv_packet_execute(ctx.clone(), &msg)
                        .await;
                }
                Some(Acknowledgement(msg)) => {
                    use execution::acknowledge_packet::AcknowledgePacketExecute;
                    let msg = MsgAcknowledgement::try_from(msg.clone()).unwrap();

                    self.state.execute(ctx.clone(), &msg).await;
                    self.app_handler
                        .acknowledge_packet_execute(ctx.clone(), &msg)
                        .await;
                }
                Some(Timeout(msg)) => {
                    use execution::timeout::TimeoutExecute;
                    let msg = MsgTimeout::try_from(msg.clone()).unwrap();

                    self.state.execute(ctx.clone(), &msg).await;
                    self.app_handler
                        .timeout_packet_execute(ctx.clone(), &msg)
                        .await;
                }

                // Other IBC messages are not handled by this component.
                _ => {}
            }
        }
    }

    #[instrument(name = "ics4_channel", skip(self, _ctx, _end_block))]
    async fn end_block(&mut self, _ctx: Context, _end_block: &abci::request::EndBlock) {}
}

#[async_trait]
pub trait View {
    async fn get_channel_counter(&self) -> Result<u64> {
        self.get_proto::<u64>("ibc_channel_counter".into())
            .await
            .map(|counter| counter.unwrap_or(0))
    }
    async fn put_channel_counter(&self, counter: u64) {
        self.put_proto::<u64>("ibc_channel_counter".into(), counter)
            .await;
    }
    async fn next_channel_id(&mut self) -> Result<ChannelId> {
        let ctr = self.get_channel_counter().await?;
        self.put_channel_counter(ctr + 1).await;

        Ok(ChannelId::new(ctr))
    }
    async fn get_channel(
        &self,
        channel_id: &ChannelId,
        port_id: &PortId,
    ) -> Result<Option<ChannelEnd>> {
        self.get_domain(state_key::channel(channel_id, port_id).into())
            .await
    }
    async fn put_channel(&mut self, channel_id: &ChannelId, port_id: &PortId, channel: ChannelEnd) {
        self.put_domain(state_key::channel(channel_id, port_id).into(), channel)
            .await;
    }
    async fn get_recv_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        self.get_proto::<u64>(state_key::seq_recv(channel_id, port_id).into())
            .await
            .map(|sequence| sequence.unwrap_or(0))
    }
    async fn get_ack_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        self.get_proto::<u64>(state_key::seq_ack(channel_id, port_id).into())
            .await
            .map(|sequence| sequence.unwrap_or(0))
    }
    async fn get_send_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        self.get_proto::<u64>(state_key::seq_send(channel_id, port_id).into())
            .await
            .map(|sequence| sequence.unwrap_or(0))
    }
    async fn put_ack_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(state_key::seq_ack(channel_id, port_id).into(), sequence)
            .await;
    }
    async fn put_recv_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(state_key::seq_recv(channel_id, port_id).into(), sequence)
            .await;
    }
    async fn put_send_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(state_key::seq_send(channel_id, port_id).into(), sequence)
            .await;
    }
    async fn put_packet_receipt(&mut self, packet: &Packet) {
        self.put_proto::<String>(state_key::packet_receipt(packet).into(), "1".to_string())
            .await;
    }
    async fn seen_packet(&self, packet: &Packet) -> Result<bool> {
        self.get_proto::<String>(state_key::packet_receipt(packet).into())
            .await
            .map(|res| res.is_some())
    }
    async fn put_packet_commitment(&self, packet: &Packet) {
        let commitment_key = state_key::packet_commitment(packet);
        let packet_hash = commit_packet(packet);

        self.put_proto::<Vec<u8>>(commitment_key.into(), packet_hash)
            .await;
    }
    async fn get_packet_commitment(&self, packet: &Packet) -> Result<Option<Vec<u8>>> {
        let commitment = self
            .get_proto::<Vec<u8>>(state_key::packet_commitment(packet).into())
            .await?;

        // this is for the special case where the commitment is empty, we consider this None.
        if let Some(commitment) = commitment.as_ref() {
            if commitment.is_empty() {
                return Ok(None);
            }
        }

        Ok(commitment)
    }
    async fn delete_packet_commitment(
        &mut self,
        channel_id: &ChannelId,
        port_id: &PortId,
        sequence: u64,
    ) {
        self.put_proto::<Vec<u8>>(
            state_key::packet_commitment_by_port(port_id, channel_id, sequence).into(),
            vec![],
        )
        .await;
    }
}
