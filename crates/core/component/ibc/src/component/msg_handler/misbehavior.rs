use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use ibc_types::core::client::{events, ClientType};
use ibc_types::core::client::{msgs::MsgSubmitMisbehaviour, ClientId};
use ibc_types::lightclients::tendermint::client_state::ClientState as TendermintClientState;
use ibc_types::lightclients::tendermint::header::Header as TendermintHeader;
use ibc_types::lightclients::tendermint::misbehaviour::Misbehaviour as TendermintMisbehavior;
use ibc_types::lightclients::tendermint::TENDERMINT_CLIENT_TYPE;
use tendermint_light_client_verifier::{
    types::{TrustedBlockState, UntrustedBlockState},
    ProdVerifier, Verdict, Verifier,
};

use cnidarium_component::ChainStateReadExt;

use super::update_client::verify_header_validator_set;
use super::MsgHandler;
use crate::component::{client::StateWriteExt as _, ics02_validation, ClientStateReadExt as _};

#[async_trait]
impl MsgHandler for MsgSubmitMisbehaviour {
    async fn check_stateless<H>(&self) -> Result<()> {
        misbehavior_is_tendermint(self)?;
        let untrusted_misbehavior =
            ics02_validation::get_tendermint_misbehavior(self.misbehaviour.clone())?;
        // misbehavior must either contain equivocation or timestamp monotonicity violation
        if !misbehavior_equivocation_violation(&untrusted_misbehavior)
            && !misbehavior_timestamp_monotonicity_violation(&untrusted_misbehavior)
        {
            anyhow::bail!(
                "misbehavior must either contain equivocation or timestamp monotonicity violation"
            );
        }

        Ok(())
    }

    async fn try_execute<S: StateWrite + ChainStateReadExt, H>(&self, mut state: S) -> Result<()> {
        tracing::debug!(msg = ?self);

        let untrusted_misbehavior =
            ics02_validation::get_tendermint_misbehavior(self.misbehaviour.clone())?;

        // misbehavior must either contain equivocation or timestamp monotonicity violation
        if !misbehavior_equivocation_violation(&untrusted_misbehavior)
            && !misbehavior_timestamp_monotonicity_violation(&untrusted_misbehavior)
        {
            anyhow::bail!(
                "misbehavior must either contain equivocation or timestamp monotonicity violation"
            );
        }

        // verify that both headers verify for an update client on the last trusted header for
        // client_id
        let client_state = client_is_present(&state, self).await?;

        // NOTE: we are allowing expired clients here. it seems correct to allow expired clients to
        // be frozen on evidence of misbehavior.
        client_is_not_frozen(&client_state)?;

        let trusted_client_state = client_state;

        verify_misbehavior_header(
            &state,
            &untrusted_misbehavior.client_id,
            &untrusted_misbehavior.header1,
            &trusted_client_state,
        )
        .await?;
        verify_misbehavior_header(
            &state,
            &untrusted_misbehavior.client_id,
            &untrusted_misbehavior.header2,
            &trusted_client_state,
        )
        .await?;

        tracing::info!(client_id = ?untrusted_misbehavior.client_id, "received valid misbehavior evidence! freezing client");

        // freeze the client
        let frozen_client =
            trusted_client_state.with_frozen_height(untrusted_misbehavior.header1.height());
        state.put_client(&self.client_id, frozen_client);

        state.record(
            events::ClientMisbehaviour {
                client_id: self.client_id.clone(),
                client_type: ClientType::new(TENDERMINT_CLIENT_TYPE.to_string()),
            }
            .into(),
        );

        Ok(())
    }
}

async fn client_is_present<S: StateRead>(
    state: S,
    msg: &MsgSubmitMisbehaviour,
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

async fn verify_misbehavior_header<S: ChainStateReadExt>(
    state: &S,
    client_id: &ClientId,
    mb_header: &TendermintHeader,
    trusted_client_state: &TendermintClientState,
) -> Result<()> {
    let trusted_height = mb_header.trusted_height;
    let last_trusted_consensus_state = state
        .get_verified_consensus_state(&trusted_height, &client_id)
        .await?;

    let trusted_height = trusted_height
        .revision_height()
        .try_into()
        .context("invalid header height")?;

    let trusted_validator_set =
        verify_header_validator_set(mb_header, &last_trusted_consensus_state)?;

    let trusted_state = TrustedBlockState {
        chain_id: &trusted_client_state.chain_id.clone().into(),
        header_time: last_trusted_consensus_state.timestamp,
        height: trusted_height,
        next_validators: trusted_validator_set,
        next_validators_hash: last_trusted_consensus_state.next_validators_hash,
    };

    let untrusted_state = UntrustedBlockState {
        signed_header: &mb_header.signed_header,
        validators: &mb_header.validator_set,
        next_validators: None,
    };

    let options = trusted_client_state.as_light_client_options()?;
    let verifier = ProdVerifier::default();

    let verdict = verifier.verify_misbehaviour_header(
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
    }
}

fn misbehavior_equivocation_violation(misbehavior: &TendermintMisbehavior) -> bool {
    misbehavior.header1.height() == misbehavior.header2.height()
        && misbehavior.header1.signed_header.commit.block_id.hash
            != misbehavior.header2.signed_header.commit.block_id.hash
}

fn misbehavior_timestamp_monotonicity_violation(misbehavior: &TendermintMisbehavior) -> bool {
    misbehavior.header1.height() < misbehavior.header2.height()
        && misbehavior.header1.signed_header.header.time
            > misbehavior.header2.signed_header.header.time
}

fn misbehavior_is_tendermint(msg: &MsgSubmitMisbehaviour) -> Result<()> {
    if ics02_validation::is_tendermint_misbehavior(&msg.misbehaviour) {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "MsgSubmitMisbehaviour is not tendermint misbehavior"
        ))
    }
}
