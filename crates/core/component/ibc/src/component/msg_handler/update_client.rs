use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::ChainStateReadExt;
use ibc_types::{
    core::{client::events::UpdateClient, client::msgs::MsgUpdateClient, client::ClientId},
    lightclients::tendermint::client_state::ClientState as TendermintClientState,
    lightclients::tendermint::header::Header as TendermintHeader,
    lightclients::tendermint::{
        consensus_state::ConsensusState as TendermintConsensusState, TENDERMINT_CLIENT_TYPE,
    },
};
use tendermint::validator;
use tendermint_light_client_verifier::{
    types::{TrustedBlockState, UntrustedBlockState},
    ProdVerifier, Verdict, Verifier,
};

use crate::component::{
    client::{
        ConsensusStateWriteExt as _, Ics2ClientExt as _, StateReadExt as _, StateWriteExt as _,
    },
    ics02_validation, MsgHandler,
};

#[async_trait]
impl MsgHandler for MsgUpdateClient {
    async fn check_stateless<H>(&self) -> Result<()> {
        header_is_tendermint(self)?;

        Ok(())
    }

    async fn try_execute<S: StateWrite + ChainStateReadExt, H>(&self, mut state: S) -> Result<()> {
        // Optimization: no-op if the update is already committed.  We no-op
        // to Ok(()) rather than erroring to avoid having two "racing" relay
        // transactions fail just because they both contain the same client
        // update.
        if !update_is_already_committed(&state, self).await? {
            tracing::debug!(msg = ?self);

            let client_state = client_is_present(&state, self).await?;

            client_is_not_frozen(&client_state)?;
            client_is_not_expired(&state, &self.client_id, &client_state).await?;

            let trusted_client_state = client_state;

            let untrusted_header =
                ics02_validation::get_tendermint_header(self.client_message.clone())?;

            header_revision_matches_client_state(&trusted_client_state, &untrusted_header)?;
            header_height_is_consistent(&untrusted_header)?;

            // The (still untrusted) header uses the `trusted_height` field to
            // specify the trusted anchor data it is extending.
            let trusted_height = untrusted_header.trusted_height;

            // We use the specified trusted height to query the trusted
            // consensus state the update extends.
            let last_trusted_consensus_state = state
                .get_verified_consensus_state(&trusted_height, &self.client_id)
                .await?;

            // We also have to convert from an IBC height, which has two
            // components, to a Tendermint height, which has only one.
            let trusted_height = trusted_height
                .revision_height()
                .try_into()
                .context("invalid header height")?;

            let trusted_validator_set =
                verify_header_validator_set(&untrusted_header, &last_trusted_consensus_state)?;

            // Now we build the trusted and untrusted states to feed to the Tendermint light client.

            let trusted_state = TrustedBlockState {
                // TODO(erwan): do we need an additional check on `chain_id`
                chain_id: &trusted_client_state.chain_id.clone().into(),
                header_time: last_trusted_consensus_state.timestamp,
                height: trusted_height,
                next_validators: trusted_validator_set,
                next_validators_hash: last_trusted_consensus_state.next_validators_hash,
            };

            let untrusted_state = UntrustedBlockState {
                signed_header: &untrusted_header.signed_header,
                validators: &untrusted_header.validator_set,
                next_validators: None, // TODO: do we need this?
            };

            let options = trusted_client_state.as_light_client_options()?;
            let verifier = ProdVerifier::default();

            let verdict = verifier.verify_update_header(
                untrusted_state,
                trusted_state,
                &options,
                state.get_block_timestamp().await?,
            );

            match verdict {
                Verdict::Success => Ok(()),
                Verdict::NotEnoughTrust(voting_power_tally) => Err(anyhow::anyhow!(
                    "not enough trust, voting power tally: {:?}",
                    voting_power_tally
                )),
                Verdict::Invalid(detail) => Err(anyhow::anyhow!(
                    "could not verify tendermint header: invalid: {:?}",
                    detail
                )),
            }?;

            let trusted_header = untrusted_header;

            // get the latest client state
            let client_state = state
                .get_client_state(&self.client_id)
                .await
                .context("unable to get client state")?;

            // NOTE: next_tendermint_state will freeze the client on equivocation.
            let (next_tm_client_state, next_tm_consensus_state) = state
                .next_tendermint_state(
                    self.client_id.clone(),
                    client_state.clone(),
                    trusted_header.clone(),
                )
                .await;

            // store the updated client and consensus states
            state.put_client(&self.client_id, next_tm_client_state);
            state
                .put_verified_consensus_state(
                    trusted_header.height(),
                    self.client_id.clone(),
                    next_tm_consensus_state,
                )
                .await?;

            state.record(
                UpdateClient {
                    client_id: self.client_id.clone(),
                    client_type: ibc_types::core::client::ClientType(
                        TENDERMINT_CLIENT_TYPE.to_string(),
                    ), // TODO: hardcoded
                    consensus_height: trusted_header.height(),
                    header: <ibc_types::lightclients::tendermint::header::Header as ibc_proto::Protobuf<ibc_proto::ibc::lightclients::tendermint::v1::Header>>::encode_vec(trusted_header),
                }
                .into(),
            );
            return Ok(());
        } else {
            tracing::debug!("skipping duplicate update");
        }

        Ok(())
    }
}

