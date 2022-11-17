use std::str::FromStr;

use crate::Component;
use anyhow::Result;
use async_trait::async_trait;
use ibc::core::ics02_client::client_state::ClientState;
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
            client_state::AnyClientState,
            client_type::ClientType,
            header::AnyHeader,
            height::Height,
            msgs::{create_client::MsgCreateAnyClient, update_client::MsgUpdateAnyClient},
        },
        ics24_host::identifier::ClientId,
    },
};
use penumbra_chain::{genesis, StateReadExt as _};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateTransaction, StateWrite};
use tendermint::{abci, validator};
use tendermint_light_client_verifier::{
    types::{TrustedBlockState, UntrustedBlockState},
    ProdVerifier, Verdict, Verifier,
};
use tracing::instrument;

use crate::ibc::{event, ClientConnections, ClientCounter, VerifiedHeights};

use super::state_key;

pub(crate) mod stateful;
pub(crate) mod stateless;

/// The Penumbra IBC client component. Handles all client-related IBC actions: MsgCreateClient,
/// MsgUpdateClient, MsgUpgradeClient, and MsgSubmitMisbehaviour. The core responsibility of the
/// client component is tracking light clients for IBC, creating new light clients and verifying
/// state updates. Currently, only Tendermint light clients are supported.
pub struct Ics2Client {}

#[async_trait]
impl Component for Ics2Client {
    #[instrument(name = "ics2_client", skip(state, _app_state))]
    async fn init_chain(state: &mut StateTransaction, _app_state: &genesis::AppState) {
        // set the initial client count
        state.put_client_counter(ClientCounter(0));
    }

    #[instrument(name = "ics2_client", skip(state, begin_block))]
    async fn begin_block(state: &mut StateTransaction, begin_block: &abci::request::BeginBlock) {
        // In BeginBlock, we want to save a copy of our consensus state to our
        // own state tree, so that when we get a message from our
        // counterparties, we can verify that they are committing the correct
        // consensus states for us to their state tree.
        let cs = TendermintConsensusState::new(
            begin_block.header.app_hash.value().into(),
            begin_block.header.time,
            begin_block.header.next_validators_hash,
        );

        // Currently, we don't use a revision number, because we don't have
        // any further namespacing of blocks than the block height.
        let revision_number = 0;
        let height = Height::new(revision_number, begin_block.header.height.into());

        state.put_penumbra_consensus_state(height, AnyConsensusState::Tendermint(cs));
    }

    #[instrument(name = "ics2_client", skip(_state, _end_block))]
    async fn end_block(_state: &mut StateTransaction, _end_block: &abci::request::EndBlock) {}
}

