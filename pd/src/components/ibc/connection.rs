use crate::{components::app::View as _, components::ibc::client::View as _};
use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::client_consensus::ConsensusState;
use ibc::core::ics02_client::client_def::AnyClient;
use ibc::core::ics02_client::client_def::ClientDef;
use ibc::core::ics02_client::client_state::AnyClientState;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics02_client::trust_threshold::TrustThreshold;
use ibc::core::ics03_connection::connection::Counterparty;
use ibc::core::ics03_connection::connection::{ConnectionEnd, State as ConnectionState};
use ibc::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use ibc::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use ibc::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use ibc::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use ibc::core::ics03_connection::version::Version;
use ibc::core::ics23_commitment::commitment::CommitmentProofBytes;
use ibc::core::ics23_commitment::specs::ProofSpecs;
use ibc::core::ics24_host::identifier::ChainId;
use ibc::core::ics24_host::identifier::ConnectionId;
use ibc::downcast;
use ibc::proofs::ConsensusProof;
use ibc::proofs::Proofs;
use ibc::Height;
use ibc::Height as IBCHeight;
use penumbra_chain::genesis;
use penumbra_component::Component;
use penumbra_ibc::{Connection, ConnectionCounter, COMMITMENT_PREFIX};
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
            let compatible_versions = vec![Version::default()];

            if !compatible_versions.contains(
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

// Check that the trust threshold is:
//
// a) non-zero
// b) greater or equal to 1/3
// c) strictly less than 1
fn validate_trust_threshold(
    id: &ChainId,
    trust_threshold: TrustThreshold,
) -> Result<(), anyhow::Error> {
    if trust_threshold.denominator() == 0 {
        return Err(anyhow::anyhow!(
            "trust threshold denominator cannot be zero"
        ));
    }

    if trust_threshold.numerator() * 3 < trust_threshold.denominator() {
        return Err(anyhow::anyhow!("trust threshold must be greater than 1/3"));
    }

    if trust_threshold.numerator() >= trust_threshold.denominator() {
        return Err(anyhow::anyhow!(
            "trust threshold must be strictly less than 1"
        ));
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

            Some(ConnectionOpenAck(msg)) => {
                let _msg_connection_open_ack = MsgConnectionOpenAck::try_from(msg.clone()).unwrap();
            }

            Some(ConnectionOpenConfirm(msg)) => {
                let _msg_connection_open_confirm =
                    MsgConnectionOpenConfirm::try_from(msg.clone()).unwrap();
            }

            _ => {}
        }
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
        // todo: construct a connection object with the TryOpen state and save it to the state
    }

    // validate the client state given to us in a MsgConnectionOpenTry, verifying that the state
    // that the counterparty chain stored is valid
    //
    // TODO: this should probably be a static function in the client crate
    async fn validate_penumbra_client_state(
        &self,
        client_state: AnyClientState,
    ) -> Result<(), anyhow::Error> {
        let tm_client_state =
            downcast!(client_state => AnyClientState::Tendermint).ok_or_else(|| {
                anyhow::anyhow!("invalid client state: not a Tendermint client state")
            })?;

        if tm_client_state.frozen_height.is_some() {
            return Err(anyhow::anyhow!("invalid client state: frozen"));
        }

        // NOTE: Chain ID validation is actually not standardized yet. see
        // https://github.com/informalsystems/ibc-rs/pull/304#discussion_r503917283
        let chain_id = ChainId::from_string(&self.state.get_chain_id().await?);
        if chain_id != tm_client_state.chain_id {
            return Err(anyhow::anyhow!(
                "invalid client state: chain id does not match"
            ));
        }

        // check that the revision number is the same as our chain ID's version
        if tm_client_state.latest_height.revision_number != chain_id.version() {
            return Err(anyhow::anyhow!(
                "invalid client state: revision number does not match"
            ));
        }

        // check that the latest height isn't gte the current block height
        if tm_client_state.latest_height.revision_height >= self.state.get_block_height().await? {
            return Err(anyhow::anyhow!(
                "invalid client state: latest height is greater than or equal to the current block height"
            ));
        }

        // check client proof specs match penumbra proof specs
        let penumbra_proof_specs: ProofSpecs = vec![jmt::ics23_spec()].into();
        if penumbra_proof_specs != tm_client_state.proof_specs {
            return Err(anyhow::anyhow!(
                "invalid client state: proof specs do not match"
            ));
        }

        // check that the trust level is correct
        validate_trust_threshold(&chain_id, tm_client_state.trust_level)?;

        // TODO: check that the unbonding period is correct
        //
        // - check unbonding period is greater than trusting period
        if tm_client_state.unbonding_period < tm_client_state.trusting_period {
            return Err(anyhow::anyhow!(
                "invalid client state: unbonding period is less than trusting period"
            ));
        }
        // TODO: check upgrade path

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

        // verify the client state
        let provided_cs = msg
            .client_state
            .clone()
            .ok_or_else(|| anyhow::anyhow!("client state not provided in MsgConnectionOpenTry"))?;

        self.validate_penumbra_client_state(provided_cs).await?;

        let mut new_conn = ConnectionEnd::new(
            ConnectionState::Init,
            msg.client_id.clone(),
            msg.counterparty.clone(),
            msg.counterparty_versions.clone(),
            msg.delay_period,
        );

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

            new_conn = prev_connection;
        }

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

        // 1. verify that the counterparty chain committed the expected_conn to its state
        self.verify_connection_proof(
            msg.proofs.height(),
            &new_conn,
            &expected_conn,
            msg.proofs.height(),
            msg.proofs.object_proof(),
        )
        .await?;

        // 2. verify that the counterparty chain committed the correct ClientState (that was
        //    provided in the msg)
        self.verify_client_proof(
            msg.proofs.height(),
            &new_conn,
            msg.client_state
                .clone()
                .ok_or_else(|| anyhow::anyhow!("client state not provided in connectionOpenTry"))?,
            msg.proofs.height(),
            msg.proofs.client_proof().as_ref().ok_or_else(|| {
                anyhow::anyhow!("client state proof not provided in the connectionOpenTry")
            })?,
        )
        .await?;

        // 3. verify that the counterparty chain stored the correct consensus state of Penumbra at
        //    the given consensus height
        self.verify_consensus_proof(
            msg.proofs.height(),
            &new_conn,
            &msg.proofs.consensus_proof().ok_or_else(|| {
                anyhow::anyhow!("consensus proof not provided in the connectionOpenTry")
            })?,
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

    async fn verify_consensus_proof(
        &self,
        height: Height,
        connection_end: &ConnectionEnd,
        proof: &ConsensusProof,
    ) -> Result<()> {
        let client_state = self
            .state
            .get_client_data(connection_end.client_id())
            .await?
            .client_state
            .0;

        if client_state.is_frozen() {
            return Err(anyhow::anyhow!("client is frozen"));
        }

        let expected_consensus = self
            .state
            .get_penumbra_consensus_state(proof.height())
            .await?
            .0;

        let consensus_state = self
            .state
            .get_verified_consensus_state(height, connection_end.client_id().clone())
            .await?
            .0;

        let client = AnyClient::from_client_type(client_state.client_type());

        client.verify_client_consensus_state(
            &client_state,
            height,
            connection_end.counterparty().prefix(),
            proof.proof(),
            consensus_state.root(),
            connection_end.counterparty().client_id(),
            proof.height(),
            &expected_consensus,
        )?;

        Ok(())
    }

    async fn verify_client_proof(
        &self,
        height: Height,
        connection_end: &ConnectionEnd,
        expected_client_state: AnyClientState,
        proof_height: Height,
        proof: &CommitmentProofBytes,
    ) -> Result<()> {
        let client_state = self
            .state
            .get_client_data(connection_end.client_id())
            .await?
            .client_state
            .0;

        if client_state.is_frozen() {
            return Err(anyhow::anyhow!("client is frozen"));
        }

        let consensus_state = self
            .state
            .get_verified_consensus_state(height, connection_end.client_id().clone())
            .await?
            .0;

        let client_def = AnyClient::from_client_type(client_state.client_type());

        client_def.verify_client_full_state(
            &client_state,
            height,
            connection_end.counterparty().prefix(),
            proof,
            consensus_state.root(),
            connection_end.counterparty().client_id(),
            &expected_client_state,
        )?;

        Ok(())
    }

    async fn verify_connection_proof(
        &self,
        height: Height,
        connection_end: &ConnectionEnd,
        expected_conn: &ConnectionEnd,
        proof_height: Height,
        proof: &CommitmentProofBytes,
    ) -> Result<()> {
        let client_state = self
            .state
            .get_client_data(&connection_end.client_id())
            .await?
            .client_state
            .0;

        // check if the client is frozen
        // TODO: should we also check if the client is expired here?
        if client_state.is_frozen() {
            return Err(anyhow::anyhow!("client is frozen"));
        }

        let consensus_state = self
            .state
            .get_verified_consensus_state(proof_height, connection_end.client_id().clone())
            .await?
            .0;

        let connection_id = connection_end
            .counterparty()
            .connection_id()
            .ok_or_else(|| anyhow::anyhow!("connection id for counterparty not provided"))?;

        let client_def = AnyClient::from_client_type(client_state.client_type());

        client_def.verify_connection_state(
            &client_state,
            height,
            connection_end.counterparty().prefix(),
            proof,
            consensus_state.root(),
            connection_id,
            expected_conn,
        )?;

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
}

impl<T: StateExt + Send + Sync> View for T {}
