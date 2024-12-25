use crate::{
    component::{
        action_handler::ActionHandler, validator_handler::ValidatorDataRead,
        validator_handler::ValidatorManager,
    },
    rate::RateData,
    validator,
};
use anyhow::{ensure, Context, Result};
use async_trait::async_trait;
use cnidarium::StateWrite;
use decaf377_rdsa::VerificationKey;
use penumbra_sdk_proto::DomainType;

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

        if self.validator.funding_streams.len() > 8 {
            anyhow::bail!("validators can declare at most 8 funding streams")
        }

        // This prevents an attacker who compromises a validator identity signing key from locking
        // the validator in an enabled state permanently, instead making it so that the original
        // operator always has the option of disabling the validator permanently, regardless of what
        // the attacker does. This reduces the incentive to steal compromise validator signing keys,
        // because it reduces the expected payoff of such a compromise.
        if self.validator.sequence_number == u32::MAX && self.validator.enabled {
            anyhow::bail!("validators must be disabled when their lifetime is over")
        }

        // Then, we check the signature:
        let definition_bytes = self.validator.encode_to_vec();
        VerificationKey::try_from(self.validator.identity_key.0)
            .and_then(|vk| vk.verify(&definition_bytes, &self.auth_sig))
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

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // These checks all formerly happened in the `check_stateful` method,
        // if profiling shows that they cause a bottleneck we could (CAREFULLY)
        // move some of them back.
        let new_validator = &self.validator;

        // Check that the sequence numbers of the updated validators is correct...
        // Check whether we are redefining an existing validator.
        let prev_definition = state
            .get_validator_definition(&new_validator.identity_key)
            .await?;

        if let Some(prev_validator) = &prev_definition {
            // Ensure that the highest existing sequence number is less than
            // the new sequence number.
            // Ensure that the sequence number keeps increasing.
            let old_seq = prev_validator.sequence_number;
            let new_seq = new_validator.sequence_number;
            ensure!(
                new_seq > old_seq,
                "definition sequence number must increase (given {}, but previous definition sequence number is {})",
                new_seq,
                old_seq,
            );
        }

        // Check if the consensus key is known, and if so, that it is by the
        // validator that declares it in this definition.
        if let Some(ck_owner) = state
            .get_validator_definition_by_consensus_key(&new_validator.consensus_key)
            .await?
        {
            // If we detect that the new definition tries to squat someone else's
            // consensus key, we MUST reject this definition:
            //
            // 1. It prevents someone from declaring an (app-level) validator that
            // "piggybacks" on the actual behavior of someone else's validator.
            //
            // 2. If we submit a validator update to CometBFT that
            // includes duplicate consensus keys, CometBFT gets confused
            // and hangs.
            ensure!(
                ck_owner.identity_key == new_validator.identity_key,
                "consensus key {:?} is already in use by validator {}",
                new_validator.consensus_key,
                ck_owner.identity_key,
            );
        }

        /* ------------ execution ----------- */
        // If the validator is already defined, we update the definition.
        // Otherwise, we add the new validator and "prime" its state.
        if prev_definition.is_some() {
            state
                .update_validator_definition(new_validator.clone())
                .await
                .context(
                    "should be able to update validator during validator definition execution",
                )?;
        } else {
            let validator_key = new_validator.identity_key;

            // The validator starts with a reward rate of 0 and an exchange rate
            // of 1, expressed in bps^2 (i.e. 1_0000_0000 is 1.0).
            let initial_rate_data = RateData {
                identity_key: validator_key,
                validator_reward_rate: 0u128.into(),
                validator_exchange_rate: 1_0000_0000u128.into(),
            };

            state
                .add_validator(new_validator.clone(), initial_rate_data)
                .await
                .context("should be able to add validator during validator definition execution")?;
        }

        Ok(())
    }
}
