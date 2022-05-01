use std::convert::TryFrom;

use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics24_host::identifier::ConnectionId;
use ibc::downcast;
use ibc::{
    clients::ics07_tendermint::{
        client_state::ClientState as TendermintClientState,
        consensus_state::ConsensusState as TendermintConsensusState,
        header::Header as TendermintHeader,
    },
    core::{
        ics02_client::{
            client_consensus::AnyConsensusState,
            client_state::{AnyClientState, ClientState},
            header::AnyHeader,
            height::Height,
            msgs::{create_client::MsgCreateAnyClient, update_client::MsgUpdateAnyClient},
        },
        ics24_host::identifier::ClientId,
    },
};
use penumbra_chain::genesis;
use penumbra_component::Component;
use penumbra_ibc::{ClientConnections, ClientCounter, ClientData, ConsensusState, VerifiedHeights};
use penumbra_proto::ibc::{
    ibc_action::Action::{CreateClient, UpdateClient},
    IbcAction,
};
use penumbra_storage::{State, StateExt};
use penumbra_transaction::Transaction;
use tendermint::abci;
use tendermint_light_client_verifier::{
    types::{TrustedBlockState, UntrustedBlockState},
    ProdVerifier, Verdict, Verifier,
};
use tracing::instrument;

use crate::components::app::View as _;

/// The Penumbra IBC client component. Handles all client-related IBC actions: MsgCreateClient,
/// MsgUpdateClient, MsgUpgradeClient, and MsgSubmitMisbehaviour. The core responsibility of the
/// client component is tracking light clients for IBC, creating new light clients and verifying
/// state updates. Currently, only Tendermint light clients are supported.
pub struct ClientComponent {
    state: State,
}

#[async_trait]
impl Component for ClientComponent {
    #[instrument(name = "ics2_client", skip(state))]
    async fn new(state: State) -> Self {
        Self { state }
    }

    #[instrument(name = "ics2_client", skip(self, _app_state))]
    async fn init_chain(&mut self, _app_state: &genesis::AppState) {
        // set the initial client count
        self.state.put_client_counter(ClientCounter(0)).await;
    }

    #[instrument(name = "ics2_client", skip(self, begin_block))]
    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock) {
        // save the penumbra verified consensus state for this block

        let cs = TendermintConsensusState::new(
            begin_block.header.app_hash.value().into(),
            begin_block.header.time,
            begin_block.header.next_validators_hash,
        );

        // TODO: hard-coded revision number
        let height = Height::new(0, begin_block.header.height.into());

        self.state
            .put_penumbra_consensus_state(height, cs.into())
            .await;
    }

    #[instrument(name = "ics2_client", skip(tx))]
    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            validate_ibc_action_stateless(ibc_action)?;
        }
        Ok(())
    }

    #[instrument(name = "ics2_client", skip(self, tx))]
    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        for ibc_action in tx.ibc_actions() {
            self.validate_ibc_action_stateful(ibc_action).await?;
        }
        Ok(())
    }

    #[instrument(name = "ics2_client", skip(self, tx))]
    async fn execute_tx(&mut self, tx: &Transaction) {
        // Handle any IBC actions found in the transaction.
        for ibc_action in tx.ibc_actions() {
            self.execute_ibc_action(ibc_action).await;
        }
    }

    #[instrument(name = "ics2_client", skip(self, _end_block))]
    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) {}
}

// validates the given ibc action statelessly
fn validate_ibc_action_stateless(ibc_action: &IbcAction) -> Result<(), anyhow::Error> {
    match &ibc_action.action {
        Some(CreateClient(msg)) => {
            let msg_create_client = MsgCreateAnyClient::try_from(msg.clone())?;

            validate_create_client_stateless(&msg_create_client)?;
        }
        Some(UpdateClient(msg)) => {
            let msg_update_client = MsgUpdateAnyClient::try_from(msg.clone())?;

            validate_update_client_stateless(&msg_update_client)?;
        }
        _ => return Ok(()),
    }

    Ok(())
}

