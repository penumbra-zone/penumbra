use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ChainStateReadExt;
use ibc_types::{
    core::{
        client::{events, msgs::MsgUpgradeClient},
        commitment::{MerkleProof, MerkleRoot},
    },
    lightclients::tendermint::consensus_state::ConsensusState as TendermintConsensusState,
    lightclients::tendermint::{
        client_state::ClientState as TendermintClientState, TENDERMINT_CLIENT_TYPE,
    },
};

use crate::component::client::ConsensusStateWriteExt as _;
use crate::component::{
    client::{StateReadExt as _, StateWriteExt as _},
    proof_verification::ClientUpgradeProofVerifier,
    MsgHandler,
};

static SENTINEL_UPGRADE_ROOT: &str = "sentinel_root";

#[async_trait]
impl MsgHandler for MsgUpgradeClient {
    async fn check_stateless<H>(&self) -> Result<()> {
        Ok(())
    }

    // execute an ibc client upgrade for a counterparty client.
    //
    // the message being parsed here is initiating an upgrade that allows the counterparty to
    // change certain parameters of its client state (such as the chain id), as well as the
    // consensus state (the next set of validators).
    //
    // in order for a client upgrade to be valid, the counterparty must have committed (using the
    // trusted, un-upgraded client state) the new client and consensus states to its state tree.
    //
    // the first consensus state of the upgraded client uses a sentinel root, against which no
    // proofs will verify. subsequent client updates, post-upgrade, will provide usable roots.
    async fn try_execute<S: StateWrite + ChainStateReadExt, H>(&self, mut state: S) -> Result<()> {
        tracing::debug!(msg = ?self);

        let upgraded_client_state_tm = TendermintClientState::try_from(self.client_state.clone())
            .context("client state is not a Tendermint client state")?;
        let upgraded_consensus_state_tm =
            TendermintConsensusState::try_from(self.consensus_state.clone())
                .context("consensus state is not a Tendermint consensus state")?;

        let proof_consensus_state: MerkleProof = self
            .proof_upgrade_consensus_state
            .clone()
            .try_into()
            .context("couldn't decode proof for upgraded consensus state")?;
        let proof_client_state: MerkleProof = self
            .proof_upgrade_client
            .clone()
            .try_into()
            .context("couldn't decode proof for upgraded client state")?;

        state
            .verify_client_upgrade_proof(
                &self.client_id,
                &proof_client_state,
                &proof_consensus_state,
                upgraded_consensus_state_tm.clone(),
                upgraded_client_state_tm.clone(),
            )
            .await?;

        let old_client_state = state.get_client_state(&self.client_id).await?;

        // construct the new client state to be committed to our state. we don't allow the
        // trust_level, trusting_period, clock_drift, allow_update, or frozen_height to change
        // across upgrades.
        //
        // NOTE: this client state can differ from the one that was committed on the other chain!
        // that is, the other chain *could* commit different trust level, trusting period, etc, but
        // we would just ignore it here. should we error instead?
        let new_client_state = TendermintClientState::new(
            upgraded_client_state_tm.chain_id,
            old_client_state.trust_level,
            old_client_state.trusting_period,
            upgraded_client_state_tm.unbonding_period,
            old_client_state.max_clock_drift,
            upgraded_client_state_tm.latest_height,
            upgraded_client_state_tm.proof_specs,
            upgraded_client_state_tm.upgrade_path,
            old_client_state.allow_update,
            old_client_state.frozen_height,
        )?;

        let new_consensus_state = TendermintConsensusState::new(
            MerkleRoot {
                hash: SENTINEL_UPGRADE_ROOT.into(),
            },
            upgraded_consensus_state_tm.timestamp,
            upgraded_consensus_state_tm.next_validators_hash,
        );

        let latest_height = new_client_state.latest_height();

        state.put_client(&self.client_id, new_client_state);
        state
            .put_verified_consensus_state(
                latest_height,
                self.client_id.clone(),
                new_consensus_state,
            )
            .await?;

        state.record(
            events::UpgradeClient {
                client_id: self.client_id.clone(),
                client_type: ibc_types::core::client::ClientType(
                    TENDERMINT_CLIENT_TYPE.to_string(),
                ),
                consensus_height: latest_height,
            }
            .into(),
        );

        Ok(())
    }
}