#[async_trait]
pub(crate) trait Ics2ClientExt: StateWrite {
    // execute a UpdateClient IBC action. this assumes that the UpdateClient has already been
    // validated, including header verification.
    async fn execute_update_client(&mut self, msg_update_client: &MsgUpdateAnyClient) {
        // get the latest client state
        let client_state = self
            .get_client_state(&msg_update_client.client_id)
            .await
            .unwrap();

        let tm_client_state = match client_state.clone() {
            AnyClientState::Tendermint(tm_state) => tm_state,
            _ => panic!("unsupported client type"),
        };
        let tm_header = match msg_update_client.header.clone() {
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
        self.put_client(
            &msg_update_client.client_id,
            AnyClientState::Tendermint(next_tm_client_state),
        );
        self.put_verified_consensus_state(
            tm_header.height(),
            msg_update_client.client_id.clone(),
            AnyConsensusState::Tendermint(next_tm_consensus_state),
        )
        .await
        .unwrap();

        self.record(event::update_client(
            msg_update_client.client_id.clone(),
            client_state,
            msg_update_client.header.clone(),
        ));
    }

    // execute IBC CreateClient.
    //
    //  we compute the client's ID (a concatenation of a monotonically increasing integer, the
    //  number of clients on Penumbra, and the client type) and commit the following to our state:
    // - client type
    // - consensus state
    // - processed time and height
    async fn execute_create_client(&mut self, msg_create_client: &MsgCreateAnyClient) {
        // get the current client counter
        let id_counter = self.client_counter().await.unwrap();
        let client_id =
            ClientId::new(msg_create_client.client_state.client_type(), id_counter.0).unwrap();

        tracing::info!("creating client {:?}", client_id);

        // store the client data
        self.put_client(&client_id, msg_create_client.client_state.clone());

        // store the genesis consensus state
        self.put_verified_consensus_state(
            msg_create_client.client_state.latest_height(),
            client_id.clone(),
            msg_create_client.consensus_state.clone(),
        )
        .await
        .unwrap();

        // increment client counter
        let counter = self.client_counter().await.unwrap_or(ClientCounter(0));
        self.put_client_counter(ClientCounter(counter.0 + 1));

        self.record(event::create_client(
            client_id,
            msg_create_client.client_state.clone(),
        ));
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
        if let Ok(stored_cs_state) = self
            .get_verified_consensus_state(verified_header.height(), client_id.clone())
            .await
        {
            let stored_cs_state_tm = downcast!(stored_cs_state => AnyConsensusState::Tendermint)
                .ok_or_else(|| {
                    anyhow::anyhow!("stored consensus state is not a Tendermint consensus state")
                })
                .unwrap();
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
            .next_verified_consensus_state(&client_id, verified_header.height())
            .await
            .unwrap();
        let prev_consensus_state = self
            .prev_verified_consensus_state(&client_id, verified_header.height())
            .await
            .unwrap();

        // case 1: if we have a verified consensus state previous to this header, verify that this
        // header's timestamp is greater than or equal to the stored consensus state's timestamp
        if let Some(prev_state) = prev_consensus_state {
            let prev_state_tm = downcast!(prev_state => AnyConsensusState::Tendermint)
                .ok_or_else(|| {
                    anyhow::anyhow!("stored consensus state is not a Tendermint consensus state")
                })
                .unwrap();
            if verified_header.signed_header.header().time < prev_state_tm.timestamp {
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
            let next_state_tm = downcast!(next_state => AnyConsensusState::Tendermint)
                .ok_or_else(|| {
                    anyhow::anyhow!("stored consensus state is not a Tendermint consensus state")
                })
                .unwrap();
            if verified_header.signed_header.header().time > next_state_tm.timestamp {
                return (
                    trusted_client_state
                        .with_header(verified_header.clone())
                        .with_frozen_height(verified_header.height())
                        .unwrap(),
                    verified_consensus_state,
                );
            }
        }

        (
            trusted_client_state.with_header(verified_header.clone()),
            verified_consensus_state,
        )
    }
}

impl<T: StateWrite + ?Sized> Ics2ClientExt for T {}

#[async_trait]
pub trait StateWriteExt: StateWrite + StateReadExt {
    fn put_client_counter(&mut self, counter: ClientCounter) {
        self.put("ibc_client_counter".into(), counter);
    }

    fn put_client(&mut self, client_id: &ClientId, client_state: AnyClientState) {
        self.put_proto(
            state_key::client_type(client_id),
            client_state.client_type().as_str().to_string(),
        );

        self.put(state_key::client_state(client_id), client_state);
    }

    fn put_verified_heights(&mut self, client_id: &ClientId, verified_heights: VerifiedHeights) {
        self.put(
            format!(
                // NOTE: this is an implementation detail of the Penumbra ICS2 implementation, so
                // it's not in the same path namespace.
                "penumbra_verified_heights/{}/verified_heights",
                client_id
            ),
            verified_heights,
        );
    }

    // returns the ConsensusState for the penumbra chain (this chain) at the given height
    fn put_penumbra_consensus_state(&mut self, height: Height, consensus_state: AnyConsensusState) {
        // NOTE: this is an implementation detail of the Penumbra ICS2 implementation, so
        // it's not in the same path namespace.
        self.put(
            format!("penumbra_consensus_states/{}", height),
            consensus_state,
        );
    }

    async fn put_verified_consensus_state(
        &mut self,
        height: Height,
        client_id: ClientId,
        consensus_state: AnyConsensusState,
    ) -> Result<()> {
        self.put(
            state_key::verified_client_consensus_state(&client_id, &height),
            consensus_state,
        );

        let current_height = self.get_block_height().await?;
        let current_time: ibc::timestamp::Timestamp = self.get_block_timestamp().await?.into();

        self.put_proto::<u64>(
            state_key::client_processed_times(&client_id, &height),
            current_time.nanoseconds(),
        );

        self.put(
            state_key::client_processed_heights(&client_id, &height),
            ibc::Height::zero().with_revision_height(current_height),
        );

        // update verified heights
        let mut verified_heights =
            self.get_verified_heights(&client_id)
                .await?
                .unwrap_or(VerifiedHeights {
                    heights: Vec::new(),
                });

        verified_heights.heights.push(height);

        self.put_verified_heights(&client_id, verified_heights);

        Ok(())
    }

    // adds the provided connection ID to the client identified by client_id. returns an error if
    // the client does not exist.
    async fn add_connection_to_client(
        &mut self,
        client_id: &ClientId,
        connection_id: &ConnectionId,
    ) -> Result<()> {
        self.get_client_state(client_id).await?;
        self.get_client_type(client_id).await?;

        let mut connections: ClientConnections = self
            .get(&state_key::client_connections(client_id))
            .await?
            .unwrap_or_default();

        connections.connection_ids.push(connection_id.clone());

        self.put(state_key::client_connections(client_id), connections);

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn client_counter(&self) -> Result<ClientCounter> {
        self.get("ibc_client_counter")
            .await
            .map(|counter| counter.unwrap_or(ClientCounter(0)))
    }

    async fn get_client_type(&self, client_id: &ClientId) -> Result<ClientType> {
        let client_type_str: String = self
            .get_proto(&state_key::client_type(client_id))
            .await?
            .ok_or_else(|| anyhow::anyhow!("client not found"))?;

        ClientType::from_str(&client_type_str).map_err(|_| anyhow::anyhow!("invalid client type"))
    }

    async fn get_client_state(&self, client_id: &ClientId) -> Result<AnyClientState> {
        let client_state = self.get(&state_key::client_state(client_id)).await?;

        client_state.ok_or_else(|| anyhow::anyhow!("client not found"))
    }

    async fn get_verified_heights(&self, client_id: &ClientId) -> Result<Option<VerifiedHeights>> {
        self.get(&format!(
            // NOTE: this is an implementation detail of the Penumbra ICS2 implementation, so
            // it's not in the same path namespace.
            "penumbra_verified_heights/{}/verified_heights",
            client_id
        ))
        .await
    }

    // returns the ConsensusState for the penumbra chain (this chain) at the given height
    async fn get_penumbra_consensus_state(&self, height: Height) -> Result<AnyConsensusState> {
        // NOTE: this is an implementation detail of the Penumbra ICS2 implementation, so
        // it's not in the same path namespace.
        self.get(&format!("penumbra_consensus_states/{}", height))
            .await?
            .ok_or_else(|| anyhow::anyhow!("consensus state not found"))
    }

    async fn get_verified_consensus_state(
        &self,
        height: Height,
        client_id: ClientId,
    ) -> Result<AnyConsensusState> {
        self.get(&state_key::verified_client_consensus_state(
            &client_id, &height,
        ))
        .await?
        .ok_or_else(|| anyhow::anyhow!("consensus state not found"))
    }

    async fn get_client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<ibc::Height> {
        self.get(&state_key::client_processed_heights(client_id, height))
            .await?
            .ok_or_else(|| anyhow::anyhow!("client update time not found"))
    }

    async fn get_client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<ibc::timestamp::Timestamp> {
        let timestamp_nanos = self
            .get_proto::<u64>(&state_key::client_processed_times(client_id, height))
            .await?
            .ok_or_else(|| anyhow::anyhow!("client update time not found"))?;

        ibc::timestamp::Timestamp::from_nanoseconds(timestamp_nanos)
            .map_err(|_| anyhow::anyhow!("invalid client update time"))
    }

    // returns the lowest verified consensus state that is higher than the given height, if it
    // exists.
    async fn next_verified_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Option<AnyConsensusState>> {
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
    ) -> Result<Option<AnyConsensusState>> {
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
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::action_handler::ActionHandler;

    use super::*;
    use ibc_proto::ibc::core::client::v1::MsgCreateClient as RawMsgCreateClient;
    use ibc_proto::ibc::core::client::v1::MsgUpdateClient as RawMsgUpdateClient;
    use penumbra_chain::StateWriteExt as _;
    use penumbra_proto::core::ibc::v1alpha1::{ibc_action::Action as IbcActionInner, IbcAction};
    use penumbra_proto::Message;
    use penumbra_storage::Storage;
    use penumbra_tct as tct;
    use penumbra_transaction::{Action, Transaction, TransactionBody};
    use tempfile::tempdir;
    use tendermint::Time;

    // test that we can create and update a light client.
    #[tokio::test]
    async fn test_create_and_update_light_client() {
        // create a storage backend for testing
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("ibc-testing.db");

        let storage = Storage::load(file_path.clone()).await.unwrap();
        let mut state = Arc::new(storage.latest_state());
        let state_mut =
            Arc::get_mut(&mut state).expect("state Arc should not be referenced elsewhere");
        let mut state_tx = state_mut.begin_transaction();

        // init chain should result in client counter = 0
        let genesis_state = genesis::AppState::default();
        let timestamp = Time::parse_from_rfc3339("2022-02-11T17:30:50.425417198Z").unwrap();
        state_tx.put_block_timestamp(timestamp);
        state_tx.put_block_height(0);
        Ics2Client::init_chain(&mut state_tx, &genesis_state).await;

        state_tx.apply();
        storage
            .commit(Arc::try_unwrap(state).unwrap())
            .await
            .unwrap();

        let mut state = Arc::new(storage.latest_state());

        // base64 encoded MsgCreateClient that was used to create the currently in-use Stargaze
        // light client on the cosmos hub:
        // https://cosmos.bigdipper.live/transactions/13C1ECC54F088473E2925AD497DDCC092101ADE420BC64BADE67D34A75769CE9
        //
        //
        let msg_create_client_stargaze_raw =
            base64::decode(include_str!("../../ibc/test/create_client.msg").replace('\n', ""))
                .unwrap();
        let msg_create_stargaze_client =
            RawMsgCreateClient::decode(msg_create_client_stargaze_raw.as_slice()).unwrap();

        // base64 encoded MsgUpdateClient that was used to issue the first update to the in-use stargaze light client on the cosmos hub:
        // https://cosmos.bigdipper.live/transactions/24F1E19F218CAF5CA41D6E0B653E85EB965843B1F3615A6CD7BCF336E6B0E707
        let msg_update_client_stargaze_raw =
            base64::decode(include_str!("../../ibc/test/update_client_1.msg").replace('\n', ""))
                .unwrap();
        let mut msg_update_stargaze_client =
            RawMsgUpdateClient::decode(msg_update_client_stargaze_raw.as_slice()).unwrap();
        msg_update_stargaze_client.client_id = "07-tendermint-0".to_string();

        let create_client_action = IbcAction {
            action: Some(IbcActionInner::CreateClient(msg_create_stargaze_client)),
        };
        let create_client_tx = Arc::new(Transaction {
            transaction_body: TransactionBody {
                actions: vec![Action::IBCAction(create_client_action)],
                expiry_height: 0,
                chain_id: "".to_string(),
                fee: Default::default(),
                fmd_clues: vec![],
                memo: None,
            },
            anchor: tct::Tree::new().root(),
            binding_sig: [0u8; 64].into(),
        });

        let update_client_action = IbcAction {
            action: Some(IbcActionInner::UpdateClient(msg_update_stargaze_client)),
        };
        let update_client_tx = Arc::new(Transaction {
            transaction_body: TransactionBody {
                actions: vec![Action::IBCAction(update_client_action)],
                expiry_height: 0,
                chain_id: "".to_string(),
                fee: Default::default(),
                fmd_clues: vec![],
                memo: None,
            },
            binding_sig: [0u8; 64].into(),
            anchor: tct::Tree::new().root(),
        });

        create_client_tx
            .check_stateless(create_client_tx.clone())
            .unwrap();
        create_client_tx
            .check_stateful(state.clone(), create_client_tx.clone())
            .await
            .unwrap();
        // execute (save client)
        let state_mut =
            Arc::get_mut(&mut state).expect("state Arc should not be referenced elsewhere");
        let mut state_tx = state_mut.begin_transaction();
        create_client_tx.execute(&mut state_tx).await.unwrap();
        state_tx.apply();
        storage
            .commit(Arc::try_unwrap(state).unwrap())
            .await
            .unwrap();

        let mut state = Arc::new(storage.latest_state());
        assert_eq!(state.clone().client_counter().await.unwrap().0, 1);

        // now try update client

        update_client_tx
            .check_stateless(update_client_tx.clone())
            .unwrap();
        // verify the ClientUpdate proof
        update_client_tx
            .check_stateful(state.clone(), update_client_tx.clone())
            .await
            .unwrap();
        // save the next tm state
        let state_mut =
            Arc::get_mut(&mut state).expect("state Arc should not be referenced elsewhere");
        let mut state_tx = state_mut.begin_transaction();
        update_client_tx.execute(&mut state_tx).await.unwrap();
        state_tx.apply();
        storage
            .commit(Arc::try_unwrap(state).unwrap())
            .await
            .unwrap();

        let mut state = Arc::new(storage.latest_state());

        // try one more client update
        // https://cosmos.bigdipper.live/transactions/ED217D360F51E622859F7B783FEF98BDE3544AA32BBD13C6C77D8D0D57A19FFD
        let msg_update_second =
            base64::decode(include_str!("../../ibc/test/update_client_2.msg").replace('\n', ""))
                .unwrap();

        let mut second_update = RawMsgUpdateClient::decode(msg_update_second.as_slice()).unwrap();
        second_update.client_id = "07-tendermint-0".to_string();
        let second_update_client_action = IbcAction {
            action: Some(IbcActionInner::UpdateClient(second_update)),
        };
        let second_update_client_tx = Arc::new(Transaction {
            transaction_body: TransactionBody {
                actions: vec![Action::IBCAction(second_update_client_action)],
                expiry_height: 0,
                chain_id: "".to_string(),
                fee: Default::default(),
                fmd_clues: vec![],
                memo: None,
            },
            anchor: tct::Tree::new().root(),
            binding_sig: [0u8; 64].into(),
        });

        second_update_client_tx
            .check_stateless(second_update_client_tx.clone())
            .unwrap();
        // verify the ClientUpdate proof
        second_update_client_tx
            .check_stateful(state.clone(), second_update_client_tx.clone())
            .await
            .unwrap();
        // save the next tm state
        let state_mut =
            Arc::get_mut(&mut state).expect("state Arc should not be referenced elsewhere");
        let mut state_tx = state_mut.begin_transaction();
        second_update_client_tx
            .execute(&mut state_tx)
            .await
            .unwrap();
        state_tx.apply();
        storage
            .commit(Arc::try_unwrap(state).unwrap())
            .await
            .unwrap();
    }
}