// check that the client is Tendermint
fn validate_create_client_stateless(
    create_client: &MsgCreateAnyClient,
) -> Result<(), anyhow::Error> {
    match create_client.client_state {
        AnyClientState::Tendermint(_) => {}
        _ => {
            return Err(anyhow::anyhow!(
                "only Tendermint clients are supported at this time"
            ))
        }
    }
    match create_client.consensus_state {
        AnyConsensusState::Tendermint(_) => {}
        _ => {
            return Err(anyhow::anyhow!(
                "only Tendermint consensus is supported at this time"
            ))
        }
    }

    Ok(())
}

fn validate_update_client_stateless(
    update_client: &MsgUpdateAnyClient,
) -> Result<(), anyhow::Error> {
    match update_client.header {
        AnyHeader::Tendermint(_) => {}
        _ => {
            return Err(anyhow::anyhow!(
                "only Tendermint clients are supported at this time"
            ))
        }
    }

    Ok(())
}

impl ClientComponent {
    // validates the given IBC action statefully.
    async fn validate_ibc_action_stateful(&self, ibc_action: &IbcAction) -> Result<()> {
        match &ibc_action.action {
            Some(CreateClient(msg)) => {
                let msg_create_client = MsgCreateAnyClient::try_from(msg.clone())?;

                self.validate_create_client_stateful(msg_create_client)
                    .await?;
            }
            Some(UpdateClient(msg)) => {
                let msg_update_client = MsgUpdateAnyClient::try_from(msg.clone())?;

                self.validate_update_client_stateful(msg_update_client)
                    .await?;
            }
            _ => return Ok(()),
        }

        Ok(())
    }

    // executes the given IBC action, assuming that it has already been validated.
    async fn execute_ibc_action(&mut self, ibc_action: &IbcAction) {
        match &ibc_action.action {
            Some(CreateClient(raw_msg_create_client)) => {
                let msg_create_client =
                    MsgCreateAnyClient::try_from(raw_msg_create_client.clone()).unwrap();

                self.execute_create_client(msg_create_client).await;
            }
            Some(UpdateClient(raw_msg_update_client)) => {
                let msg_update_client =
                    MsgUpdateAnyClient::try_from(raw_msg_update_client.clone()).unwrap();

                self.execute_update_client(msg_update_client).await;
            }
            _ => {}
        }
    }

    // validate IBC UpdateClient. This is one of the most important IBC messages, as it has
    // the responsibility of verifying consensus state updates (through the semantics of
    // the light client's verification fn).
    //
    // verify:
    // - we have a client corresponding to the UpdateClient's client_id
    // - the stored client is not frozen
    // - the stored client is not expired
    // - the supplied update verifies using the semantics of the light client's header verification fn
    async fn validate_update_client_stateful(
        &self,
        msg_update_client: MsgUpdateAnyClient,
    ) -> Result<()> {
        // get the latest client state
        let client_data = self
            .state
            .get_client_data(&msg_update_client.client_id)
            .await?;

        // check that the client is not frozen or expired
        if client_data.client_state.0.is_frozen() {
            return Err(anyhow::anyhow!("client is frozen"));
        }

        // check if client is expired
        let latest_consensus_state = self
            .state
            .get_verified_consensus_state(
                client_data.client_state.0.latest_height(),
                client_data.client_id.clone(),
            )
            .await?;

        let latest_consensus_state_tm =
            downcast!(latest_consensus_state.0 => AnyConsensusState::Tendermint).ok_or_else(
                || anyhow::anyhow!("invalid consensus state: not a Tendermint consensus state"),
            )?;

        let now = self.state.get_block_timestamp().await?;
        if client_data.client_state.0.expired(
            now.duration_since(latest_consensus_state_tm.timestamp)
                .unwrap(),
        ) {
            return Err(anyhow::anyhow!("client is expired"));
        }

        // verify the clientupdate's header
        let tm_client_state = match client_data.clone().client_state.0 {
            AnyClientState::Tendermint(tm_state) => tm_state,
            _ => return Err(anyhow::anyhow!("unsupported client type")),
        };
        let tm_header = match msg_update_client.header {
            AnyHeader::Tendermint(tm_header) => tm_header,
            _ => {
                return Err(anyhow::anyhow!("client update is not a Tendermint header"));
            }
        };

        self.verify_tendermint_update(
            msg_update_client.client_id.clone(),
            tm_client_state,
            tm_header.clone(),
        )
        .await?;

        Ok(())
    }

