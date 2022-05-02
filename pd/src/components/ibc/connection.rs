use crate::{components::app::View as _, components::ibc::client::View as _};
use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::client_consensus::ConsensusState;
use ibc::core::ics02_client::client_def::AnyClient;
use ibc::core::ics02_client::client_def::ClientDef;
use ibc::core::ics02_client::client_state::AnyClientState;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics03_connection::connection::Counterparty;
use ibc::core::ics03_connection::connection::{ConnectionEnd, State as ConnectionState};
use ibc::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use ibc::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use ibc::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use ibc::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use ibc::core::ics03_connection::version::{pick_version, Version};
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::ics24_host::identifier::ConnectionId;
use ibc::proofs::Proofs;
use ibc::Height as IBCHeight;
use penumbra_chain::genesis;
use penumbra_component::Component;
use penumbra_ibc::{
    validate_penumbra_client_state, Connection, ConnectionCounter, COMMITMENT_PREFIX,
    SUPPORTED_VERSIONS,
};
use penumbra_proto::ibc::{
    ibc_action::Action::{
        ConnectionOpenAck, ConnectionOpenConfirm, ConnectionOpenInit, ConnectionOpenTry,
    },
    IbcAction,
};
use penumbra_storage::{State, StateExt};
use penumbra_transaction::Transaction;
use tendermint::abci;
use tracing::instrument;

pub struct ConnectionComponent {
    state: State,
}

#[async_trait]
impl Component for ConnectionComponent {
    #[instrument(name = "ibc_connection", skip(state))]
    async fn new(state: State) -> Self {
        Self { state }
    }

    #[instrument(name = "ibc_connection", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {}

    #[instrument(name = "ibc_connection", skip(self, _begin_block))]
    async fn begin_block(&mut self, _begin_block: &abci::request::BeginBlock) {}

    #[instrument(name = "ibc_connection", skip(tx))]
    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            validate_ibc_action_stateless(ibc_action)?;
        }

        Ok(())
    }

    #[instrument(name = "ibc_connection", skip(self, tx))]
    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            self.validate_ibc_action_stateful(ibc_action).await?;
        }
        Ok(())
    }

    #[instrument(name = "ibc_connection", skip(self, tx))]
    async fn execute_tx(&mut self, tx: &Transaction) {
        for ibc_action in tx.ibc_actions() {
            self.execute_ibc_action(ibc_action).await;
        }
    }

    #[instrument(name = "ibc_connection", skip(self, _end_block))]
    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) {}
}

fn validate_ibc_action_stateless(ibc_action: &IbcAction) -> Result<(), anyhow::Error> {
    match &ibc_action.action {
        Some(ConnectionOpenInit(msg)) => {
            let msg_connection_open_init = MsgConnectionOpenInit::try_from(msg.clone())?;

            // check if the version is supported (we use the same versions as the cosmos SDK)
            // TODO: should we be storing the compatible versions in our state instead?
            if !SUPPORTED_VERSIONS.contains(
                &msg_connection_open_init
                    .version
                    .ok_or_else(|| anyhow::anyhow!("invalid version"))?,
            ) {
                return Err(anyhow::anyhow!(
                    "unsupported version: in ConnectionOpenInit",
                ));
            }

            return Ok(());
        }

        // process a notice of a connection attempt on a counterparty chain
        Some(ConnectionOpenTry(msg)) => {
            let _ = MsgConnectionOpenTry::try_from(msg.clone())?;
        }

        Some(ConnectionOpenAck(msg)) => {
            let _msg_connection_open_ack = MsgConnectionOpenAck::try_from(msg.clone())?;
        }

        Some(ConnectionOpenConfirm(msg)) => {
            let _msg_connection_open_confirm = MsgConnectionOpenConfirm::try_from(msg.clone())?;
        }

        _ => {}
    }

    Ok(())
}
impl ConnectionComponent {
    async fn execute_ibc_action(&mut self, ibc_action: &IbcAction) {
        match &ibc_action.action {
            Some(ConnectionOpenInit(msg)) => {
                let msg_connection_open_init =
                    MsgConnectionOpenInit::try_from(msg.clone()).unwrap();
                self.execute_connection_open_init(&msg_connection_open_init)
                    .await;
            }

            Some(ConnectionOpenTry(raw_msg)) => {
                let msg = MsgConnectionOpenTry::try_from(raw_msg.clone()).unwrap();
                self.execute_connection_open_try(&msg).await;
            }

            Some(ConnectionOpenAck(raw_msg)) => {
                let msg = MsgConnectionOpenAck::try_from(raw_msg.clone()).unwrap();
                self.execute_connection_open_ack(&msg).await;
            }

            Some(ConnectionOpenConfirm(msg)) => {
                let _msg_connection_open_confirm =
                    MsgConnectionOpenConfirm::try_from(msg.clone()).unwrap();
            }

            _ => {}
        }
    }

