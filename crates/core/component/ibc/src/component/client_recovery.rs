use anyhow::{ensure, Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use ibc_types::core::client::{ClientId, Height};
use penumbra_sdk_sct::component::clock::EpochRead;

use crate::component::{ConsensusStateWriteExt, HostInterface};

use super::client::{
    ClientStatus, StateReadExt as ClientStateReadExt, StateWriteExt as ClientStateWriteExt,
};

/// Extension trait for IBC client recovery operations.
///
/// This trait provides privileged operations for recovering frozen/expired IBC clients
/// by substituting them with active clients. This is typically used during chain upgrades
/// or emergency recovery scenarios.
#[async_trait]
pub trait ClientRecoveryExt: StateWrite + ConsensusStateWriteExt {
    /// Validate a client recovery operation
    async fn validate_recover_client<HI: HostInterface>(
        &self,
        subject_client_id: &ClientId,
        substitute_client_id: &ClientId,
    ) -> Result<()> {
        tracing::debug!(
            %subject_client_id,
            %substitute_client_id,
            "validating ibc client recovery"
        );

        // 1. Check that the clients are well-formed (regex validation)
        validate_client_id_format_ics07(subject_client_id)?;
        validate_client_id_format_ics07(substitute_client_id)?;

        // Needed for status checks
        let local_chain_current_time = self
            .get_current_block_timestamp()
            .await
            .context("failed to get current block timestamp")?;

        // 2. Check that the clients are found
        let subject_client_state = self
            .get_client_state(subject_client_id)
            .await
            .context("subject client not found")?;

        let substitute_client_state = self
            .get_client_state(substitute_client_id)
            .await
            .context("substitute client not found")?;

        // 3. Check that the subject client is NOT Active
        let subject_status = self
            .get_client_status(subject_client_id, local_chain_current_time)
            .await;
        ensure!(
            subject_status != ClientStatus::Active,
            "subject client must not be Active, found: {}",
            subject_status
        );

        // 4. Check that the substitute client IS Active
        let substitute_status = self
            .get_client_status(substitute_client_id, local_chain_current_time)
            .await;
        ensure!(
            substitute_status == ClientStatus::Active,
            "substitute client must be Active, found: {}",
            substitute_status
        );

        // 5. Check that all client parameters must match except
        // for the frozen height, latest height, trust period, and proof specs
        check_field_consistency(&subject_client_state, &substitute_client_state)?;

        // 6. Check that the substitute client height is greater than subject's latest height
        let subject_height = get_client_latest_height(&subject_client_state)?;
        let substitute_height = get_client_latest_height(&substitute_client_state)?;
        ensure!(
            substitute_height > subject_height,
            "substitute client height ({}) must be greater than subject client height ({})",
            substitute_height,
            subject_height
        );

        // 7. Perform the recovery: copy substitute client state to subject client
        tracing::debug!("overwriting client state");

        Ok(())
    }
    /// Recover a frozen or expired client by substituting it with an active client.
    ///
    /// This operation will:
    /// 1. Validate both client IDs are well-formed
    /// 2. Verify both clients exist
    /// 3. Check that the subject client is NOT Active
    /// 4. Check that the substitute client IS Active
    /// 5. Verify client parameters match.
    /// 6. Verify substitute client has greater height
    /// 7. Copy the substitute client's state over the subject client
    async fn recover_client<HI: HostInterface>(
        &mut self,
        subject_client_id: &ClientId,
        substitute_client_id: &ClientId,
    ) -> Result<()> {
        tracing::debug!(
            %subject_client_id,
            %substitute_client_id,
            "starting ibc client recovery"
        );
        self.validate_recover_client::<HI>(&subject_client_id, &substitute_client_id)
            .await?;

        let substitute_client_state = self
            .get_client_state(substitute_client_id)
            .await
            .context("substitute client not found")?;

        let substitute_consensus_state = self
            .get_verified_consensus_state(
                &substitute_client_state.latest_height(),
                &substitute_client_id,
            )
            .await?;

        // smooth brain: we write the substitute - into -> the subject.
        self.put_verified_consensus_state::<HI>(
            substitute_client_state.latest_height(),
            subject_client_id.clone(),
            substitute_consensus_state,
        )
        .await?;

        self.put_client(&subject_client_id, substitute_client_state);

        tracing::info!(
            subject = %subject_client_id,
            substitute = %substitute_client_id,
            "client recovery completed successfully"
        );

        Ok(())
    }
}

impl<T: StateWrite + ConsensusStateWriteExt> ClientRecoveryExt for T {}

/// Validate that a client ID matches the expected format.
/// Client IDs must be of the form: 07-tendermint-<NUM> where NUM is a non-empty sequence of digits
/// TODO(erwan): iirc there's an ibc types routine that does this?
pub fn validate_client_id_format_ics07(client_id: &ClientId) -> Result<()> {
    use regex::Regex;

    let client_id_str = client_id.as_str();

    // Match exactly: 07-tendermint- followed by one or more digits
    let re = Regex::new(r"^07-tendermint-\d+$").expect("valid regex");

    ensure!(
        re.is_match(client_id_str),
        "invalid client ID format: '{}'. Expected format: 07-tendermint-<NUM> (e.g., 07-tendermint-0, 07-tendermint-123)",
        client_id_str
    );

    let parts: Vec<&str> = client_id_str.split('-').collect();
    if parts.len() == 3 {
        let num_part = parts[2];
        ensure!(
            !(num_part.len() > 1 && num_part.starts_with('0')),
            "invalid client ID: '{}'. Number part cannot have leading zeros",
            client_id_str
        );
    }

    Ok(())
}

/// Check that the field of two client states are coherent.
///
/// The goal is to verify that a subject/substitute couple are fundamentally the same client.
/// Evne if at different points in their lifecycle. This is directly inspired by Cosmos ADR-26,
/// which recognizes that client recovery is a form of controlled mutation: we are not replacing
/// one client state with an arbitrary other, but ratehr fast-forwarding a stuck client to a
/// healthy state.
///
/// The Tendermint ClientState contains:
/// ```
/// ClientState {
///     // IMMUTABLE
///     chain_id,          
///     trust_level,       
///     trusting_period,   
///     unbonding_period,  
///     max_clock_drift,   
///     upgrade_path,      
///     allow_update,      
///     
///     // MUTABLE
///     latest_height,     // can advance
///     frozen_height,     // can be unfrozen
///     proof_specs,       // mechanical, not trust-related
/// }
/// ```
pub fn check_field_consistency(
    subject: &ibc_types::lightclients::tendermint::client_state::ClientState,
    substitute: &ibc_types::lightclients::tendermint::client_state::ClientState,
) -> Result<()> {
    ensure!(
        subject.chain_id == substitute.chain_id,
        "chain IDs must match: subject has '{}', substitute has '{}'",
        subject.chain_id,
        substitute.chain_id
    );

    ensure!(
        subject.trust_level == substitute.trust_level,
        "trust levels must match: subject has '{:?}', substitute has '{:?}'",
        subject.trust_level,
        substitute.trust_level
    );

    // We leave out checking the trust period.
    // This makes testing easier, gives some leeway in case of
    // misconfiguration, and is safe because ICS02 validation requires:
    // `trust_period < unbonding_period`

    ensure!(
        subject.unbonding_period == substitute.unbonding_period,
        "unbonding periods must match: subject has '{:?}', substitute has '{:?}'",
        subject.unbonding_period,
        substitute.unbonding_period
    );

    ensure!(
        subject.max_clock_drift == substitute.max_clock_drift,
        "max clock drifts must match: subject has '{:?}', substitute has '{:?}'",
        subject.max_clock_drift,
        substitute.max_clock_drift
    );

    ensure!(
        subject.upgrade_path == substitute.upgrade_path,
        "upgrade paths must match: subject has '{:?}', substitute has '{:?}'",
        subject.upgrade_path,
        substitute.upgrade_path
    );

    ensure!(
        subject.allow_update == substitute.allow_update,
        "allow_update flags must match: subject has '{:?}', substitute has '{:?}'",
        subject.allow_update,
        substitute.allow_update
    );

    Ok(())
}

/// Extract the latest height from a client state.
pub fn get_client_latest_height(
    client_state: &ibc_types::lightclients::tendermint::client_state::ClientState,
) -> Result<Height> {
    Ok(client_state.latest_height)
}