    async fn validate_create_client_stateful(
        &self,
        msg_create_client: MsgCreateAnyClient,
    ) -> Result<()> {
        let id_counter = self.state.client_counter().await?;
        ClientId::new(msg_create_client.client_state.client_type(), id_counter.0)?;

        Ok(())
    }

    // execute a UpdateClient IBC action. this assumes that the UpdateClient has already been
    // validated, including header verification.
    async fn execute_update_client(&mut self, msg_update_client: MsgUpdateAnyClient) {
        // get the latest client state
        let client_data = self
            .state
            .get_client_data(&msg_update_client.client_id)
            .await
            .unwrap();

        let tm_client_state = match client_data.clone().client_state.0 {
            AnyClientState::Tendermint(tm_state) => tm_state,
            _ => panic!("unsupported client type"),
        };
        let tm_header = match msg_update_client.header {
            AnyHeader::Tendermint(tm_header) => tm_header,
            _ => {
                panic!("update header is not a Tendermint header");
            }
        };

        let (next_tm_client_state, next_tm_consensus_state) = self
            .next_tendermint_state(
                msg_update_client.client_id.clone(),
                tm_client_state,
                tm_header.clone(),
            )
            .await;

        // store the updated client and consensus states
        let height = self.state.get_block_height().await.unwrap();
        let now = self.state.get_block_timestamp().await.unwrap();
        let next_client_data = client_data.with_new_client_state(
            AnyClientState::Tendermint(next_tm_client_state),
            now.to_rfc3339(),
            height,
        );
        self.state.put_client_data(next_client_data).await;
        self.state
            .put_verified_consensus_state(
                tm_header.height(),
                msg_update_client.client_id.clone(),
                ConsensusState(AnyConsensusState::Tendermint(next_tm_consensus_state)),
            )
            .await
            .unwrap();
    }

    // execute IBC CreateClient.
    //
    //  we compute the client's ID (a concatenation of a monotonically increasing integer, the
    //  number of clients on Penumbra, and the client type) and commit the following to our state:
    // - client type
    // - consensus state
    // - processed time and height
    async fn execute_create_client(&mut self, msg_create_client: MsgCreateAnyClient) {
        // get the current client counter
        let id_counter = self.state.client_counter().await.unwrap();
        let client_id =
            ClientId::new(msg_create_client.client_state.client_type(), id_counter.0).unwrap();

        tracing::info!("creating client {:?}", client_id);

        let height = self.state.get_block_height().await.unwrap();
        let timestamp = self.state.get_block_timestamp().await.unwrap();

        let data = ClientData::new(
            client_id.clone(),
            msg_create_client.client_state,
            timestamp.to_rfc3339(),
            height,
        );

        // store the client data
        self.state.put_client_data(data.clone()).await;

        // store the genesis consensus state
        self.state
            .put_verified_consensus_state(
                data.client_state.0.latest_height(),
                client_id,
                ConsensusState(msg_create_client.consensus_state),
            )
            .await
            .unwrap();

        // increment client counter
        let counter = self
            .state
            .client_counter()
            .await
            .unwrap_or(ClientCounter(0));
        self.state
            .put_client_counter(ClientCounter(counter.0 + 1))
            .await;
    }