    async fn execute_connection_open_ack(&mut self, msg: &MsgConnectionOpenAck) {
        let mut connection = self
            .state
            .get_connection(&msg.connection_id)
            .await
            .unwrap()
            .unwrap()
            .0;

        let prev_counterparty = connection.counterparty();
        let counterparty = Counterparty::new(
            prev_counterparty.client_id().clone(),
            Some(msg.connection_id.clone()),
            prev_counterparty.prefix().clone(),
        );
        connection.set_state(ConnectionState::Open);
        connection.set_version(msg.version.clone());
        connection.set_counterparty(counterparty);

        self.state
            .update_connection(&msg.connection_id, connection.into())
            .await;
    }

    async fn execute_connection_open_init(&mut self, msg: &MsgConnectionOpenInit) {
        let connection_id = ConnectionId::new(self.state.get_connection_counter().await.unwrap().0);

        let compatible_versions = vec![Version::default()];

        let new_connection_end = ConnectionEnd::new(
            ConnectionState::Init,
            msg.client_id.clone(),
            msg.counterparty.clone(),
            compatible_versions,
            msg.delay_period,
        );

        // commit the connection, this also increments the connection counter
        self.state
            .put_new_connection(&connection_id, new_connection_end.into())
            .await
            .unwrap();
    }

    async fn execute_connection_open_try(&mut self, msg: &MsgConnectionOpenTry) {
        // new_conn is the new connection that we will open on this chain
        let mut new_conn = ConnectionEnd::new(
            ConnectionState::TryOpen,
            msg.client_id.clone(),
            msg.counterparty.clone(),
            msg.counterparty_versions.clone(),
            msg.delay_period,
        );
        new_conn.set_version(
            pick_version(
                SUPPORTED_VERSIONS.to_vec(),
                msg.counterparty_versions.clone(),
            )
            .unwrap(),
        );

        let mut new_connection_id =
            ConnectionId::new(self.state.get_connection_counter().await.unwrap().0);

        if let Some(prev_conn_id) = &msg.previous_connection_id {
            // prev conn ID already validated in check_tx_stateful
            new_connection_id = prev_conn_id.clone();
        }

        self.state
            .put_new_connection(&new_connection_id, new_conn.into())
            .await
            .unwrap();
    }

    async fn verify_connection_proofs(
        &self,
        client_id: &ClientId,
        msg_proofs: Proofs,
        msg_client_state: &AnyClientState,
        counterparty_connection_id: &ConnectionId,
        counterparty: Counterparty,
        expected_conn: ConnectionEnd,
    ) -> Result<()> {
        // get the stored client state for the counterparty
        let stored_client_state = self.state.get_client_data(client_id).await?.client_state.0;

        // check if the client is frozen
        // TODO: should we also check if the client is expired here?
        if stored_client_state.is_frozen() {
            return Err(anyhow::anyhow!("client is frozen"));
        }

        // get the stored consensus state for the counterparty
        let stored_consensus_state = self
            .state
            .get_verified_consensus_state(msg_proofs.height(), client_id.clone())
            .await?
            .0;

        let client_def = AnyClient::from_client_type(stored_client_state.client_type());

        // PROOF VERIFICATION
        // 1. verify that the counterparty chain committed the expected_conn to its state
        client_def.verify_connection_state(
            &stored_client_state,
            msg_proofs.height(),
            counterparty.prefix(),
            msg_proofs.object_proof(),
            stored_consensus_state.root(),
            &counterparty_connection_id,
            &expected_conn,
        )?;

        // 2. verify that the counterparty chain committed the correct ClientState (that was
        //    provided in the msg)
        client_def.verify_client_full_state(
            &stored_client_state,
            msg_proofs.height(),
            counterparty.prefix(),
            msg_proofs.client_proof().as_ref().ok_or_else(|| {
                anyhow::anyhow!("client proof not provided in the connectionOpenTry")
            })?,
            stored_consensus_state.root(),
            counterparty.client_id(),
            msg_client_state,
        )?;

        let cons_proof = msg_proofs.consensus_proof().ok_or_else(|| {
            anyhow::anyhow!("consensus proof not provided in the connectionOpenTry")
        })?;
        let expected_consensus = self
            .state
            .get_penumbra_consensus_state(cons_proof.height())
            .await?
            .0;

        // 3. verify that the counterparty chain stored the correct consensus state of Penumbra at
        //    the given consensus height
        client_def.verify_client_consensus_state(
            &stored_client_state,
            msg_proofs.height(),
            counterparty.prefix(),
            cons_proof.proof(),
            stored_consensus_state.root(),
            counterparty.client_id(),
            cons_proof.height(),
            &expected_consensus,
        )?;

        Ok(())
    }

