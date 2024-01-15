use crate::component::proof_verification;
use crate::version::pick_connection_version;
use crate::IBC_COMMITMENT_PREFIX;
use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ChainStateReadExt;
use ibc_types::lightclients::tendermint::client_state::ClientState as TendermintClientState;
use ibc_types::path::{ClientConsensusStatePath, ClientStatePath, ConnectionPath};
use ibc_types::{
    core::client::Height as IBCHeight,
    core::connection::{
        events, msgs::MsgConnectionOpenTry, ConnectionEnd, ConnectionId, Counterparty,
        State as ConnectionState,
    },
};

use crate::component::{
    client::StateReadExt as _,
    connection::{StateReadExt as _, StateWriteExt as _},
    connection_counter::SUPPORTED_VERSIONS,
    ics02_validation::validate_penumbra_client_state,
    MsgHandler,
};

#[async_trait]
impl MsgHandler for MsgConnectionOpenTry {
    async fn check_stateless<H>(&self) -> Result<()> {
        // basic checks are performed by the ibc-rs crate when deserializing domain types.
        Ok(())
    }

    async fn try_execute<S: StateWrite + ChainStateReadExt, H>(&self, mut state: S) -> Result<()> {
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

        consensus_height_is_correct(&state, self).await?;

        // verify that the client state (which is a Penumbra client) is well-formed for a
        // penumbra client.
        penumbra_client_state_is_well_formed(&state, self).await?;

        // TODO(erwan): how to handle this with ibc-rs@0.23.0?
        // if this msg provides a previous_connection_id to resume from, then check that the
        // provided previous connection ID is valid
        // let previous_connection = self.check_previous_connection(msg).await?;

        // perform version intersection
        // let supported_versions = previous_connection
        //     .map(|c| c.versions().to_vec())
        //     .unwrap_or_else(|| SUPPORTED_VERSIONS.clone());
        let supported_versions = SUPPORTED_VERSIONS.clone();

        pick_connection_version(&supported_versions, &self.versions_on_a.clone())?;

        // expected_conn is the conn that we expect to have been committed to on the counterparty
        // chain
        let expected_conn = ConnectionEnd {
            state: ConnectionState::Init,
            client_id: self.counterparty.client_id.clone(),
            counterparty: Counterparty {
                client_id: self.client_id_on_b.clone(),
                connection_id: None,
                prefix: IBC_COMMITMENT_PREFIX.clone(),
            },
            versions: self.versions_on_a.clone(),
            delay_period: self.delay_period,
        };

        // get the stored client state for the counterparty
        let trusted_client_state = state.get_client_state(&self.client_id_on_b).await?;

        // check if the client is frozen
        // TODO: should we also check if the client is expired here?
        if trusted_client_state.is_frozen() {
            anyhow::bail!("client is frozen");
        }

        // get the stored consensus state for the counterparty
        let trusted_consensus_state = state
            .get_verified_consensus_state(&self.proofs_height_on_a, &self.client_id_on_b)
            .await?;

        // PROOF VERIFICATION
        // 1. verify that the counterparty chain committed the expected_conn to its state
        let proof_conn_end_on_a = self.proof_conn_end_on_a.clone();
        proof_verification::verify_connection_state(
            &trusted_client_state,
            self.proofs_height_on_a,
            &self.counterparty.prefix,
            &proof_conn_end_on_a,
            &trusted_consensus_state.root,
            &ConnectionPath::new(
                self.counterparty
                    .connection_id
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("counterparty connection id is not set"))?,
            ),
            &expected_conn,
        )
        .context("failed to verify connection state")?;

        // 2. verify that the counterparty chain committed the correct ClientState (that was
        //    provided in the msg)
        let proof_client_state_of_b_on_a = self.proof_client_state_of_b_on_a.clone();

        let client_state_of_b_on_a: TendermintClientState =
            self.client_state_of_b_on_a.clone().try_into()?;

        proof_verification::verify_client_full_state(
            &trusted_client_state,
            self.proofs_height_on_a,
            &self.counterparty.prefix,
            &proof_client_state_of_b_on_a,
            &trusted_consensus_state.root,
            &ClientStatePath::new(&self.counterparty.client_id),
            client_state_of_b_on_a,
        )
        .context("couldn't verify client state")?;

        let expected_consensus = state
            .get_penumbra_consensus_state(self.consensus_height_of_b_on_a)
            .await?;

        // 3. verify that the counterparty chain stored the correct consensus state of Penumbra at
        //    the given consensus height
        let proof_consensus_state_of_b_on_a = self.proof_consensus_state_of_b_on_a.clone();
        proof_verification::verify_client_consensus_state(
            &trusted_client_state,
            self.proofs_height_on_a,
            &self.counterparty.prefix,
            &proof_consensus_state_of_b_on_a,
            &trusted_consensus_state.root,
            &ClientConsensusStatePath::new(
                &self.counterparty.client_id,
                &self.consensus_height_of_b_on_a,
            ),
            expected_consensus,
        )
        .context("couldn't verify client consensus state")?;

        // VALIDATION SUCCESSSFUL, now execute
        //
        // new_conn is the new connection that we will open on this chain
        let mut new_conn = ConnectionEnd {
            state: ConnectionState::TryOpen,
            client_id: self.client_id_on_b.clone(),
            counterparty: self.counterparty.clone(),
            versions: self.versions_on_a.clone(),
            delay_period: self.delay_period,
        };

        new_conn.versions = vec![pick_connection_version(
            &SUPPORTED_VERSIONS.to_vec(),
            &self.versions_on_a.clone(),
        )?];

        let new_connection_id = ConnectionId::new(
            state
                .get_connection_counter()
                .await
                .context("unable to get connection counter")?
                .0,
        );

        state
            .put_new_connection(&new_connection_id, new_conn)
            .await
            .context("unable to put new connection")?;

        state.record(
            events::ConnectionOpenTry {
                conn_id_on_b: new_connection_id.clone(),
                client_id_on_b: self.client_id_on_b.clone(),
                conn_id_on_a: self.counterparty.connection_id.clone().unwrap_or_default(),
                client_id_on_a: self.counterparty.client_id.clone(),
            }
            .into(),
        );

        Ok(())
    }
}
async fn consensus_height_is_correct<S: ChainStateReadExt>(
    state: &S,
    msg: &MsgConnectionOpenTry,
) -> anyhow::Result<()> {
    let current_height = IBCHeight::new(
        state.get_revision_number().await?,
        state.get_block_height().await?,
    )?;
    if msg.consensus_height_of_b_on_a >= current_height {
        anyhow::bail!("consensus height is greater than the current block height",);
    }

    Ok(())
}
async fn penumbra_client_state_is_well_formed<S: ChainStateReadExt>(
    state: &S,
    msg: &MsgConnectionOpenTry,
) -> anyhow::Result<()> {
    let height = state.get_block_height().await?;
    let chain_id = state.get_chain_id().await?;
    validate_penumbra_client_state(msg.client_state_of_b_on_a.clone(), &chain_id, height)?;

    Ok(())
}
