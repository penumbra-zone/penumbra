use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ibc_types::core::{
    ics02_client::{client_state::ClientState, consensus_state::ConsensusState},
    ics03_connection::{
        connection::ConnectionEnd,
        connection::{Counterparty, State as ConnectionState},
        msgs::conn_open_try::MsgConnectionOpenTry,
        version::pick_version,
    },
    ics24_host::{
        identifier::ConnectionId,
        path::{ClientConsensusStatePath, ClientStatePath, ConnectionPath},
    },
};
use penumbra_chain::StateReadExt as _;
use penumbra_storage::{StateRead, StateWrite};
use penumbra_transaction::Transaction;

use crate::{
    action_handler::ActionHandler,
    ibc::{
        component::{
            client::StateReadExt as _,
            connection::{StateReadExt as _, StateWriteExt as _},
        },
        event, validate_penumbra_client_state, SUPPORTED_VERSIONS,
    },
};
use ibc_types::Height as IBCHeight;

#[async_trait]
impl ActionHandler for MsgConnectionOpenTry {
    async fn check_stateless(&self, _context: Arc<Transaction>) -> Result<()> {
        // basic checks are performed by the ibc-rs crate when deserializing domain types.
        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, _state: Arc<S>) -> Result<()> {
        // IBC actions merge check_stateful and execute.
        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        tracing::debug!(msg = ?self);

        // Validate a ConnectionOpenTry message, which is sent to us by a counterparty chain that
        // has committed a Connection to us in an INIT state on its chain. Before executing a
        // ConnectionOpenTry message, we have no knowledge about the connection: our counterparty
        // is in INIT state, and we are in none state. After executing ConnectionOpenTry, our
        // counterparty is in INIT state, and we are in TRYOPEN state.
        //
        // In order to verify a ConnectionOpenTry, we need to check that the counterparty chain has
        // committed a _valid_ Penumbra consensus state, that the counterparty chain has committed
        // the expected Connection to its state (in the INIT state), and that the counterparty has
        // committed a correct Penumbra client state to its state.
        //
        // Here we are Chain B.
        // CHAINS:          (A, B)
        // PRIOR STATE:     (INIT, none)
        // POSTERIOR STATE: (INIT, TRYOPEN)
        // verify that the consensus height is correct

        consensus_height_is_correct(&mut state, self).await?;

        // verify that the client state (which is a Penumbra client) is well-formed for a
        // penumbra client.
        penumbra_client_state_is_well_formed(&mut state, self).await?;

        // TODO(erwan): how to handle this with ibc-rs@0.23.0?
        // if this msg provides a previous_connection_id to resume from, then check that the
        // provided previous connection ID is valid
        // let previous_connection = self.check_previous_connection(msg).await?;

        // perform version intersection
        // let supported_versions = previous_connection
        //     .map(|c| c.versions().to_vec())
        //     .unwrap_or_else(|| SUPPORTED_VERSIONS.clone());
        let supported_versions = SUPPORTED_VERSIONS.clone();

        pick_version(&supported_versions, &self.versions_on_a.clone())?;

        // expected_conn is the conn that we expect to have been committed to on the counterparty
        // chain
        let expected_conn = ConnectionEnd::new(
            ConnectionState::Init,
            self.counterparty.client_id().clone(),
            Counterparty::new(
                self.counterparty.client_id().clone(),
                None,
                penumbra_chain::PENUMBRA_COMMITMENT_PREFIX.clone(),
            ),
            self.versions_on_a.clone(),
            self.delay_period,
        );

        // get the stored client state for the counterparty
        let trusted_client_state = state
            .get_client_state(self.counterparty.client_id())
            .await?;

        // check if the client is frozen
        // TODO: should we also check if the client is expired here?
        if trusted_client_state.is_frozen() {
            return Err(anyhow::anyhow!("client is frozen"));
        }

        // get the stored consensus state for the counterparty
        let trusted_consensus_state = state
            .get_verified_consensus_state(
                self.proofs_height_on_a,
                self.counterparty.client_id().clone(),
            )
            .await?;

        // PROOF VERIFICATION
        // 1. verify that the counterparty chain committed the expected_conn to its state
        trusted_client_state.verify_connection_state(
            self.proofs_height_on_a,
            self.counterparty.prefix(),
            &self.proof_conn_end_on_a,
            trusted_consensus_state.root(),
            &ConnectionPath::new(
                self.counterparty
                    .connection_id
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("counterparty connection id is not set"))?,
            ),
            &expected_conn,
        )?;

        // 2. verify that the counterparty chain committed the correct ClientState (that was
        //    provided in the msg)
        trusted_client_state.verify_client_full_state(
            self.proofs_height_on_a,
            self.counterparty.prefix(),
            &self.proof_client_state_of_b_on_a,
            trusted_consensus_state.root(),
            &ClientStatePath::new(self.counterparty.client_id()),
            self.client_state_of_b_on_a.clone(),
        )?;

        let expected_consensus = state
            .get_penumbra_consensus_state(self.consensus_height_of_b_on_a)
            .await?;

        // 3. verify that the counterparty chain stored the correct consensus state of Penumbra at
        //    the given consensus height
        trusted_client_state.verify_client_consensus_state(
            self.proofs_height_on_a,
            self.counterparty.prefix(),
            &self.proof_consensus_state_of_b_on_a,
            trusted_consensus_state.root(),
            &ClientConsensusStatePath::new(
                self.counterparty.client_id(),
                &self.consensus_height_of_b_on_a,
            ),
            &expected_consensus,
        )?;

        // VALIDATION SUCCESSSFUL, now execute
        //
        // new_conn is the new connection that we will open on this chain
        let mut new_conn = ConnectionEnd::new(
            ConnectionState::TryOpen,
            self.client_id_on_b.clone(),
            self.counterparty.clone(),
            self.versions_on_a.clone(),
            self.delay_period,
        );
        new_conn.set_version(
            pick_version(&SUPPORTED_VERSIONS.to_vec(), &self.versions_on_a.clone()).unwrap(),
        );

        let new_connection_id = ConnectionId::new(state.get_connection_counter().await.unwrap().0);

        state
            .put_new_connection(&new_connection_id, new_conn)
            .await
            .unwrap();

        state.record(event::connection_open_try(
            &new_connection_id,
            &self.client_id_on_b,
            &self.counterparty,
        ));

        Ok(())
    }
}
async fn consensus_height_is_correct<S: StateRead>(
    state: S,
    msg: &MsgConnectionOpenTry,
) -> anyhow::Result<()> {
    let current_height = IBCHeight::new(0, state.get_block_height().await?)?;
    if msg.consensus_height_of_b_on_a > current_height {
        return Err(anyhow::anyhow!(
            "consensus height is greater than the current block height",
        ));
    }

    Ok(())
}
async fn penumbra_client_state_is_well_formed<S: StateRead>(
    state: S,
    msg: &MsgConnectionOpenTry,
) -> anyhow::Result<()> {
    let height = state.get_block_height().await?;
    let chain_id = state.get_chain_id().await?;
    validate_penumbra_client_state(msg.client_state_of_b_on_a.clone(), &chain_id, height)?;

    Ok(())
}