    // given an already verified tendermint header, and a trusted tendermint client state, compute
    // the next client and consensus states.
    async fn next_tendermint_state(
        &self,
        client_id: ClientId,
        trusted_client_state: TendermintClientState,
        verified_header: TendermintHeader,
    ) -> (TendermintClientState, TendermintConsensusState) {
        let verified_consensus_state = TendermintConsensusState::from(verified_header.clone());

        // if we have a stored consensus state for this height that conflicts, we need to freeze
        // the client. if it doesn't conflict, we can return early
        if let Some(stored_cs_state) = self
            .state
            .get_verified_consensus_state(verified_header.height(), client_id.clone())
            .await
            .ok()
        {
            let stored_cs_state_tm = stored_cs_state.as_tendermint().unwrap();
            if stored_cs_state_tm == verified_consensus_state {
                return (trusted_client_state, verified_consensus_state);
            } else {
                return (
                    trusted_client_state
                        .with_header(verified_header.clone())
                        .with_frozen_height(verified_header.height())
                        .unwrap(),
                    verified_consensus_state,
                );
            }
        }

        // check that updates have monotonic timestamps. we may receive client updates that are
        // disjoint: the header we received and validated may be older than the newest header we
        // have. In that case, we need to verify that the timestamp is correct. if it isn't, freeze
        // the client.
        let next_consensus_state = self
            .state
            .next_verified_consensus_state(&client_id, verified_header.height())
            .await
            .unwrap();
        let prev_consensus_state = self
            .state
            .prev_verified_consensus_state(&client_id, verified_header.height())
            .await
            .unwrap();

        // case 1: if we have a verified consensus state previous to this header, verify that this
        // header's timestamp is greater than or equal to the stored consensus state's timestamp
        if let Some(prev_state) = prev_consensus_state {
            let prev_state_tm = prev_state.as_tendermint().unwrap();
            if !(verified_header.signed_header.header().time >= prev_state_tm.timestamp) {
                return (
                    trusted_client_state
                        .with_header(verified_header.clone())
                        .with_frozen_height(verified_header.height())
                        .unwrap(),
                    verified_consensus_state,
                );
            }
        }
        // case 2: if we have a verified consensus state with higher block height than this header,
        // verify that this header's timestamp is less than or equal to this header's timestamp.
        if let Some(next_state) = next_consensus_state {
            let next_state_tm = next_state.as_tendermint().unwrap();
            if !(verified_header.signed_header.header().time <= next_state_tm.timestamp) {
                return (
                    trusted_client_state
                        .with_header(verified_header.clone())
                        .with_frozen_height(verified_header.height())
                        .unwrap(),
                    verified_consensus_state,
                );
            }
        }

        return (
            trusted_client_state.with_header(verified_header.clone()),
            verified_consensus_state,
        );
    }

    // verify a new header for a tendermint client, given a trusted client state.
    async fn verify_tendermint_update(
        &self,
        client_id: ClientId,
        trusted_client_state: TendermintClientState,
        untrusted_header: TendermintHeader,
    ) -> Result<()> {
        let untrusted_consensus_state = TendermintConsensusState::from(untrusted_header.clone());

        if untrusted_header.height().revision_number != trusted_client_state.chain_id.version() {
            return Err(anyhow::anyhow!(
                "client update revision number does not match client state"
            ));
        }

        if untrusted_header.height() <= untrusted_header.trusted_height {
            return Err(anyhow::anyhow!(
                "client update height is not greater than trusted height"
            ));
        }

        // check if we already have a consensus state for this height, if we do, check that it is
        // the same as this update, if it is, return early.
        if let Ok(stored_consensus_state) = self
            .state
            .get_verified_consensus_state(untrusted_header.height(), client_id.clone())
            .await
        {
            let stored_tm_consensus_state = stored_consensus_state.as_tendermint()?;
            if stored_tm_consensus_state == untrusted_consensus_state {
                return Ok(());
            }
        }

        let last_trusted_consensus_state = self
            .state
            .get_verified_consensus_state(untrusted_header.trusted_height, client_id.clone())
            .await?
            .as_tendermint()?;

        if untrusted_header.trusted_validator_set.hash()
            != last_trusted_consensus_state.next_validators_hash
        {
            return Err(anyhow::anyhow!(
                "client update validator set hash does not match trusted consensus state"
            ));
        }

        let trusted_state = TrustedBlockState {
            header_time: last_trusted_consensus_state.timestamp,
            height: untrusted_header
                .trusted_height
                .revision_height
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid header height"))?,
            next_validators: &untrusted_header.trusted_validator_set,
            next_validators_hash: last_trusted_consensus_state.next_validators_hash,
        };

        let untrusted_state = UntrustedBlockState {
            signed_header: &untrusted_header.signed_header,
            validators: &untrusted_header.validator_set,

            // TODO: do we need this?
            next_validators: None,
        };

        let options = trusted_client_state.as_light_client_options()?;
        let verifier = ProdVerifier::default();

        let verdict = verifier.verify(
            untrusted_state,
            trusted_state,
            &options,
            self.state.get_block_timestamp().await?,
        );
        match verdict {
            Verdict::Success => {}
            Verdict::NotEnoughTrust(voting_power_tally) => {
                return Err(anyhow::anyhow!(
                    "not enough trust, voting power tally: {:?}",
                    voting_power_tally
                ));
            }
            Verdict::Invalid(detail) => {
                return Err(anyhow::anyhow!(
                    "could not verify tendermint header: invalid: {:?}",
                    detail
                ));
            }
        }

        // consensus state is verified
        Ok(())
    }
}

