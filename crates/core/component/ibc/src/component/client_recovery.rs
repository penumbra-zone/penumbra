use anyhow::{ensure, Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use ibc_types::core::client::{ClientId, Height};
use penumbra_sdk_sct::component::clock::EpochRead;

use crate::component::{ConsensusStateWriteExt, HostInterface};

use super::client::{
    ClientStatus, StateReadExt as ClientStateReadExt, StateWriteExt as ClientStateWriteExt,
};

/// How strictly a recovery requires the subject and substitute to be "the same client".
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryPolicy {
    /// Every trust parameter must match (see [`check_field_consistency`]). This is the
    /// ADR-26 rule, and what an operator-supplied recovery (`pd migrate ibc-recovery`) uses.
    Strict,
    /// Only the counterparty chain ID must match, and it must equal `expected_chain_id`.
    /// The subject then takes on the substitute's trust level, unbonding period and clock
    /// drift along with its latest state.
    ///
    /// This is for recoveries hardcoded into an upgrade, where re-parameterizing the client
    /// is part of what the upgrade ships. Such a client's parameters were set by a relayer
    /// when it was created, and a later substitute created against the same chain can
    /// legitimately carry different ones (the counterparty changed its unbonding period, or
    /// the relayer changed its defaults). Pinning the chain ID in code means the recovery
    /// can never be pointed at a client for a different chain.
    AdoptSubstituteParameters { expected_chain_id: String },
}

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
        self.validate_recover_client_with_policy::<HI>(
            subject_client_id,
            substitute_client_id,
            &RecoveryPolicy::Strict,
        )
        .await
    }

    /// Validate a client recovery operation under the given [`RecoveryPolicy`].
    async fn validate_recover_client_with_policy<HI: HostInterface>(
        &self,
        subject_client_id: &ClientId,
        substitute_client_id: &ClientId,
        policy: &RecoveryPolicy,
    ) -> Result<()> {
        tracing::debug!(
            %subject_client_id,
            %substitute_client_id,
            ?policy,
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

        // 5. Check that the two clients track the same chain. Under the strict policy,
        // all client parameters must match except for the frozen height, latest height,
        // trust period, and proof specs.
        match policy {
            RecoveryPolicy::Strict => {
                check_field_consistency(&subject_client_state, &substitute_client_state)?
            }
            RecoveryPolicy::AdoptSubstituteParameters { expected_chain_id } => check_chain_id(
                &subject_client_state,
                &substitute_client_state,
                expected_chain_id,
            )?,
        }

        // 6. Check that the substitute client height is greater than subject's latest height
        let subject_height = get_client_latest_height(&subject_client_state)?;
        let substitute_height = get_client_latest_height(&substitute_client_state)?;
        ensure!(
            substitute_height > subject_height,
            "substitute client height ({}) must be greater than subject client height ({})",
            substitute_height,
            subject_height
        );

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
        self.recover_client_with_policy::<HI>(
            subject_client_id,
            substitute_client_id,
            &RecoveryPolicy::Strict,
        )
        .await
    }

    /// Recover a frozen or expired client under the given [`RecoveryPolicy`].
    ///
    /// Identical to [`Self::recover_client`] except for step 5, which the policy decides.
    /// Either way the subject ends up with the substitute's full client state, parameters
    /// included; the policy only decides which parameter differences are acceptable.
    async fn recover_client_with_policy<HI: HostInterface>(
        &mut self,
        subject_client_id: &ClientId,
        substitute_client_id: &ClientId,
        policy: &RecoveryPolicy,
    ) -> Result<()> {
        tracing::debug!(
            %subject_client_id,
            %substitute_client_id,
            ?policy,
            "starting ibc client recovery"
        );
        self.validate_recover_client_with_policy::<HI>(
            subject_client_id,
            substitute_client_id,
            policy,
        )
        .await?;

        let subject_client_state = self
            .get_client_state(subject_client_id)
            .await
            .context("subject client not found")?;
        let substitute_client_state = self
            .get_client_state(substitute_client_id)
            .await
            .context("substitute client not found")?;

        // Say exactly which parameters change, so the upgrade's effect is on record in
        // every validator's migration log rather than only in the diff.
        for (field, before, after) in [
            (
                "trust_level",
                format!("{:?}", subject_client_state.trust_level),
                format!("{:?}", substitute_client_state.trust_level),
            ),
            (
                "trusting_period",
                format!("{:?}", subject_client_state.trusting_period),
                format!("{:?}", substitute_client_state.trusting_period),
            ),
            (
                "unbonding_period",
                format!("{:?}", subject_client_state.unbonding_period),
                format!("{:?}", substitute_client_state.unbonding_period),
            ),
            (
                "max_clock_drift",
                format!("{:?}", subject_client_state.max_clock_drift),
                format!("{:?}", substitute_client_state.max_clock_drift),
            ),
        ] {
            if before != after {
                tracing::info!(subject = %subject_client_id, field, %before, %after, "client parameter changes");
            }
        }

        let substitute_consensus_state = self
            .get_verified_consensus_state(
                &substitute_client_state.latest_height(),
                substitute_client_id,
            )
            .await?;

        // smooth brain: we write the substitute - into -> the subject.
        self.put_verified_consensus_state::<HI>(
            substitute_client_state.latest_height(),
            subject_client_id.clone(),
            substitute_consensus_state,
        )
        .await?;

        self.put_client(subject_client_id, substitute_client_state);

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
/// Evne if at different points in their lifecyle. This is directly inspired by Cosmos ADR-26,
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

/// Check that both clients track `expected_chain_id`, the only consistency check
/// [`RecoveryPolicy::AdoptSubstituteParameters`] makes.
pub fn check_chain_id(
    subject: &ibc_types::lightclients::tendermint::client_state::ClientState,
    substitute: &ibc_types::lightclients::tendermint::client_state::ClientState,
    expected_chain_id: &str,
) -> Result<()> {
    ensure!(
        subject.chain_id == substitute.chain_id,
        "chain IDs must match: subject has '{}', substitute has '{}'",
        subject.chain_id,
        substitute.chain_id
    );
    ensure!(
        subject.chain_id.as_str() == expected_chain_id,
        "expected both clients to track '{}', but they track '{}'",
        expected_chain_id,
        subject.chain_id
    );
    Ok(())
}

/// Extract the latest height from a client state.
pub fn get_client_latest_height(
    client_state: &ibc_types::lightclients::tendermint::client_state::ClientState,
) -> Result<Height> {
    Ok(client_state.latest_height)
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use cnidarium::{ArcStateDeltaExt, StateDelta, StateRead};
    use ibc_types::core::{client::Height, commitment::MerkleRoot, connection::ChainId};
    use ibc_types::lightclients::tendermint::{
        client_state::{AllowUpdate, ClientState},
        consensus_state::ConsensusState,
        TrustThreshold,
    };
    use penumbra_sdk_sct::component::clock::{EpochManager as _, EpochRead};
    use tendermint::Time;

    use super::*;

    struct MockHost {}

    #[async_trait]
    impl HostInterface for MockHost {
        async fn get_chain_id<S: StateRead>(_state: S) -> Result<String> {
            Ok("mock_chain_id".to_string())
        }

        async fn get_revision_number<S: StateRead>(_state: S) -> Result<u64> {
            Ok(0u64)
        }

        async fn get_block_height<S: StateRead>(state: S) -> Result<u64> {
            Ok(state.get_block_height().await?)
        }

        async fn get_block_timestamp<S: StateRead>(state: S) -> Result<tendermint::Time> {
            state.get_current_block_timestamp().await
        }
    }

    const DAY: u64 = 86_400;
    const NOW: &str = "2026-09-25T00:00:00Z";

    /// Shaped like mainnet's `07-tendermint-0` (Cosmos Hub, created in 2024).
    fn stale_hub_client() -> ClientState {
        ClientState {
            chain_id: ChainId::from_string("cosmoshub-4"),
            trust_level: TrustThreshold::TWO_THIRDS,
            trusting_period: Duration::from_secs(14 * DAY),
            unbonding_period: Duration::from_secs(21 * DAY),
            max_clock_drift: Duration::from_secs(7235),
            latest_height: Height::new(4, 31_036_976).unwrap(),
            proof_specs: vec![ics23::iavl_spec(), ics23::tendermint_spec()],
            upgrade_path: vec!["upgrade".into(), "upgradedIBCState".into()],
            allow_update: AllowUpdate {
                after_expiry: true,
                after_misbehaviour: true,
            },
            frozen_height: None,
        }
    }

    /// Shaped like mainnet's `07-tendermint-29`: same chain, different trust level and drift.
    fn fresh_hub_client() -> ClientState {
        ClientState {
            trust_level: TrustThreshold::ONE_THIRD,
            max_clock_drift: Duration::from_secs(25),
            latest_height: Height::new(4, 33_114_570).unwrap(),
            ..stale_hub_client()
        }
    }

    fn consensus_state_at(timestamp: Time) -> ConsensusState {
        ConsensusState::new(
            MerkleRoot {
                hash: vec![7u8; 32],
            },
            timestamp,
            tendermint::Hash::Sha256([9u8; 32]),
        )
    }

    fn days_ago(days: u64) -> Time {
        (Time::parse_from_rfc3339(NOW).unwrap() - Duration::from_secs(days * DAY)).unwrap()
    }

    /// State with the two clients installed: the subject last updated 400 days ago (so
    /// Expired), the substitute one day ago (so Active).
    async fn setup(subject: ClientState, substitute: ClientState) -> Arc<StateDelta<()>> {
        let mut state = Arc::new(StateDelta::new(()));
        let mut tx = state.try_begin_transaction().unwrap();
        tx.put_block_height(1);
        tx.put_block_timestamp(1u64, Time::parse_from_rfc3339(NOW).unwrap());
        tx.put_epoch_by_height(
            1,
            penumbra_sdk_sct::epoch::Epoch {
                index: 0,
                start_height: 0,
            },
        );
        for (id, client, updated) in [
            ("07-tendermint-0", subject, 400),
            ("07-tendermint-29", substitute, 1),
        ] {
            let id: ClientId = id.parse().unwrap();
            tx.put_verified_consensus_state::<MockHost>(
                client.latest_height,
                id.clone(),
                consensus_state_at(days_ago(updated)),
            )
            .await
            .unwrap();
            tx.put_client(&id, client);
        }
        tx.apply();
        state
    }

    fn ids() -> (ClientId, ClientId) {
        (
            "07-tendermint-0".parse().unwrap(),
            "07-tendermint-29".parse().unwrap(),
        )
    }

    fn adopt(chain_id: &str) -> RecoveryPolicy {
        RecoveryPolicy::AdoptSubstituteParameters {
            expected_chain_id: chain_id.to_string(),
        }
    }

    #[tokio::test]
    async fn strict_recovery_rejects_reparameterized_substitute() {
        let mut state = setup(stale_hub_client(), fresh_hub_client()).await;
        let (subject, substitute) = ids();
        let mut tx = state.try_begin_transaction().unwrap();
        let err = tx
            .recover_client::<MockHost>(&subject, &substitute)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("trust levels must match"), "{err}");
    }

    #[tokio::test]
    async fn adopting_recovery_takes_the_substitute_state_wholesale() {
        let mut state = setup(stale_hub_client(), fresh_hub_client()).await;
        let (subject, substitute) = ids();
        let mut tx = state.try_begin_transaction().unwrap();
        let now = tx.get_current_block_timestamp().await.unwrap();
        assert_eq!(
            tx.get_client_status(&subject, now).await,
            ClientStatus::Expired
        );

        tx.recover_client_with_policy::<MockHost>(&subject, &substitute, &adopt("cosmoshub-4"))
            .await
            .unwrap();

        let recovered = tx.get_client_state(&subject).await.unwrap();
        assert_eq!(recovered, fresh_hub_client());
        assert_eq!(
            tx.get_client_status(&subject, now).await,
            ClientStatus::Active
        );
        assert_eq!(
            tx.get_verified_consensus_state(&recovered.latest_height, &subject)
                .await
                .unwrap(),
            consensus_state_at(days_ago(1)),
        );
        // The substitute is left in place and still usable.
        assert_eq!(
            tx.get_client_state(&substitute).await.unwrap(),
            fresh_hub_client()
        );
        assert_eq!(
            tx.get_client_status(&substitute, now).await,
            ClientStatus::Active
        );
    }

    #[tokio::test]
    async fn adopting_recovery_rejects_a_different_chain() {
        let other_chain = ClientState {
            chain_id: ChainId::from_string("osmosis-1"),
            // The revision number is part of the chain ID, so it has to agree.
            latest_height: Height::new(1, 71_275_053).unwrap(),
            ..fresh_hub_client()
        };
        let mut state = setup(stale_hub_client(), other_chain).await;
        let (subject, substitute) = ids();
        let mut tx = state.try_begin_transaction().unwrap();
        let err = tx
            .recover_client_with_policy::<MockHost>(&subject, &substitute, &adopt("cosmoshub-4"))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("chain IDs must match"), "{err}");
    }

    #[tokio::test]
    async fn adopting_recovery_rejects_an_unexpected_chain() {
        // Both clients agree with each other, but not with the chain the upgrade names.
        let mut state = setup(stale_hub_client(), fresh_hub_client()).await;
        let (subject, substitute) = ids();
        let mut tx = state.try_begin_transaction().unwrap();
        let err = tx
            .recover_client_with_policy::<MockHost>(&subject, &substitute, &adopt("celestia"))
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("expected both clients to track"),
            "{err}"
        );
    }

    #[tokio::test]
    async fn adopting_recovery_keeps_the_status_and_height_checks() {
        // An Active subject cannot be overwritten.
        let mut state = setup(fresh_hub_client(), fresh_hub_client()).await;
        let (subject, substitute) = ids();
        {
            let mut tx = state.try_begin_transaction().unwrap();
            tx.put_verified_consensus_state::<MockHost>(
                fresh_hub_client().latest_height,
                subject.clone(),
                consensus_state_at(days_ago(1)),
            )
            .await
            .unwrap();
            let err = tx
                .recover_client_with_policy::<MockHost>(
                    &subject,
                    &substitute,
                    &adopt("cosmoshub-4"),
                )
                .await
                .unwrap_err();
            assert!(err.to_string().contains("must not be Active"), "{err}");
        }

        // An Expired substitute cannot be used: its trusting period is 14 days.
        let mut state = setup(stale_hub_client(), fresh_hub_client()).await;
        let mut tx = state.try_begin_transaction().unwrap();
        tx.put_block_timestamp(
            1u64,
            (Time::parse_from_rfc3339(NOW).unwrap() + Duration::from_secs(15 * DAY)).unwrap(),
        );
        let err = tx
            .recover_client_with_policy::<MockHost>(&subject, &substitute, &adopt("cosmoshub-4"))
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("substitute client must be Active"),
            "{err}"
        );

        // A substitute behind the subject cannot be used.
        let behind = ClientState {
            latest_height: Height::new(4, 30_000_000).unwrap(),
            ..fresh_hub_client()
        };
        let mut state = setup(stale_hub_client(), behind).await;
        let mut tx = state.try_begin_transaction().unwrap();
        let err = tx
            .recover_client_with_policy::<MockHost>(&subject, &substitute, &adopt("cosmoshub-4"))
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("must be greater than subject"),
            "{err}"
        );
    }
}
