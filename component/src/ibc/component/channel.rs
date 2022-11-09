use std::sync::Arc;

use super::state_key;
use crate::ibc::event;
use crate::ibc::ibc_handler::AppHandlerCheck;
use crate::ibc::ibc_handler::AppHandlerExecute;
use crate::ibc::transfer::Ics20Transfer;
use crate::Component;
use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::client_consensus::AnyConsensusState;
use ibc::core::ics02_client::client_consensus::ConsensusState;
use ibc::core::ics02_client::client_def::AnyClient;
use ibc::core::ics02_client::client_def::ClientDef;
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
use penumbra_storage::StateTransaction;
use penumbra_storage::StateWrite;
use penumbra_storage::{State, StateRead};
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

mod execution;
mod stateful;
mod stateless;

use stateful::proof_verification::commit_packet;

pub struct Ics4Channel {}

#[async_trait]
impl Component for Ics4Channel {
    #[instrument(name = "ics4_channel", skip(_state, _app_state))]
    async fn init_chain(_state: &mut StateTransaction, _app_state: &genesis::AppState) {}

    #[instrument(name = "ics4_channel", skip(_state, _begin_block))]
    async fn begin_block(_state: &mut StateTransaction, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "ics4_channel", skip(tx))]
    fn check_tx_stateless(tx: Arc<Transaction>) -> Result<()> {
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

    #[instrument(name = "ics4_channel", skip(state, tx))]
    async fn check_tx_stateful(state: Arc<State>, tx: Arc<Transaction>) -> Result<()> {
        let transfer = PortId::transfer();
        for ibc_action in tx.ibc_actions() {
            let state = state.clone();
            match &ibc_action.action {
                Some(ChannelOpenInit(msg)) => {
                    use stateful::channel_open_init::ChannelOpenInitCheck;
                    let msg = MsgChannelOpenInit::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                    if msg.port_id == transfer {
                        Ics20Transfer::chan_open_init_check(state, &msg).await?
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(ChannelOpenTry(msg)) => {
                    use stateful::channel_open_try::ChannelOpenTryCheck;
                    let msg = MsgChannelOpenTry::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                    if msg.port_id == transfer {
                        Ics20Transfer::chan_open_try_check(state, &msg).await?
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(ChannelOpenAck(msg)) => {
                    use stateful::channel_open_ack::ChannelOpenAckCheck;
                    let msg = MsgChannelOpenAck::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                    if msg.port_id == transfer {
                        Ics20Transfer::chan_open_ack_check(state, &msg).await?
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(ChannelOpenConfirm(msg)) => {
                    use stateful::channel_open_confirm::ChannelOpenConfirmCheck;
                    let msg = MsgChannelOpenConfirm::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                    if msg.port_id == transfer {
                        Ics20Transfer::chan_open_confirm_check(state, &msg).await?
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(ChannelCloseInit(msg)) => {
                    use stateful::channel_close_init::ChannelCloseInitCheck;
                    let msg = MsgChannelCloseInit::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                    if msg.port_id == transfer {
                        Ics20Transfer::chan_close_init_check(state, &msg).await?
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(ChannelCloseConfirm(msg)) => {
                    use stateful::channel_close_confirm::ChannelCloseConfirmCheck;
                    let msg = MsgChannelCloseConfirm::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                    if msg.port_id == transfer {
                        Ics20Transfer::chan_close_confirm_check(state, &msg).await?
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(RecvPacket(msg)) => {
                    use stateful::recv_packet::RecvPacketCheck;
                    let msg = MsgRecvPacket::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                    if msg.packet.destination_port == transfer {
                        Ics20Transfer::recv_packet_check(state, &msg).await?
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(Acknowledgement(msg)) => {
                    use stateful::acknowledge_packet::AcknowledgePacketCheck;
                    let msg = MsgAcknowledgement::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                    if msg.packet.destination_port == transfer {
                        Ics20Transfer::acknowledge_packet_check(state, &msg).await?
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(Timeout(msg)) => {
                    use stateful::timeout::TimeoutCheck;
                    let msg = MsgTimeout::try_from(msg.clone())?;

                    state.validate(&msg).await?;
                    if msg.packet.destination_port == transfer {
                        Ics20Transfer::timeout_packet_check(state, &msg).await?
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }

                // Other IBC messages are not handled by this component.
                _ => {}
            }
        }
        Ok(())
    }

    #[instrument(name = "ics4_channel", skip(state, tx))]
    async fn execute_tx(state: &mut StateTransaction, tx: Arc<Transaction>) -> Result<()> {
        let transfer = PortId::transfer();
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(ChannelOpenInit(msg)) => {
                    use execution::channel_open_init::ChannelOpenInitExecute;
                    let msg = MsgChannelOpenInit::try_from(msg.clone()).unwrap();

                    state.execute(&msg).await;
                    if msg.port_id == transfer {
                        Ics20Transfer::chan_open_init_execute(state, &msg).await
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(ChannelOpenTry(msg)) => {
                    use execution::channel_open_try::ChannelOpenTryExecute;
                    let msg = MsgChannelOpenTry::try_from(msg.clone()).unwrap();

                    state.execute(&msg).await;
                    if msg.port_id == transfer {
                        Ics20Transfer::chan_open_try_execute(state, &msg).await
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(ChannelOpenAck(msg)) => {
                    use execution::channel_open_ack::ChannelOpenAckExecute;
                    let msg = MsgChannelOpenAck::try_from(msg.clone()).unwrap();

                    state.execute(&msg).await;
                    if msg.port_id == transfer {
                        Ics20Transfer::chan_open_ack_execute(state, &msg).await
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(ChannelOpenConfirm(msg)) => {
                    use execution::channel_open_confirm::ChannelOpenConfirmExecute;
                    let msg = MsgChannelOpenConfirm::try_from(msg.clone()).unwrap();

                    state.execute(&msg).await;
                    if msg.port_id == transfer {
                        Ics20Transfer::chan_open_confirm_execute(state, &msg).await
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(ChannelCloseInit(msg)) => {
                    use execution::channel_close_init::ChannelCloseInitExecute;
                    let msg = MsgChannelCloseInit::try_from(msg.clone()).unwrap();

                    state.execute(&msg).await;
                    if msg.port_id == transfer {
                        Ics20Transfer::chan_close_init_execute(state, &msg).await
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(ChannelCloseConfirm(msg)) => {
                    use execution::channel_close_confirm::ChannelCloseConfirmExecute;
                    let msg = MsgChannelCloseConfirm::try_from(msg.clone()).unwrap();

                    state.execute(&msg).await;
                    if msg.port_id == transfer {
                        Ics20Transfer::chan_close_confirm_execute(state, &msg).await
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(RecvPacket(msg)) => {
                    use execution::recv_packet::RecvPacketExecute;
                    let msg = MsgRecvPacket::try_from(msg.clone()).unwrap();

                    state.execute(&msg).await;
                    if msg.packet.destination_port == transfer {
                        Ics20Transfer::recv_packet_execute(state, &msg).await
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(Acknowledgement(msg)) => {
                    use execution::acknowledge_packet::AcknowledgePacketExecute;
                    let msg = MsgAcknowledgement::try_from(msg.clone()).unwrap();

                    state.execute(&msg).await;
                    if msg.packet.destination_port == transfer {
                        Ics20Transfer::acknowledge_packet_execute(state, &msg).await
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }
                Some(Timeout(msg)) => {
                    use execution::timeout::TimeoutExecute;
                    let msg = MsgTimeout::try_from(msg.clone()).unwrap();

                    state.execute(&msg).await;
                    if msg.packet.destination_port == transfer {
                        Ics20Transfer::timeout_packet_execute(state, &msg).await
                    } else {
                        return Err(anyhow::anyhow!("invalid port id"));
                    }
                }

                // Other IBC messages are not handled by this component.
                _ => {}
            }
        }

        Ok(())
    }

    #[instrument(name = "ics4_channel", skip(_state, _end_block))]
    async fn end_block(_state: &mut StateTransaction, _end_block: &abci::request::EndBlock) {}
}

#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    fn put_channel_counter(&mut self, counter: u64) {
        self.put_proto::<u64>("ibc_channel_counter".into(), counter);
    }

    async fn next_channel_id(&mut self) -> Result<ChannelId> {
        let ctr = self.get_channel_counter().await?;
        self.put_channel_counter(ctr + 1);

        Ok(ChannelId::new(ctr))
    }

    fn put_channel(&mut self, channel_id: &ChannelId, port_id: &PortId, channel: ChannelEnd) {
        self.put(state_key::channel(channel_id, port_id), channel);
    }

    fn put_ack_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(state_key::seq_ack(channel_id, port_id), sequence);
    }

    fn put_recv_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(state_key::seq_recv(channel_id, port_id), sequence);
    }

    fn put_send_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(state_key::seq_send(channel_id, port_id), sequence);
    }

    fn put_packet_receipt(&mut self, packet: &Packet) {
        self.put_proto::<String>(state_key::packet_receipt(packet), "1".to_string());
    }

    fn put_packet_commitment(&mut self, packet: &Packet) {
        let commitment_key = state_key::packet_commitment(packet);
        let packet_hash = commit_packet(packet);

        self.put_proto::<Vec<u8>>(commitment_key, packet_hash);
    }

    fn delete_packet_commitment(
        &mut self,
        channel_id: &ChannelId,
        port_id: &PortId,
        sequence: u64,
    ) {
        self.put_proto::<Vec<u8>>(
            state_key::packet_commitment_by_port(port_id, channel_id, sequence),
            vec![],
        );
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn get_channel_counter(&self) -> Result<u64> {
        self.get_proto::<u64>("ibc_channel_counter")
            .await
            .map(|counter| counter.unwrap_or(0))
    }

    async fn get_channel(
        &self,
        channel_id: &ChannelId,
        port_id: &PortId,
    ) -> Result<Option<ChannelEnd>> {
        self.get(&state_key::channel(channel_id, port_id)).await
    }

    async fn get_recv_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        self.get_proto::<u64>(&state_key::seq_recv(channel_id, port_id))
            .await
            .map(|sequence| sequence.unwrap_or(0))
    }

    async fn get_ack_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        self.get_proto::<u64>(&state_key::seq_ack(channel_id, port_id))
            .await
            .map(|sequence| sequence.unwrap_or(0))
    }

    async fn get_send_sequence(&self, channel_id: &ChannelId, port_id: &PortId) -> Result<u64> {
        self.get_proto::<u64>(&state_key::seq_send(channel_id, port_id))
            .await
            .map(|sequence| sequence.unwrap_or(0))
    }

    async fn seen_packet(&self, packet: &Packet) -> Result<bool> {
        self.get_proto::<String>(&state_key::packet_receipt(packet))
            .await
            .map(|res| res.is_some())
    }

    async fn get_packet_commitment(&self, packet: &Packet) -> Result<Option<Vec<u8>>> {
        let commitment = self
            .get_proto::<Vec<u8>>(&state_key::packet_commitment(packet))
            .await?;

        // this is for the special case where the commitment is empty, we consider this None.
        if let Some(commitment) = commitment.as_ref() {
            if commitment.is_empty() {
                return Ok(None);
            }
        }

        Ok(commitment)
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