#[async_trait]
pub trait View: StateExt + Send + Sync {
    async fn put_client_counter(&mut self, counter: ClientCounter) {
        self.put_domain("ibc/ics02-client/client_counter".into(), counter)
            .await;
    }
    async fn client_counter(&self) -> Result<ClientCounter> {
        self.get_domain("ibc/ics02-client/client_counter".into())
            .await
            .map(|counter| counter.unwrap_or(ClientCounter(0)))
    }
    async fn put_client_data(&mut self, data: ClientData) {
        self.put_domain(
            format!(
                "ibc/ics02-client/clients/{}",
                hex::encode(data.client_id.as_bytes())
            )
            .into(),
            data,
        )
        .await;
    }
    async fn get_client_data(&self, client_id: &ClientId) -> Result<ClientData> {
        let client_data = self
            .get_domain(
                format!(
                    "ibc/ics02-client/clients/{}",
                    hex::encode(client_id.as_bytes())
                )
                .into(),
            )
            .await?;

        client_data.ok_or(anyhow::anyhow!("client not found"))
    }

    async fn get_verified_heights(&self, client_id: &ClientId) -> Result<Option<VerifiedHeights>> {
        self.get_domain(
            format!(
                "ibc/ics02-client/clients/{}/verified_heights",
                hex::encode(client_id.as_bytes())
            )
            .into(),
        )
        .await
    }

    async fn put_verified_heights(
        &mut self,
        client_id: &ClientId,
        verified_heights: VerifiedHeights,
    ) {
        self.put_domain(
            format!(
                "ibc/ics02-client/clients/{}/verified_heights",
                hex::encode(client_id.as_bytes())
            )
            .into(),
            verified_heights,
        )
        .await;
    }

    // returns the ConsensusState for the penumbra chain (this chain) at the given height
    async fn get_penumbra_consensus_state(&self, height: Height) -> Result<ConsensusState> {
        self.get_domain(format!("ibc/ics02-client/penumbra_consensus_states/{}", height).into())
            .await?
            .ok_or(anyhow::anyhow!("consensus state not found"))
    }

    // returns the ConsensusState for the penumbra chain (this chain) at the given height
    async fn put_penumbra_consensus_state(&self, height: Height, consensus_state: ConsensusState) {
        self.put_domain(
            format!("ibc/ics02-client/penumbra_consensus_states/{}", height).into(),
            consensus_state,
        )
        .await;
    }

    async fn get_verified_consensus_state(
        &self,
        height: Height,
        client_id: ClientId,
    ) -> Result<ConsensusState> {
        self.get_domain(
            format!(
                "ibc/ics02-client/clients/{}/consensus_state/{}",
                hex::encode(client_id.as_bytes()),
                height
            )
            .into(),
        )
        .await?
        .ok_or(anyhow::anyhow!("consensus state not found"))
    }

    async fn put_verified_consensus_state(
        &mut self,
        height: Height,
        client_id: ClientId,
        consensus_state: ConsensusState,
    ) -> Result<()> {
        self.put_domain(
            format!(
                "ibc/ics02-client/clients/{}/consensus_state/{}",
                hex::encode(client_id.as_bytes()),
                height
            )
            .into(),
            consensus_state,
        )
        .await;

        // update verified heights
        let mut verified_heights =
            self.get_verified_heights(&client_id)
                .await?
                .unwrap_or(VerifiedHeights {
                    heights: Vec::new(),
                });

        verified_heights.heights.push(height.clone());

        self.put_verified_heights(&client_id, verified_heights)
            .await;

        Ok(())
    }

