use crate::component::client::View as _;
use crate::component::connection::View as _;
use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::client_consensus::ConsensusState;
use ibc::core::ics02_client::client_def::AnyClient;
use ibc::core::ics02_client::client_def::ClientDef;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics03_connection::connection::{ConnectionEnd, State as ConnectionState};
use ibc::core::ics04_channel::channel::State as ChannelState;
use ibc::core::ics04_channel::channel::{ChannelEnd, Counterparty};
use ibc::core::ics04_channel::msgs::chan_close_confirm::MsgChannelCloseConfirm;
use ibc::core::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;
use ibc::core::ics04_channel::msgs::chan_open_ack::MsgChannelOpenAck;
use ibc::core::ics04_channel::msgs::chan_open_confirm::MsgChannelOpenConfirm;
use ibc::core::ics04_channel::msgs::chan_open_init::MsgChannelOpenInit;
use ibc::core::ics04_channel::msgs::chan_open_try::MsgChannelOpenTry;
use ibc::core::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use ibc::core::ics04_channel::packet::Packet;
use ibc::core::ics24_host::identifier::ChannelId;
use ibc::core::ics24_host::identifier::PortId;
use penumbra_chain::genesis;
use penumbra_component::Component;
use penumbra_proto::ibc::ibc_action::Action::{
    ChannelCloseConfirm, ChannelCloseInit, ChannelOpenAck, ChannelOpenConfirm, ChannelOpenInit,
    ChannelOpenTry, RecvPacket,
};
use penumbra_storage::{State, StateExt};
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

mod stateful;
mod stateless;

mod execution;

pub struct ICS4Channel {
    state: State,
}

impl ICS4Channel {
    #[instrument(name = "ics4_channel", skip(state))]
    pub async fn new(state: State) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Component for ICS4Channel {
    #[instrument(name = "ics4_channel", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {}

    #[instrument(name = "ics4_channel", skip(self, _begin_block))]
    async fn begin_block(&mut self, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "ics4_channel", skip(tx))]
    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
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

                // Other IBC messages are not handled by this component.
                _ => {}
            }
        }

        Ok(())
    }

    #[instrument(name = "ics4_channel", skip(self, tx))]
    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(ChannelOpenInit(msg)) => {
                    use stateful::channel_open_init::ChannelOpenInitCheck;
                    let msg = MsgChannelOpenInit::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                }
                Some(ChannelOpenTry(msg)) => {
                    use stateful::channel_open_try::ChannelOpenTryCheck;
                    let msg = MsgChannelOpenTry::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                }
                Some(ChannelOpenAck(msg)) => {
                    use stateful::channel_open_ack::ChannelOpenAckCheck;
                    let msg = MsgChannelOpenAck::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                }
                Some(ChannelOpenConfirm(msg)) => {
                    use stateful::channel_open_confirm::ChannelOpenConfirmCheck;
                    let msg = MsgChannelOpenConfirm::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                }
                Some(ChannelCloseInit(msg)) => {
                    use stateful::channel_close_init::ChannelCloseInitCheck;
                    let msg = MsgChannelCloseInit::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                }
                Some(ChannelCloseConfirm(msg)) => {
                    use stateful::channel_close_confirm::ChannelCloseConfirmCheck;
                    let msg = MsgChannelCloseConfirm::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                }
                Some(RecvPacket(msg)) => {
                    use stateful::recv_packet::RecvPacketCheck;
                    let msg = MsgRecvPacket::try_from(msg.clone())?;

                    self.state.validate(&msg).await?;
                }

                // Other IBC messages are not handled by this component.
                _ => {}
            }
        }
        Ok(())
    }

    #[instrument(name = "ics4_channel", skip(self, tx))]
    async fn execute_tx(&mut self, tx: &Transaction) {
        for ibc_action in tx.ibc_actions() {
            match &ibc_action.action {
                Some(ChannelOpenInit(msg)) => {
                    use execution::channel_open_init::ChannelOpenInitExecute;
                    let msg = MsgChannelOpenInit::try_from(msg.clone()).unwrap();

                    self.state.execute(&msg).await;
                }
                Some(ChannelOpenTry(msg)) => {
                    use execution::channel_open_try::ChannelOpenTryExecute;
                    let msg = MsgChannelOpenTry::try_from(msg.clone()).unwrap();

                    self.state.execute(&msg).await;
                }
                Some(ChannelOpenAck(msg)) => {
                    use execution::channel_open_ack::ChannelOpenAckExecute;
                    let msg = MsgChannelOpenAck::try_from(msg.clone()).unwrap();

                    self.state.execute(&msg).await;
                }
                Some(ChannelOpenConfirm(msg)) => {
                    use execution::channel_open_confirm::ChannelOpenConfirmExecute;
                    let msg = MsgChannelOpenConfirm::try_from(msg.clone()).unwrap();

                    self.state.execute(&msg).await;
                }
                Some(ChannelCloseInit(msg)) => {
                    use execution::channel_close_init::ChannelCloseInitExecute;
                    let msg = MsgChannelCloseInit::try_from(msg.clone()).unwrap();

                    self.state.execute(&msg).await;
                }
                Some(ChannelCloseConfirm(msg)) => {
                    use execution::channel_close_confirm::ChannelCloseConfirmExecute;
                    let msg = MsgChannelCloseConfirm::try_from(msg.clone()).unwrap();

                    self.state.execute(&msg).await;
                }

                // Other IBC messages are not handled by this component.
                _ => {}
            }
        }
    }

    #[instrument(name = "ics4_channel", skip(self, _end_block))]
    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) {}
}

impl ICS4Channel {}

#[async_trait]
pub trait View: StateExt {
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
        self.get_domain(format!("channelEnds/ports/{}/channels/{}", port_id, channel_id).into())
            .await
    }
    async fn put_channel(&mut self, channel_id: &ChannelId, port_id: &PortId, channel: ChannelEnd) {
        self.put_domain(
            format!("channelEnds/ports/{}/channels/{}", port_id, channel_id).into(),
            channel,
        )
        .await;
    }
    async fn put_ack_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(
            format!(
                "seqAcks/ports/{}/channels/{}/nextSequenceAck",
                port_id, channel_id
            )
            .into(),
            sequence,
        )
        .await;
    }
    async fn put_recv_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(
            format!(
                "seqRecvs/ports/{}/channels/{}/nextSequenceRecv",
                port_id, channel_id
            )
            .into(),
            sequence,
        )
        .await;
    }
    async fn put_send_sequence(&mut self, channel_id: &ChannelId, port_id: &PortId, sequence: u64) {
        self.put_proto::<u64>(
            format!(
                "seqSends/ports/{}/channels/{}/nextSequenceSend",
                port_id, channel_id
            )
            .into(),
            sequence,
        )
        .await;
    }
}

impl<T: StateExt> View for T {}
