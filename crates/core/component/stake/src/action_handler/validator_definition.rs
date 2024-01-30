use anyhow::{Context, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use penumbra_sct::component::EpochRead;

use std::sync::Arc;

use penumbra_proto::DomainType;

use crate::{
    action_handler::ActionHandler, component::StakingImpl as _, rate::RateData, validator, StateReadExt as _
};

#[async_trait]
impl ActionHandler for validator::Definition {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // First, we check that the validator website/name/description does not
        // exceed 70, 140, and 280 characters respectively. We use guard statements
        // so that clients can display actionable error messages.
        if self.validator.website.len() > 70 {
            anyhow::bail!("validator website field must be less than 70 characters")
        }

        if self.validator.name.len() > 140 {
            anyhow::bail!("validator name must be less than 140 characters")
        }

        if self.validator.description.len() > 280 {
            anyhow::bail!("validator description must be less than 280 characters")
        }

        // Then, we check the signature:
        let definition_bytes = self.validator.encode_to_vec();
        self.validator
            .identity_key
            .0
            .verify(&definition_bytes, &self.auth_sig)
            .context("validator definition signature failed to verify")?;

        let total_funding_bps = self
            .validator
            .funding_streams
            .iter()
            .map(|fs| fs.rate_bps() as u64)
            .sum::<u64>();

        if total_funding_bps > 10_000 {
            anyhow::bail!(
                "validator defined {} bps of funding streams, greater than 10000bps (= 100%)",
                total_funding_bps
            );
        }

        Ok(())
    }

    async fn check_stateful<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        let v = self;

        // Check that the sequence numbers of the updated validators is correct...
        // Check whether we are redefining an existing validator.
        if let Some(existing_v) = state.validator(&v.validator.identity_key).await? {
            // Ensure that the highest existing sequence number is less than
            // the new sequence number.
            let current_seq = existing_v.sequence_number;
            if v.validator.sequence_number <= current_seq {
                anyhow::bail!(
                    "expected sequence numbers to be increasing: current sequence number is {}",
                    current_seq
                );
            }
        }

        // Check whether the consensus key has already been used by another validator.
        if let Some(existing_v) = state
            .validator_by_consensus_key(&v.validator.consensus_key)
            .await?
        {
            if v.validator.identity_key != existing_v.identity_key {
                // This is a new validator definition, but the consensus it declares
                // is used by another validator. We MUST reject this definition:
                //
                // 1. It prevents someone from declaring an (app-level) validator that
                // "piggybacks" on the actual behavior of someone else's validator.
                //
                // 2. If we submit a validator update to Tendermint that
                // includes duplicate consensus keys, Tendermint gets confused
                // and hangs.
                anyhow::bail!(
                    "consensus key {:?} is already in use by validator {}",
                    v.validator.consensus_key,
                    existing_v.identity_key,
                );
            }
        }

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let v = self;

        let current_epoch = state
            .current_epoch()
            .await
            .context("should be able to get current epoch during validator definition execution")?;

        let validator_exists = state
            .validator(&v.validator.identity_key)
            .await
            .context("should be able to fetch validator during validator definition execution")?
            .is_some();

        if validator_exists {
            state.update_validator(v.validator.clone()).await.context(
                "should be able to update validator during validator definition execution",
            )?;
        } else {
            // This is a new validator definition. We prime the validator's
            // rate data with an initial exchange rate of 1:1.
            let validator_key = v.validator.identity_key;

            let initial_rate_data = RateData {
                identity_key: validator_key,
                epoch_index: current_epoch.index,
                validator_reward_rate: 0u128.into(),
                validator_exchange_rate: 1_0000_0000u128.into(), // 1 represented as 1e8
            };

            state
                .add_validator(v.validator.clone(), initial_rate_data)
                .await
                .context("should be able to add validator during validator definition execution")?;
        }

        Ok(())
    }
}