    // returns the lowest verified consensus state that is higher than the given height, if it
    // exists.
    async fn next_verified_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Option<ConsensusState>> {
        let mut verified_heights =
            self.get_verified_heights(client_id)
                .await?
                .unwrap_or(VerifiedHeights {
                    heights: Vec::new(),
                });

        // WARNING: load-bearing sort
        verified_heights.heights.sort();

        if let Some(next_height) = verified_heights
            .heights
            .iter()
            .find(|&verified_height| verified_height > &height)
        {
            let next_cons_state = self
                .get_verified_consensus_state(*next_height, client_id.clone())
                .await?;
            return Ok(Some(next_cons_state));
        } else {
            return Ok(None);
        }
    }

    // returns the highest verified consensus state that is lower than the given height, if it
    // exists.
    async fn prev_verified_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Option<ConsensusState>> {
        let mut verified_heights =
            self.get_verified_heights(client_id)
                .await?
                .unwrap_or(VerifiedHeights {
                    heights: Vec::new(),
                });

        // WARNING: load-bearing sort
        verified_heights.heights.sort();

        if let Some(prev_height) = verified_heights
            .heights
            .iter()
            .find(|&verified_height| verified_height < &height)
        {
            let prev_cons_state = self
                .get_verified_consensus_state(*prev_height, client_id.clone())
                .await?;
            return Ok(Some(prev_cons_state));
        } else {
            return Ok(None);
        }
    }

    // adds the provided connection ID to the client identified by client_id. returns an error if
    // the client does not exist.
    async fn add_connection_to_client(
        &mut self,
        client_id: &ClientId,
        connection_id: &ConnectionId,
    ) -> Result<()> {
        self.get_client_data(client_id).await?;

        let mut connections = self
            .get_domain(
                format!(
                    "ibc/ics02-client/clients/{}/connections",
                    hex::encode(client_id.as_bytes())
                )
                .into(),
            )
            .await?
            .unwrap_or(ClientConnections::default());

        connections.connection_ids.push(connection_id.clone());

        self.put_domain(
            format!(
                "ibc/ics02-client/clients/{}/connections",
                hex::encode(client_id.as_bytes())
            )
            .into(),
            connections,
        )
        .await;

        Ok(())
    }
}

