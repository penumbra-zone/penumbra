use anyhow::{Context, Result};
use penumbra_storage::State;
use penumbra_transaction::Transaction;
use std::sync::Arc;

use penumbra_proto::{core::stake::v1alpha1::ValidatorDefinition, Protobuf};

use crate::stake::{validator, StateReadExt as _};

pub(crate) fn check_stateless(
    definition: &ValidatorDefinition,
    context: Arc<Transaction>,
) -> Result<()> {
    // Check that validator definition is correctly signed and well-formed:
    let definition = validator::Definition::try_from(definition.clone())
        .context("supplied proto is not a valid definition")?;
    // First, check the signature:
    let definition_bytes = definition.validator.encode_to_vec();
    definition
        .validator
        .identity_key
        .0
        .verify(&definition_bytes, &definition.auth_sig)
        .context("validator definition signature failed to verify")?;

    // TODO(hdevalence) -- is this duplicated by the check during parsing?
    // Check that the funding streams do not exceed 100% commission (10000bps)
    let total_funding_bps = definition
        .validator
        .funding_streams
        .iter()
        .map(|fs| fs.rate_bps as u64)
        .sum::<u64>();

    if total_funding_bps > 10000 {
        return Err(anyhow::anyhow!(
            "validator defined {} bps of funding streams, greater than 10000bps = 100%",
            total_funding_bps
        ));
    }

    Ok(())
}

pub(crate) async fn check_stateful(
    definition: &ValidatorDefinition,
    state: Arc<State>,
    context: Arc<Transaction>,
) -> Result<()> {
    // Check that the sequence numbers of the updated validators is correct.
    let v = validator::Definition::try_from(definition.clone())
        .context("supplied proto is not a valid definition")?;

    // Check whether we are redefining an existing validator.
    if let Some(existing_v) = state.validator(&v.validator.identity_key).await? {
        // Ensure that the highest existing sequence number is less than
        // the new sequence number.
        let current_seq = existing_v.sequence_number;
        if v.validator.sequence_number <= current_seq {
            return Err(anyhow::anyhow!(
                "expected sequence numbers to be increasing: current sequence number is {}",
                current_seq
            ));
        }
    }

    // Check whether the consensus key has already been used by another validator.
    if let Some(existing_v) = state
        .validator_by_consensus_key(&v.validator.consensus_key)
        .await?
    {
        if v.validator.identity_key != existing_v.identity_key {
            // This is a new validator definition, but the consensus
            // key it declares is already in use by another validator.
            //
            // Rejecting this is important for two reasons:
            //
            // 1. It prevents someone from declaring an (app-level)
            // validator that "piggybacks" on the actual behavior of someone
            // else's validator.
            //
            // 2. If we submit a validator update to Tendermint that
            // includes duplicate consensus keys, Tendermint gets confused
            // and hangs.
            return Err(anyhow::anyhow!(
                "consensus key {:?} is already in use by validator {}",
                v.validator.consensus_key,
                existing_v.identity_key,
            ));
        }
    }

    // the validator definition has now passed all verification checks
    Ok(())
}
