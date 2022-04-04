use std::collections::BTreeSet;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use penumbra_proto::{stake as pb, Protobuf};
use penumbra_stake::{
    BaseRateData, DelegationChanges, Epoch, IdentityKey, RateData, Validator, ValidatorList,
    STAKING_TOKEN_ASSET_ID,
};
use penumbra_transaction::{Action, Transaction};
use serde::{Deserialize, Serialize};
use tendermint::{
    abci::{
        self,
        types::{Evidence, ValidatorUpdate},
    },
    block, PublicKey,
};
use tracing::instrument;

use super::{Component, Overlay, ShieldedPoolStore};
use crate::{genesis, PenumbraStore, WriteOverlayExt};

// Staking component
pub struct Staking {
    overlay: Overlay,
    /// Delegation changes accumulated over the course of this block, to be
    /// persisted at the end of the block for processing at the end of the next
    /// epoch.
    delegation_changes: DelegationChanges,
}

impl Staking {
    #[instrument(skip(self))]
    async fn end_epoch(&mut self, epoch_to_end: Epoch) -> Result<()> {
        // calculate rate data for next rate, move previous next rate to cur rate,
        // and save the next rate data. ensure that non-Active validators maintain constant rates.
        let mut total_changes = DelegationChanges::default();
        for height in epoch_to_end.start_height().value()..=epoch_to_end.end_height().value() {
            let changes = self
                .overlay
                .delegation_changes(height.try_into().unwrap())
                .await?;
            total_changes.delegations.extend(changes.delegations);
            total_changes.undelegations.extend(changes.undelegations);
        }
        tracing::debug!(
            total_delegations = total_changes.delegations.len(),
            total_undelegations = total_changes.undelegations.len(),
        );
        // TODO: now the delegations and undelegations need to be grouped by validator ?

        let chain_params = self.overlay.get_chain_params().await?;
        let unbonding_epochs = chain_params.unbonding_epochs;
        let active_validator_limit = chain_params.active_validator_limit;

        tracing::debug!("processing base rate");
        // We are transitioning to the next epoch, so set "cur_base_rate" to the previous "next_base_rate", and
        // update "next_base_rate".
        let current_base_rate = self.overlay.current_base_rate().await?;
        /// FIXME: set this less arbitrarily, and allow this to be set per-epoch
        /// 3bps -> 11% return over 365 epochs, why not
        const BASE_REWARD_RATE: u64 = 3_0000;

        let next_base_rate = current_base_rate.next(BASE_REWARD_RATE);

        // rename to curr_rate so it lines up with next_rate (same # chars)
        tracing::debug!(curr_base_rate = ?current_base_rate);
        tracing::debug!(?next_base_rate);

        // Update these in the JMT:
        self.overlay
            .set_base_rates(current_base_rate, next_base_rate)
            .await;

        let mut staking_token_supply = self
            .overlay
            .token_supply(&STAKING_TOKEN_ASSET_ID)
            .await?
            .expect("staking token should be known");

        Ok(())
    }

    /// Called during `end_epoch`. Will perform state transitions to validators based
    /// on changes to voting power that occurred in this epoch.
    pub fn process_epoch_transitions(
        &mut self,
        active_validator_limit: u64,
        unbonding_epochs: u64,
    ) -> Result<()> {
        Ok(())
    }