impl<T: StateExt + Send + Sync> View for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use ibc_proto::ibc::core::client::v1::MsgCreateClient as RawMsgCreateClient;
    use ibc_proto::ibc::core::client::v1::MsgUpdateClient as RawMsgUpdateClient;
    use penumbra_crypto::merkle;
    use penumbra_crypto::{Fq, Zero};
    use penumbra_proto::ibc::ibc_action::Action as IbcActionInner;
    use penumbra_proto::Message;
    use penumbra_storage::Storage;
    use penumbra_transaction::{Action, Fee, Transaction, TransactionBody};
    use std::fs;
    use tempfile::tempdir;
    use tendermint::Time;

    // test that we can create and update a light client.
    #[tokio::test]
    async fn test_create_and_update_light_client() {
        // create a storage backend for testing
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("ibc-testing.db");

        let storage = Storage::load(file_path).await.unwrap();
        let state = storage.state().await.unwrap();

        let mut client_component = ClientComponent::new(state).await;

        // init chain should result in client counter = 0
        let genesis_state = genesis::AppState::default();
        let timestamp = Time::parse_from_rfc3339("2022-02-11T17:30:50.425417198Z").unwrap();
        client_component.state.put_block_timestamp(timestamp).await;
        client_component.state.put_block_height(0).await;
        client_component.init_chain(&genesis_state).await;

        assert_eq!(client_component.state.client_counter().await.unwrap().0, 0);

        // base64 encoded MsgCreateClient that was used to create the currently in-use Stargaze
        // light client on the cosmos hub:
        // https://cosmos.bigdipper.live/transactions/13C1ECC54F088473E2925AD497DDCC092101ADE420BC64BADE67D34A75769CE9
        //
        //
        let msg_create_client_stargaze_raw = base64::decode(
            fs::read_to_string("../ibc/test/create_client.msg")
                .unwrap()
                .replace('\n', ""),
        )
        .unwrap();
        let msg_create_stargaze_client =
            RawMsgCreateClient::decode(msg_create_client_stargaze_raw.as_slice()).unwrap();

        // base64 encoded MsgUpdateClient that was used to issue the first update to the in-use stargaze light client on the cosmos hub:
        // https://cosmos.bigdipper.live/transactions/24F1E19F218CAF5CA41D6E0B653E85EB965843B1F3615A6CD7BCF336E6B0E707
        let msg_update_client_stargaze_raw = base64::decode(
            fs::read_to_string("../ibc/test/update_client_1.msg")
                .unwrap()
                .replace('\n', ""),
        )
        .unwrap();
        let mut msg_update_stargaze_client =
            RawMsgUpdateClient::decode(msg_update_client_stargaze_raw.as_slice()).unwrap();
        msg_update_stargaze_client.client_id = "07-tendermint-0".to_string();

        let create_client_action = IbcAction {
            action: Some(IbcActionInner::CreateClient(msg_create_stargaze_client)),
        };
        let create_client_tx = Transaction {
            transaction_body: TransactionBody {
                actions: vec![Action::IBCAction(create_client_action)],
                merkle_root: merkle::Root(Fq::zero()),
                expiry_height: 0,
                chain_id: "".to_string(),
                fee: Fee(0),
            },
            binding_sig: [0u8; 64].into(),
        };

        let update_client_action = IbcAction {
            action: Some(IbcActionInner::UpdateClient(msg_update_stargaze_client)),
        };
        let update_client_tx = Transaction {
            transaction_body: TransactionBody {
                actions: vec![Action::IBCAction(update_client_action)],
                merkle_root: merkle::Root(Fq::zero()),
                expiry_height: 0,
                chain_id: "".to_string(),
                fee: Fee(0),
            },
            binding_sig: [0u8; 64].into(),
        };

        ClientComponent::check_tx_stateless(&create_client_tx).unwrap();
        client_component
            .check_tx_stateful(&create_client_tx)
            .await
            .unwrap();
        // execute (save client)
        client_component.execute_tx(&create_client_tx).await;

        assert_eq!(client_component.state.client_counter().await.unwrap().0, 1);

        // now try update client

        ClientComponent::check_tx_stateless(&update_client_tx).unwrap();
        // verify the ClientUpdate proof
        client_component
            .check_tx_stateful(&update_client_tx)
            .await
            .unwrap();
        // save the next tm state
        client_component.execute_tx(&update_client_tx).await;

        // try one more client update
        // https://cosmos.bigdipper.live/transactions/ED217D360F51E622859F7B783FEF98BDE3544AA32BBD13C6C77D8D0D57A19FFD
        let msg_update_second = base64::decode(
            fs::read_to_string("../ibc/test/update_client_2.msg")
                .unwrap()
                .replace('\n', ""),
        )
        .unwrap();

        let mut second_update = RawMsgUpdateClient::decode(msg_update_second.as_slice()).unwrap();
        second_update.client_id = "07-tendermint-0".to_string();
        let second_update_client_action = IbcAction {
            action: Some(IbcActionInner::UpdateClient(second_update)),
        };
        let second_update_client_tx = Transaction {
            transaction_body: TransactionBody {
                actions: vec![Action::IBCAction(second_update_client_action)],
                merkle_root: merkle::Root(Fq::zero()),
                expiry_height: 0,
                chain_id: "".to_string(),
                fee: Fee(0),
            },
            binding_sig: [0u8; 64].into(),
        };

        ClientComponent::check_tx_stateless(&second_update_client_tx).unwrap();
        // verify the ClientUpdate proof
        client_component
            .check_tx_stateful(&second_update_client_tx)
            .await
            .unwrap();
        // save the next tm state
        client_component.execute_tx(&second_update_client_tx).await;
    }
}