    async fn validate_connection_open_try(
        &self,
        msg: &MsgConnectionOpenTry,
    ) -> Result<(), anyhow::Error> {
        // verify the consensus height is correct
        if msg.consensus_height()
            > IBCHeight::zero().with_revision_height(self.state.get_block_height().await?)
        {
            return Err(anyhow::anyhow!(
                "consensus height is greater than the current block height",
            ));
        }

        //
        // TODO: store our earliest non-pruned block height and check against the consensus height
        //

        // verify the provided client state
        let provided_cs = msg
            .client_state
            .clone()
            .ok_or_else(|| anyhow::anyhow!("client state not provided in MsgConnectionOpenTry"))?;

        let height = self.state.get_block_height().await?;
        let chain_id = self.state.get_chain_id().await?;
        validate_penumbra_client_state(provided_cs, &chain_id, height)?;

        let mut previous_conn: Option<ConnectionEnd> = None;
        if let Some(prev_conn_id) = &msg.previous_connection_id {
            // check that we have a valid connection with the given ID
            let prev_connection = self
                .state
                .get_connection(prev_conn_id)
                .await?
                .ok_or_else(|| anyhow::anyhow!("no connection with the given ID"))?
                .0;

            // check that the existing connection matches the incoming connectionOpenTry
            if !(prev_connection.state_matches(&ConnectionState::Init)
                && prev_connection.counterparty_matches(&msg.counterparty)
                && prev_connection.client_id_matches(&msg.client_id)
                && prev_connection.delay_period() == msg.delay_period)
            {
                return Err(anyhow::anyhow!(
                    "connection with the given ID is not in the correct state",
                ));
            }
            previous_conn = Some(prev_connection);
        }

        // expected_conn is the conn that we expect to have been committed to on the counterparty
        // chain
        let expected_conn = ConnectionEnd::new(
            ConnectionState::Init,
            msg.counterparty.client_id().clone(),
            Counterparty::new(
                msg.client_id.clone(),
                None,
                COMMITMENT_PREFIX.as_bytes().to_vec().try_into().unwrap(),
            ),
            msg.counterparty_versions.clone(),
            msg.delay_period,
        );

        // perform version intersection
        let mut supported_versions = SUPPORTED_VERSIONS.clone();
        if let Some(prev_conn) = previous_conn {
            supported_versions = prev_conn.versions().to_vec();
        }
        pick_version(supported_versions, msg.counterparty_versions.clone())?;

        // get the connection ID of the counterparty
        let counterparty_connection_id = msg
            .counterparty
            .connection_id
            .clone()
            .ok_or_else(|| anyhow::anyhow!("connection id for counterparty not provided"))?;

        // PROOF VERIFICATION
        self.verify_connection_proofs(
            &msg.client_id,
            msg.proofs.clone(),
            msg.client_state.as_ref().ok_or_else(|| {
                anyhow::anyhow!("client state not provided in MsgConnectionOpenTry")
            })?,
            &counterparty_connection_id,
            msg.counterparty.clone(),
            expected_conn,
        )
        .await?;

        Ok(())
    }