    // Returns the list of validator updates formatted for inclusion in the Tendermint `EndBlockResponse`
    pub async fn tm_validator_updates(&self) -> Result<Vec<ValidatorUpdate>> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl Component for Staking {
    async fn new(overlay: Overlay) -> Result<Self> {
        Ok(Self {
            overlay,
            delegation_changes: Default::default(),
        })
    }

    async fn init_chain(&mut self, app_state: &genesis::AppState) -> Result<()> {
        let starting_height = self.overlay.get_block_height().await?;
        let starting_epoch =
            Epoch::from_height(starting_height, self.overlay.get_epoch_duration().await?);
        let epoch_index = starting_epoch.index;

        // Delegations require knowing the rates for the next epoch, so
        // pre-populate with 0 reward => exchange rate 1 for the current
        // (index 0) and next (index 1) epochs for base rate data.
        let cur_base_rate = BaseRateData {
            epoch_index,
            base_reward_rate: 0,
            base_exchange_rate: 1_0000_0000,
        };
        let next_base_rate = BaseRateData {
            epoch_index: epoch_index + 1,
            base_reward_rate: 0,
            base_exchange_rate: 1_0000_0000,
        };
        self.overlay
            .set_base_rates(cur_base_rate, next_base_rate)
            .await;

        // Add initial validators to the JMT
        // Validators are indexed in the JMT by their public key,
        // and there is a separate key containing the list of all validator keys.
        let mut validator_list = Vec::new();
        for validator in &app_state.validators {
            let validator_key = validator.validator.identity_key.clone();

            // Delegations require knowing the rates for the
            // next epoch, so pre-populate with 0 reward => exchange rate 1 for
            // the current and next epochs.
            let cur_rate_data = RateData {
                identity_key: validator_key.clone(),
                epoch_index,
                validator_reward_rate: 0,
                validator_exchange_rate: 1_0000_0000, // 1 represented as 1e8
            };
            let next_rate_data = RateData {
                identity_key: validator_key.clone(),
                epoch_index: epoch_index + 1,
                validator_reward_rate: 0,
                validator_exchange_rate: 1_0000_0000, // 1 represented as 1e8
            };

            self.overlay
                .save_validator(validator.validator.clone(), cur_rate_data, next_rate_data)
                .await;
            validator_list.push(validator_key);
        }

        self.overlay.set_validator_list(validator_list).await;

        // Finally, record that there were no delegations in this block, so the data
        // isn't missing when we process the first epoch transition.
        self.overlay
            .set_delegation_changes(0u32.into(), Default::default())
            .await;

        Ok(())
    }

    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock) -> Result<()> {
        tracing::debug!("Staking: begin_block");

        let slashing_penalty = self.overlay.get_chain_params().await?.slashing_penalty;

        // For each validator identified as byzantine by tendermint, update its
        // status to be slashed.
        for evidence in begin_block.byzantine_validators.iter() {
            self.overlay
                .slash_validator(evidence, slashing_penalty)
                .await;
        }

        Ok(())
    }

    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        // Check that the transaction undelegates from at most one validator.
        let undelegation_identities = tx
            .undelegations()
            .map(|u| u.validator_identity.clone())
            .collect::<BTreeSet<_>>();

        if undelegation_identities.len() > 1 {
            return Err(anyhow!(
                "Transaction contains undelegations from multiple validators: {:?}",
                undelegation_identities
            ));
        }

        // Check that validator definitions are correctly signed and well-formed:
        for definition in tx.validator_definitions() {
            // First, check the signature:
            let definition_bytes = definition.validator.encode_to_vec();
            definition
                .validator
                .identity_key
                .0
                .verify(&definition_bytes, &definition.auth_sig)
                .context("Validator definition signature failed to verify")?;

            // Check that the funding streams do not exceed 100% commission (10000bps)
            let total_funding_bps = definition
                .validator
                .funding_streams
                .iter()
                .map(|fs| fs.rate_bps as u64)
                .sum::<u64>();

            if total_funding_bps > 10000 {
                return Err(anyhow::anyhow!(
                    "Validator defined {} bps of funding streams, greater than 10000bps = 100%",
                    total_funding_bps
                ));
            }
        }

        Ok(())
    }

    async fn check_tx_stateful(&self, _tx: &Transaction) -> Result<()> {
        Ok(())
    }

    async fn execute_tx(&mut self, tx: &Transaction) -> Result<()> {
        // Queue any (un)delegations for processing at the next epoch boundary.
        for action in &tx.transaction_body.actions {
            match action {
                Action::Delegate(d) => {
                    tracing::debug!(?d, "queuing delegation for next epoch");
                    self.delegation_changes.delegations.push(d.clone());
                }
                Action::Undelegate(u) => {
                    tracing::debug!(?u, "queuing undelegation for next epoch");
                    self.delegation_changes.undelegations.push(u.clone());
                }
                _ => {}
            }
        }

        // TODO: process validator definitions

        Ok(())
    }

    async fn end_block(&mut self, end_block: &abci::request::EndBlock) -> Result<()> {
        // Write the delegation changes for this block.
        self.overlay
            .set_delegation_changes(
                end_block.height.try_into().unwrap(),
                std::mem::take(&mut self.delegation_changes),
            )
            .await;

        // If this is an epoch boundary, updated rates need to be calculated and set.
        let cur_epoch = self.overlay.get_current_epoch().await?;
        let cur_height = self.overlay.get_block_height().await?;

        if cur_epoch.is_epoch_end(cur_height) {
            self.end_epoch(cur_epoch).await?;
        }

        Ok(())
    }
}