fn header_is_tendermint(msg: &MsgUpdateClient) -> anyhow::Result<()> {
    if ics02_validation::is_tendermint_header_state(&msg.client_message) {
        Ok(())
    } else {
        Err(anyhow::anyhow!("MsgUpdateClient: not a tendermint header"))
    }
}

async fn update_is_already_committed<S: StateRead>(
    state: S,
    msg: &MsgUpdateClient,
) -> anyhow::Result<bool> {
    let untrusted_header = ics02_validation::get_tendermint_header(msg.client_message.clone())?;
    let client_id = msg.client_id.clone();

    // check if we already have a consensus state for this height, if we do, check that it is
    // the same as this update, if it is, return early.
    let height = untrusted_header.height();
    let untrusted_consensus_state = TendermintConsensusState::from(untrusted_header);
    if let Ok(stored_consensus_state) = state
        .get_verified_consensus_state(&height, &client_id)
        .await
    {
        let stored_tm_consensus_state = stored_consensus_state;

        Ok(stored_tm_consensus_state == untrusted_consensus_state)
    } else {
        // If we don't have a consensus state for this height for
        // whatever reason (either missing or a DB error), we don't
        // consider it an error, it's just not already committed.
        Ok(false)
    }
}

async fn client_is_not_expired<S: ChainStateReadExt>(
    state: &S,
    client_id: &ClientId,
    client_state: &TendermintClientState,
) -> anyhow::Result<()> {
    let latest_consensus_state = state
        .get_verified_consensus_state(&client_state.latest_height(), client_id)
        .await?;

    // TODO(erwan): for now there is no casting that needs to happen because `get_verified_consensus_state` does not return an
    // abstracted consensus state.
    let latest_consensus_state_tm = latest_consensus_state;

    let now = state.get_block_timestamp().await?;
    let time_elapsed = now.duration_since(latest_consensus_state_tm.timestamp)?;

    if client_state.expired(time_elapsed) {
        Err(anyhow::anyhow!("client is expired"))
    } else {
        Ok(())
    }
}

async fn client_is_present<S: StateRead>(
    state: S,
    msg: &MsgUpdateClient,
) -> anyhow::Result<TendermintClientState> {
    state.get_client_type(&msg.client_id).await?;

    state.get_client_state(&msg.client_id).await
}

fn client_is_not_frozen(client: &TendermintClientState) -> anyhow::Result<()> {
    if client.is_frozen() {
        Err(anyhow::anyhow!("client is frozen"))
    } else {
        Ok(())
    }
}

fn header_revision_matches_client_state(
    trusted_client_state: &TendermintClientState,
    untrusted_header: &TendermintHeader,
) -> anyhow::Result<()> {
    if untrusted_header.height().revision_number() != trusted_client_state.chain_id.version() {
        Err(anyhow::anyhow!(
            "client update revision number does not match client state"
        ))
    } else {
        Ok(())
    }
}

fn header_height_is_consistent(untrusted_header: &TendermintHeader) -> anyhow::Result<()> {
    if untrusted_header.height() <= untrusted_header.trusted_height {
        Err(anyhow::anyhow!(
            "client update height is not greater than trusted height"
        ))
    } else {
        Ok(())
    }
}

pub fn verify_header_validator_set<'h>(
    untrusted_header: &'h TendermintHeader,
    last_trusted_consensus_state: &TendermintConsensusState,
) -> anyhow::Result<&'h validator::Set> {
    if untrusted_header.trusted_validator_set.hash()
        != last_trusted_consensus_state.next_validators_hash
    {
        Err(anyhow::anyhow!(
            "client update validator set hash does not match trusted consensus state"
        ))
    } else {
        Ok(&untrusted_header.trusted_validator_set)
    }
}
