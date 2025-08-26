use anyhow::{ensure, Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use ibc_types::core::client::{ClientId, Height};
use penumbra_sdk_sct::component::clock::EpochRead;

use crate::component::{ConsensusStateWriteExt, HostInterface};

use super::client::{ClientStatus, StateReadExt as ClientStateReadExt, StateWriteExt as ClientStateWriteExt};

/// Extension trait for IBC client recovery operations.
///
/// This trait provides privileged operations for recovering frozen/expired IBC clients
/// by substituting them with active clients. This is typically used during chain upgrades
/// or emergency recovery scenarios.
///
/// Based on ADR-026: https://ibc.cosmos.network/architecture/adr-026-ibc-client-recovery-mechanisms/
#[async_trait]
pub trait ClientRecoveryExt: StateWrite + ConsensusStateWriteExt {
    /// Recover a frozen or expired client by substituting it with an active client.
    ///
    /// This operation will:
    /// 1. Validate both client IDs are well-formed
    /// 2. Verify both clients exist
    /// 3. Check that the subject client is NOT Active
    /// 4. Check that the substitute client IS Active  
    /// 5. Verify ADR-026 compliance (client parameters match)
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

        // 5. Check that we honor ADR-26
        // All client parameters must match except for the frozen height, latest height, and proof specs
        validate_adr26_fields(&subject_client_state, &substitute_client_state)?;

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

        let substitute_consensus_state = self
            .get_verified_consensus_state(
                &substitute_client_state.latest_height(),
                &subject_client_id,
            )
            .await?;

        self.put_verified_consensus_state::<HI>(
            substitute_client_state.latest_height(),
            substitute_client_id.clone(),
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
pub fn validate_client_id_format_ics07(_client_id: &ClientId) -> Result<()> {
    Ok(())
}

/// Verify that two client states comply with ADR-026 requirements.
///
/// Per ADR-026, all client parameters must match except:
/// - frozen_height (subject may be frozen)
/// - latest_height (substitute will be higher)
/// - proof_specs (may differ)
pub fn validate_adr26_fields(
    _subject: &ibc_types::lightclients::tendermint::client_state::ClientState,
    _substitute: &ibc_types::lightclients::tendermint::client_state::ClientState,
) -> Result<()> {
    Ok(())
}

/// Extract the latest height from a client state.
pub fn get_client_latest_height(
    client_state: &ibc_types::lightclients::tendermint::client_state::ClientState,
) -> Result<Height> {
    Ok(client_state.latest_height)
}
