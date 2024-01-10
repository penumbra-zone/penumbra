use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use ibc_types::{
    core::connection::{
        events, msgs::MsgConnectionOpenConfirm, ConnectionEnd, Counterparty, State,
    },
    path::ConnectionPath,
};

use crate::{
    component::{
        client::StateReadExt as _,
        connection::{StateReadExt as _, StateWriteExt as _},
        proof_verification, MsgHandler,
    },
    IBC_COMMITMENT_PREFIX,
};

#[async_trait]
impl MsgHandler for MsgConnectionOpenConfirm {
    async fn check_stateless<H>(&self) -> Result<()> {
        // NOTE: other than that the message is a well formed ConnectionOpenConfirm,
        // there is no other stateless validation to perform.

        Ok(())
    }

    async fn try_execute<S: StateWrite, AH, HI>(&self, mut state: S) -> Result<()> {
        tracing::debug!(msg = ?self);
        // Validate a ConnectionOpenConfirm message, completing the IBC connection handshake.
        //
        // Verify that we have a connection in the correct state (TRYOPEN), and that the
        // counterparty has committed a connection with the expected state (OPEN) to their state
        // store.
        //
        // Here we are Chain B.
        // CHAINS:          (A, B)
        // PRIOR STATE:     (OPEN, TRYOPEN)
        // POSTERIOR STATE: (OPEN, OPEN)
        //
        // verify that a connection with the provided ID exists and is in the correct state
        // (TRYOPEN)
        let connection = verify_previous_connection(&state, self).await?;

        let expected_conn = ConnectionEnd {
            state: State::Open,
            client_id: connection.counterparty.client_id.clone(),
            counterparty: Counterparty {
                client_id: connection.client_id.clone(),
                connection_id: Some(self.conn_id_on_b.clone()),
                prefix: IBC_COMMITMENT_PREFIX.clone(),
            },
            versions: connection.versions.to_vec(),
            delay_period: connection.delay_period,
        };

        // get the trusted client state for the counterparty
        let trusted_client_state = state.get_client_state(&connection.client_id).await?;

        // check if the client is frozen
        // TODO: should we also check if the client is expired here?
        if trusted_client_state.is_frozen() {
            anyhow::bail!("client is frozen");
        }

        // get the stored consensus state for the counterparty
        let trusted_consensus_state = state
            .get_verified_consensus_state(&self.proof_height_on_a, &connection.client_id)
            .await?;

        // PROOF VERIFICATION
        // in connectionOpenConfirm, only the inclusion of the connection state must be
        // verified, not the client or consensus states.

        let proof_conn_end_on_a = self.proof_conn_end_on_a.clone();
        proof_verification::verify_connection_state(
            &trusted_client_state,
            self.proof_height_on_a,
            &connection.counterparty.prefix,
            &proof_conn_end_on_a,
            &trusted_consensus_state.root,
            &ConnectionPath::new(connection.counterparty.connection_id.as_ref().ok_or_else(
                || anyhow::anyhow!("missing counterparty in connection open confirm"),
            )?),
            &expected_conn,
        )?;

        // VERIFICATION SUCCESSFUL. now execute
        let mut connection = state
            .get_connection(&self.conn_id_on_b)
            .await?
            .ok_or_else(|| anyhow::anyhow!("no connection with the given ID"))
            .context("unable to get connection")?;

        connection.state = State::Open;

        state.update_connection(&self.conn_id_on_b, connection.clone());

        state.record(
            events::ConnectionOpenConfirm {
                conn_id_on_b: self.conn_id_on_b.clone(),
                client_id_on_b: connection.client_id.clone(),
                conn_id_on_a: connection
                    .counterparty
                    .connection_id
                    .clone()
                    .unwrap_or_default(),
                client_id_on_a: connection.counterparty.client_id,
            }
            .into(),
        );

        Ok(())
    }
}

async fn verify_previous_connection<S: StateRead>(
    state: S,
    msg: &MsgConnectionOpenConfirm,
) -> anyhow::Result<ConnectionEnd> {
    let connection = state
        .get_connection(&msg.conn_id_on_b)
        .await?
        .ok_or_else(|| anyhow::anyhow!("connection not found"))?;

    if !connection.state_matches(&State::TryOpen) {
        Err(anyhow::anyhow!("connection not in correct state"))
    } else {
        Ok(connection)
    }
}