    async fn validate_connection_open_ack(
        &self,
        msg: &MsgConnectionOpenAck,
    ) -> Result<(), anyhow::Error> {
        // verify the consensus height is correct
        if msg.consensus_height()
            > IBCHeight::zero().with_revision_height(self.state.get_block_height().await?)
        {
            return Err(anyhow::anyhow!(
                "consensus height is greater than the current block height",
            ));
        }

        // verify the provided client state
        let provided_cs = msg
            .client_state
            .clone()
            .ok_or_else(|| anyhow::anyhow!("client state not provided in MsgConnectionOpenTry"))?;

        let height = self.state.get_block_height().await?;
        let chain_id = self.state.get_chain_id().await?;
        validate_penumbra_client_state(provided_cs, &chain_id, height)?;

        let connection = self
            .state
            .get_connection(&msg.connection_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("no connection with the given ID"))?
            .0;

        let state_is_consistent = connection.state_matches(&ConnectionState::Init)
            && connection.versions().contains(&msg.version)
            || connection.state_matches(&ConnectionState::TryOpen)
                && connection.versions().get(0).eq(&Some(&msg.version));

        if !state_is_consistent {
            return Err(anyhow::anyhow!("connection is not in the correct state"));
        }

        // we are the counterparty here
        let counterparty = Counterparty::new(
            connection.client_id().clone(),
            Some(msg.counterparty_connection_id.clone()),
            COMMITMENT_PREFIX.as_bytes().to_vec().try_into().unwrap(),
        );

        let expected_conn = ConnectionEnd::new(
            ConnectionState::TryOpen,
            connection.counterparty().client_id().clone(),
            counterparty.clone(),
            vec![msg.version.clone()],
            connection.delay_period(),
        );

        // PROOF VERIFICATION
        self.verify_connection_proofs(
            connection.client_id(),
            msg.proofs.clone(),
            msg.client_state.as_ref().ok_or_else(|| {
                anyhow::anyhow!("client state not provided in MsgConnectionOpenAck")
            })?,
            counterparty
                .connection_id()
                .ok_or_else(|| anyhow::anyhow!("connection id for counterparty not provided"))?,
            counterparty.clone(),
            expected_conn,
        )
        .await?;

        Ok(())
    }

    async fn validate_ibc_action_stateful(&self, ibc_action: &IbcAction) -> Result<()> {
        match &ibc_action.action {
            Some(ConnectionOpenInit(raw_msg)) => {
                // check that the client id exists
                let msg = MsgConnectionOpenInit::try_from(raw_msg.clone())?;
                self.state.get_client_data(&msg.client_id).await?;

                return Ok(());
            }

            Some(ConnectionOpenTry(raw_msg)) => {
                let msg = MsgConnectionOpenTry::try_from(raw_msg.clone())?;
                self.validate_connection_open_try(&msg).await?;
            }

            Some(ConnectionOpenAck(raw_msg)) => {
                let msg = MsgConnectionOpenAck::try_from(raw_msg.clone())?;
                self.validate_connection_open_ack(&msg).await?;
            }

            Some(ConnectionOpenConfirm(msg)) => {
                let _msg_connection_open_confirm = MsgConnectionOpenConfirm::try_from(msg.clone())?;
            }

            _ => {}
        }

        Ok(())
    }
}

#[async_trait]
pub trait View: StateExt + Send + Sync {
    async fn get_connection_counter(&self) -> Result<ConnectionCounter> {
        self.get_domain("ibc/ics03-connection/connection_counter".into())
            .await
            .map(|counter| counter.unwrap_or(ConnectionCounter(0)))
    }

    async fn put_connection_counter(&self, counter: ConnectionCounter) {
        self.put_domain("ibc/ics03-connection/connection_counter".into(), counter)
            .await;
    }

    // puts a new connection into the state, updating the connections associated with the client,
    // and incrementing the client counter.
    async fn put_new_connection(
        &mut self,
        connection_id: &ConnectionId,
        connection: Connection,
    ) -> Result<()> {
        self.put_domain(
            format!(
                "ibc/ics03-connection/connections/{}",
                connection_id.as_str()
            )
            .into(),
            connection.clone(),
        )
        .await;
        let counter = self
            .get_connection_counter()
            .await
            .unwrap_or(ConnectionCounter(0));
        self.put_connection_counter(ConnectionCounter(counter.0 + 1))
            .await;

        self.add_connection_to_client(connection.0.client_id(), connection_id)
            .await?;

        return Ok(());
    }

    async fn get_connection(&self, connection_id: &ConnectionId) -> Result<Option<Connection>> {
        self.get_domain(
            format!(
                "ibc/ics03-connection/connections/{}",
                connection_id.as_str()
            )
            .into(),
        )
        .await
    }

    async fn update_connection(&self, connection_id: &ConnectionId, connection: Connection) {
        self.put_domain(
            format!(
                "ibc/ics03-connection/connections/{}",
                connection_id.as_str()
            )
            .into(),
            connection,
        )
        .await;
    }
}

impl<T: StateExt + Send + Sync> View for T {}
