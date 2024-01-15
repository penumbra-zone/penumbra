use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::ChainStateReadExt;
use ibc_types::core::{
    client::Height,
    connection::{events, msgs::MsgConnectionOpenAck, ConnectionEnd, Counterparty, State},
};
use ibc_types::lightclients::tendermint::client_state::ClientState as TendermintClientState;
use ibc_types::path::{ClientConsensusStatePath, ClientStatePath, ConnectionPath};

use crate::{
    component::{
        client::StateReadExt as _,
        connection::{StateReadExt as _, StateWriteExt as _},
        ics02_validation::validate_penumbra_client_state,
        proof_verification, MsgHandler,
    },
    IBC_COMMITMENT_PREFIX,
};

#[async_trait]
impl MsgHandler for MsgConnectionOpenAck {
    async fn check_stateless<H>(&self) -> Result<()> {
        Ok(())
    }

    async fn try_execute<S: StateWrite + ChainStateReadExt, H>(&self, mut state: S) -> Result<()> {
        tracing::debug!(msg = ?self);
        // Validate a ConnectionOpenAck message, which is sent to us by a counterparty chain that
        // has committed a Connection to us expected to be in the TRYOPEN state. Before executing a
        // ConnectionOpenAck, we must have a prior connection to this chain in the INIT state.
        //
        // In order to verify a ConnectionOpenAck, we need to check that the counterparty chain has
        // committed a _valid_ Penumbra consensus state, that the counterparty chain has committed
        // the expected Connection to its state (in the TRYOPEN state) with the expected version,
        // and that the counterparty has committed a correct Penumbra client state to its state.
        //
        // Here we are Chain A.
        // CHAINS:          (A, B)
        // PRIOR STATE:     (INIT, TRYOPEN)
        // POSTERIOR STATE: (OPEN, TRYOPEN)
        //
        // verify that the consensus height is correct
        consensus_height_is_correct(&state, self).await?;

        // verify that the client state is well formed
        penumbra_client_state_is_well_formed(&state, self).await?;

        // verify the previous connection that we're ACKing is in the correct state
        let connection = verify_previous_connection(&state, self).await?;

        // verify that the counterparty committed a TRYOPEN connection with us as the
        // counterparty
        let expected_counterparty = Counterparty {
            client_id: connection.client_id.clone(), // client ID (local)
            connection_id: Some(self.conn_id_on_a.clone()), // connection ID (local)
            prefix: IBC_COMMITMENT_PREFIX.clone(),   // commitment prefix (local)
        };

        // the connection we expect the counterparty to have committed
        let expected_conn = ConnectionEnd {
            state: State::TryOpen,
            client_id: connection.counterparty.client_id.clone(),
            counterparty: expected_counterparty.clone(),
            versions: vec![self.version.clone()],
            delay_period: connection.delay_period,
        };

        // get the stored client state for the counterparty
        let trusted_client_state = state.get_client_state(&connection.client_id).await?;

        // check if the client is frozen
        // TODO: should we also check if the client is expired here?
        if trusted_client_state.is_frozen() {
            anyhow::bail!("client is frozen");
        }

        // get the stored consensus state for the counterparty
        let trusted_consensus_state = state
            .get_verified_consensus_state(&self.proofs_height_on_b, &connection.client_id)
            .await?;

        // PROOF VERIFICATION
        // 1. verify that the counterparty chain committed the expected_conn to its state
        tracing::debug!(?trusted_client_state,);
        tracing::debug!(
            msg.proofs_height_on_b = ?self.proofs_height_on_b,
        );
        tracing::debug!(
            counterparty_prefix = ?connection.counterparty.prefix,
        );
        tracing::debug!(
            msg.proof_conn_end_on_b = ?self.proof_conn_end_on_b,
        );
        tracing::debug!(
            trusted_consensus_state_root = ?trusted_consensus_state.root,
        );
        tracing::debug!(
            connection_path = %ConnectionPath::new(&self.conn_id_on_b),
        );
        tracing::debug!(
            expected_conn = ?expected_conn,
        );
        let conn_end_on_b_proof = self.proof_conn_end_on_b.clone();
        proof_verification::verify_connection_state(
            &trusted_client_state,
            self.proofs_height_on_b,
            &connection.counterparty.prefix,
            &conn_end_on_b_proof,
            &trusted_consensus_state.root,
            &ConnectionPath::new(&self.conn_id_on_b),
            &expected_conn,
        )
        .context("couldn't verify connection state")?;

        // 2. verify that the counterparty chain committed the correct ClientState (that was
        //    provided in the msg)
        let proof_client_state_of_a_on_b = self.proof_client_state_of_a_on_b.clone();
        let client_state_of_a_on_b: TendermintClientState =
            self.client_state_of_a_on_b.clone().try_into()?;

        proof_verification::verify_client_full_state(
            &trusted_client_state,
            self.proofs_height_on_b,
            &connection.counterparty.prefix,
            &proof_client_state_of_a_on_b,
            &trusted_consensus_state.root,
            &ClientStatePath::new(&connection.counterparty.client_id),
            client_state_of_a_on_b,
        )
        .context("couldn't verify client state")?;

        let expected_consensus = state
            .get_penumbra_consensus_state(self.consensus_height_of_a_on_b)
            .await?;

        // 3. verify that the counterparty chain stored the correct consensus state of Penumbra at
        //    the given consensus height
        let proof_consensus_state_of_a_on_b = self.proof_consensus_state_of_a_on_b.clone();
        proof_verification::verify_client_consensus_state(
            &trusted_client_state,
            self.proofs_height_on_b,
            &connection.counterparty.prefix,
            &proof_consensus_state_of_a_on_b,
            &trusted_consensus_state.root,
            &ClientConsensusStatePath::new(
                &connection.counterparty.client_id,
                &self.consensus_height_of_a_on_b,
            ),
            expected_consensus,
        )
        .context("couldn't verify client consensus state")?;

        // VERIFICATION SUCCESSFUL. now execute

        let mut connection = state
            .get_connection(&self.conn_id_on_a)
            .await
            .context("should be able to get connection")?
            .context("missing connection")?;

        // TODO(erwan): reviewer should check that CP is correct pls
        let prev_counterparty = connection.counterparty;
        let counterparty = Counterparty {
            client_id: prev_counterparty.client_id.clone(),
            connection_id: Some(self.conn_id_on_b.clone()),
            prefix: prev_counterparty.prefix,
        };
        connection.state = State::Open;
        connection.versions = vec![self.version.clone()];
        connection.counterparty = counterparty;

        state.update_connection(&self.conn_id_on_a, connection.clone());

        state.record(
            events::ConnectionOpenAck {
                conn_id_on_a: self.conn_id_on_a.clone(),
                client_id_on_a: connection.client_id.clone(),
                conn_id_on_b: connection
                    .counterparty
                    .connection_id
                    .clone()
                    .unwrap_or_default(),
                client_id_on_b: connection.counterparty.client_id,
            }
            .into(),
        );

        Ok(())
    }
}
async fn consensus_height_is_correct<S: ChainStateReadExt>(
    state: &S,
    msg: &MsgConnectionOpenAck,
) -> anyhow::Result<()> {
    let current_height = Height::new(
        state.get_revision_number().await?,
        state.get_block_height().await?,
    )?;
    if msg.consensus_height_of_a_on_b >= current_height {
        anyhow::bail!("consensus height is greater than the current block height",);
    }

    Ok(())
}

async fn penumbra_client_state_is_well_formed<S: ChainStateReadExt>(
    state: &S,
    msg: &MsgConnectionOpenAck,
) -> anyhow::Result<()> {
    let height = state.get_block_height().await?;
    let chain_id = state.get_chain_id().await?;
    validate_penumbra_client_state(msg.client_state_of_a_on_b.clone(), &chain_id, height)?;

    Ok(())
}
async fn verify_previous_connection<S: StateRead>(
    state: S,
    msg: &MsgConnectionOpenAck,
) -> anyhow::Result<ConnectionEnd> {
    let connection = state
        .get_connection(&msg.conn_id_on_a)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "no connection with the specified ID {} exists",
                msg.conn_id_on_a
            )
        })?;

    // see
    // https://github.com/cosmos/ibc/blob/master/spec/core/ics-003-connection-semantics/README.md
    // for this validation logic
    let state_is_consistent = connection.state_matches(&State::Init)
        && connection.versions.contains(&msg.version)
        || connection.state_matches(&State::TryOpen)
            && connection.versions.get(0).eq(&Some(&msg.version));

    if !state_is_consistent {
        anyhow::bail!("connection is not in the correct state");
    } else {
        Ok(connection)
    }
}