/// Extension trait providing read/write access to staking data.
///
/// TODO: should this be split into Read and Write traits?
#[async_trait]
pub trait View: WriteOverlayExt + Send + Sync {
    async fn current_base_rate(&self) -> Result<BaseRateData> {
        self.get_domain("staking/base_rate/current".into())
            .await
            .map(|rate_data| rate_data.expect("rate data must be set after init_chain"))
    }

    async fn next_base_rate(&self) -> Result<BaseRateData> {
        self.get_domain("staking/base_rate/next".into())
            .await
            .map(|rate_data| rate_data.expect("rate data must be set after init_chain"))
    }

    #[instrument(skip(self))]
    async fn set_base_rates(&self, current: BaseRateData, next: BaseRateData) {
        tracing::debug!("setting base rates");
        self.put_domain("staking/base_rate/current".into(), current)
            .await;
        self.put_domain("staking/base_rate/next".into(), next).await;
    }

    async fn current_validator_rate(&self, identity_key: &IdentityKey) -> Result<Option<RateData>> {
        self.get_domain(format!("staking/validators/{}/rate/current", identity_key).into())
            .await
    }

    async fn next_validator_rate(&self, identity_key: &IdentityKey) -> Result<Option<RateData>> {
        self.get_domain(format!("staking/validators/{}/rate/next", identity_key).into())
            .await
    }

    #[instrument(skip(self))]
    async fn set_validator_rates(
        &self,
        identity_key: &IdentityKey,
        current_rates: RateData,
        next_rates: RateData,
    ) {
        tracing::debug!("setting validator rates");
        self.put_domain(
            format!("staking/validators/{}/rate/current", identity_key).into(),
            current_rates,
        )
        .await;
        self.put_domain(
            format!("staking/validators/{}/rate/next", identity_key).into(),
            next_rates,
        )
        .await;
    }

    async fn validator(&self, identity_key: &IdentityKey) -> Result<Option<Validator>> {
        self.get_domain(format!("staking/validators/{}", identity_key).into())
            .await
    }

    // Tendermint validators are referenced to us by their Tendermint consensus key,
    // but we reference them by their Penumbra identity key.
    async fn validator_by_consensus_key(&self, ck: &PublicKey) -> Result<Option<Validator>> {
        // We maintain an internal mapping of consensus keys to identity keys to make this
        // lookup more efficient.
        let identity_key: Option<IdentityKey> = self
            .get_domain(format!("staking/consensus_key/{}", ck.to_hex()).into())
            .await?;

        if identity_key.is_none() {
            return Ok(None);
        }

        let identity_key = identity_key.unwrap();

        self.validator(&identity_key).await
    }

    async fn slash_validator(&mut self, evidence: &Evidence, slashing_penalty: u64) -> Result<()> {
        let ck = tendermint::PublicKey::from_raw_ed25519(&evidence.validator.address)
            .ok_or_else(|| anyhow::anyhow!("invalid ed25519 consensus pubkey from tendermint"))
            .unwrap();

        let validator = self.validator_by_consensus_key(&ck).await?;

        tracing::info!(?validator, ?slashing_penalty, "slashing validator");

        // TOD: implement slashing

        Ok(())
    }

    // TODO(zbuc): does this make sense? will we always be saving cur&next rate data when we save a validator definition?
    async fn save_validator(
        &self,
        validator: Validator,
        current_rates: RateData,
        next_rates: RateData,
    ) {
        tracing::debug!(?validator);
        let id = validator.identity_key.clone();
        self.put_domain(format!("staking/validators/{}", id).into(), validator)
            .await;
        self.set_validator_rates(&id, current_rates, next_rates)
            .await
    }

    async fn validator_list(&self) -> Result<Vec<IdentityKey>> {
        Ok(self
            .get_domain("staking/validators/list".into())
            .await?
            .map(|list: ValidatorList| list.0)
            .unwrap_or(Vec::new()))
    }

    async fn set_validator_list(&self, validators: Vec<IdentityKey>) {
        self.put_domain("staking/validators/list".into(), ValidatorList(validators))
            .await;
    }

    async fn delegation_changes(&self, height: block::Height) -> Result<DelegationChanges> {
        Ok(self
            .get_domain(format!("staking/delegation_changes/{}", height.value()).into())
            .await?
            .ok_or_else(|| anyhow!("missing delegation changes for block {}", height))?)
    }

    async fn set_delegation_changes(&self, height: block::Height, changes: DelegationChanges) {
        self.put_domain(
            format!("staking/delegation_changes/{}", height.value()).into(),
            changes,
        )
        .await
    }
}

impl<T: WriteOverlayExt + Send + Sync> View for T {}
