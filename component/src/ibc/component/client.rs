use crate::ibc::client::ics02_validation;
use crate::Component;
use anyhow::Result;
use async_trait::async_trait;
use ibc::clients::ics07_tendermint;

use ibc::{
    clients::ics07_tendermint::{
        client_state::ClientState as TendermintClientState,
        consensus_state::ConsensusState as TendermintConsensusState,
        header::Header as TendermintHeader,
    },
    core::{
        ics02_client::{
            client_state::ClientState,
            client_type::ClientType,
            consensus_state::ConsensusState,
            height::Height,
            msgs::{create_client::MsgCreateClient, update_client::MsgUpdateClient},
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

use crate::ibc::{event, ClientCounter, VerifiedHeights};

use super::state_key;

pub(crate) mod stateful;
pub(crate) mod stateless;

// TODO(erwan): remove before opening PR
// + replace concrete types with trait objects
// + evaluate how to make penumbra_proto::Protobuf more friendly with the erased protobuf traits that
//   underpins the ics02 traits
// + ADR004 defers LC state/consensus deserialization later, maybe we should have a preprocessing step before execution
// . to distinguish I/O errors from actual execution errors. It would also make things a little less boilerplaty.
// +

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
        let height = Height::new(revision_number, begin_block.header.height.into())
            .expect("block height cannot be zero");

        state.put_penumbra_consensus_state(height, cs);
    }

    #[instrument(name = "ics2_client", skip(_state, _end_block))]
    async fn end_block(_state: &mut StateTransaction, _end_block: &abci::request::EndBlock) {}
}

#[async_trait]
pub(crate) trait Ics2ClientExt: StateWrite {
    // execute a UpdateClient IBC action. this assumes that the UpdateClient has already been
    // validated, including header verification.
    async fn execute_update_client(&mut self, msg_update_client: &MsgUpdateClient) -> Result<()> {
        // TODO(erwan): deferred client state deserialization means `execute_update_client` is faillible
        // see ibc-rs ADR004: https://github.com/cosmos/ibc-rs/blob/main/docs/architecture/adr-004-light-client-crates-extraction.md#light-client-specific-code
        let tm_header = ics02_validation::get_tendermint_header(msg_update_client.header.clone())?;

        // get the latest client state
        let client_state = self
            .get_client_state(&msg_update_client.client_id)
            .await
            .unwrap();

        let (next_tm_client_state, next_tm_consensus_state) = self
            .next_tendermint_state(
                msg_update_client.client_id.clone(),
                client_state.clone(),
                tm_header.clone(),
            )
            .await;

        // store the updated client and consensus states
        self.put_client(&msg_update_client.client_id, next_tm_client_state);
        self.put_verified_consensus_state(
            tm_header.height(),
            msg_update_client.client_id.clone(),
            next_tm_consensus_state,
        )
        .await
        .unwrap();

        self.record(event::update_client(
            msg_update_client.client_id.clone(),
            client_state,
            tm_header,
        ));
        Ok(())
    }

    // execute IBC CreateClient.
    //
    //  we compute the client's ID (a concatenation of a monotonically increasing integer, the
    //  number of clients on Penumbra, and the client type) and commit the following to our state:
    // - client type
    // - consensus state
    // - processed time and height
    async fn execute_create_client(&mut self, msg_create_client: &MsgCreateClient) -> Result<()> {
        tracing::info!("deserializing client state");
        // TODO(erwan): deferred client state deserialization means `execute_create_client` is faillible
        // see ibc-rs ADR004: https://github.com/cosmos/ibc-rs/blob/main/docs/architecture/adr-004-light-client-crates-extraction.md#light-client-specific-code
        let client_state =
            ics02_validation::get_tendermint_client_state(msg_create_client.client_state.clone())?;

        // get the current client counter
        let id_counter = self.client_counter().await.unwrap();
        let client_id = ClientId::new(client_state.client_type(), id_counter.0).unwrap();

        tracing::info!("creating client {:?}", client_id);

        let consensus_state = ics02_validation::get_tendermint_consensus_state(
            msg_create_client.consensus_state.clone(),
        )?;

        // store the client data
        self.put_client(&client_id, client_state.clone());

        // store the genesis consensus state
        self.put_verified_consensus_state(
            client_state.latest_height(),
            client_id.clone(),
            consensus_state,
        )
        .await
        .unwrap();

        // increment client counter
        let counter = self.client_counter().await.unwrap_or(ClientCounter(0));
        self.put_client_counter(ClientCounter(counter.0 + 1));

        self.record(event::create_client(client_id, client_state));
        Ok(())
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
            if stored_cs_state == verified_consensus_state {
                return (trusted_client_state, verified_consensus_state);
            } else {
                return (
                    trusted_client_state
                        .with_header(verified_header.clone())
                        .unwrap()
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
            if verified_header.signed_header.header().time < prev_state.timestamp {
                return (
                    trusted_client_state
                        .with_header(verified_header.clone())
                        .unwrap()
                        .with_frozen_height(verified_header.height())
                        .unwrap(),
                    verified_consensus_state,
                );
            }
        }
        // case 2: if we have a verified consensus state with higher block height than this header,
        // verify that this header's timestamp is less than or equal to this header's timestamp.
        if let Some(next_state) = next_consensus_state {
            if verified_header.signed_header.header().time > next_state.timestamp {
                return (
                    trusted_client_state
                        .with_header(verified_header.clone())
                        .unwrap()
                        .with_frozen_height(verified_header.height())
                        .unwrap(),
                    verified_consensus_state,
                );
            }
        }

        (
            trusted_client_state
                .with_header(verified_header.clone())
                .unwrap(),
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

    fn put_client(&mut self, client_id: &ClientId, client_state: impl ClientState) {
        let client_state = client_state
            .as_any()
            .downcast_ref::<ics07_tendermint::client_state::ClientState>()
            .expect("not a tendermint client state")
            .to_owned();

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
    fn put_penumbra_consensus_state(
        &mut self,
        height: Height,
        consensus_state: impl ConsensusState,
    ) {
        // NOTE: this is an implementation detail of the Penumbra ICS2 implementation, so
        // it's not in the same path namespace.

        // let tm = consensus_state;
        let tm = consensus_state
            .as_any()
            .downcast_ref::<ics07_tendermint::consensus_state::ConsensusState>()
            .expect("not an tendermint consensus state")
            .to_owned();
        self.put(format!("penumbra_consensus_states/{}", height), tm);
    }

    async fn put_verified_consensus_state(
        &mut self,
        height: Height,
        client_id: ClientId,
        consensus_state: impl ConsensusState,
    ) -> Result<()> {
        let consensus_state = consensus_state
            .as_any()
            .downcast_ref::<ics07_tendermint::consensus_state::ConsensusState>()
            .expect("not a tendermint consensus state")
            .to_owned();
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
            ibc::Height::new(0, current_height)?,
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
        self.get_proto(&state_key::client_type(client_id))
            .await?
            .ok_or_else(|| anyhow::anyhow!("client not found"))
            .map(ClientType::new)
    }

    async fn get_client_state(&self, client_id: &ClientId) -> Result<TendermintClientState> {
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
    async fn get_penumbra_consensus_state(
        &self,
        height: Height,
    ) -> Result<TendermintConsensusState> {
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
    ) -> Result<TendermintConsensusState> {
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
    ) -> Result<Option<TendermintConsensusState>> {
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
    ) -> Result<Option<TendermintConsensusState>> {
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
    use crate::TempStorageExt;

    use super::*;
    use ibc_proto::protobuf::Protobuf;
    use penumbra_chain::StateWriteExt;
    use penumbra_proto::core::ibc::v1alpha1::IbcAction;
    use penumbra_storage::{ArcStateExt, TempStorage};
    use penumbra_transaction::Transaction;
    use std::str::FromStr;
    use tendermint::Time;

    // test that we can create and update a light client.
    #[tokio::test]
    async fn test_create_and_update_light_client() -> anyhow::Result<()> {
        // create a storage backend for testing
        let storage = TempStorage::new().await?.apply_default_genesis().await?;

        let mut state = Arc::new(storage.latest_state());

        // Light client verification is time-dependent.  In practice, the latest
        // (consensus) time will be delivered in each BeginBlock and written
        // into the state.  Here, set the block timestamp manually so it's
        // available to the unit test.
        let timestamp = Time::parse_from_rfc3339("2022-02-11T17:30:50.425417198Z")?;
        let mut state_tx = state.try_begin_transaction().unwrap();
        state_tx.put_block_timestamp(timestamp);
        // TODO(erwan): check that this is a correct assumption to make?
        //              the ibc::ics02::Height constructor forbids building `Height` with value zero.
        //              Semantically this seem to correspond to a blockchain that has not begun to produce blocks
        state_tx.put_block_height(1);
        state_tx.apply();

        // base64 encoded MsgCreateClient that was used to create the currently in-use Stargaze
        // light client on the cosmos hub:
        // https://cosmos.bigdipper.live/transactions/13C1ECC54F088473E2925AD497DDCC092101ADE420BC64BADE67D34A75769CE9
        let msg_create_client_stargaze_raw =
            base64::decode(include_str!("../../ibc/test/create_client.msg").replace('\n', ""))
                .unwrap();
        let msg_create_stargaze_client =
            MsgCreateClient::decode(msg_create_client_stargaze_raw.as_slice()).unwrap();

        // base64 encoded MsgUpdateClient that was used to issue the first update to the in-use stargaze light client on the cosmos hub:
        // https://cosmos.bigdipper.live/transactions/24F1E19F218CAF5CA41D6E0B653E85EB965843B1F3615A6CD7BCF336E6B0E707
        let msg_update_client_stargaze_raw =
            base64::decode(include_str!("../../ibc/test/update_client_1.msg").replace('\n', ""))
                .unwrap();
        let mut msg_update_stargaze_client =
            MsgUpdateClient::decode(msg_update_client_stargaze_raw.as_slice()).unwrap();

        msg_update_stargaze_client.client_id = ClientId::from_str("07-tendermint-0").unwrap();

        let create_client_action: IbcAction = msg_create_stargaze_client.into();
        let update_client_action: IbcAction = msg_update_stargaze_client.into();

        // The ActionHandler trait provides the transaction the action was part
        // of as context available during verification.  This is used, for instance,
        // to allow spend and output proofs to access the (transaction-wide) anchor.
        // Since the context is not used by the IBC action handlers, we can pass a dummy transaction.
        let dummy_context = Arc::new(Transaction::default());

        create_client_action
            .check_stateless(dummy_context.clone())
            .await?;
        create_client_action.check_stateful(state.clone()).await?;
        let mut state_tx = state.try_begin_transaction().unwrap();
        create_client_action.execute(&mut state_tx).await?;
        state_tx.apply();

        // Check that state reflects +1 client apps registered.
        assert_eq!(state.client_counter().await.unwrap().0, 1);

        // Now we update the client and confirm that the update landed in state.
        update_client_action
            .check_stateless(dummy_context.clone())
            .await?;
        update_client_action.check_stateful(state.clone()).await?;
        let mut state_tx = state.try_begin_transaction().unwrap();
        update_client_action.execute(&mut state_tx).await?;
        state_tx.apply();

        // We've had one client update, yes. What about second client update?
        // https://cosmos.bigdipper.live/transactions/ED217D360F51E622859F7B783FEF98BDE3544AA32BBD13C6C77D8D0D57A19FFD
        let msg_update_second =
            base64::decode(include_str!("../../ibc/test/update_client_2.msg").replace('\n', ""))
                .unwrap();

        let mut second_update = MsgUpdateClient::decode(msg_update_second.as_slice()).unwrap();
        second_update.client_id = ClientId::from_str("07-tendermint-0").unwrap();
        let second_update_client_action: IbcAction = second_update.into();

        second_update_client_action
            .check_stateless(dummy_context.clone())
            .await?;
        second_update_client_action
            .check_stateful(state.clone())
            .await?;
        let mut state_tx = state.try_begin_transaction().unwrap();
        second_update_client_action.execute(&mut state_tx).await?;
        state_tx.apply();

        Ok(())
    }
}
