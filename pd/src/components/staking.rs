use anyhow::Result;
use async_trait::async_trait;
use penumbra_proto::{stake as pb, Protobuf};
use penumbra_stake::{BaseRateData, Epoch, IdentityKey, RateData, Validator, ValidatorList};
use penumbra_transaction::{Action, Transaction};
use serde::{Deserialize, Serialize};
use tendermint::abci::{self, types::ValidatorUpdate};

use super::{Component, Overlay};
use crate::{genesis, PenumbraStore, WriteOverlayExt};

const DOMAIN_PREFIX: &str = "staking";

// Stub component
pub struct Staking {
    overlay: Overlay,
}

impl Staking {
    // TODO: does this make sense? will we always be saving cur&next rate data when we save a validator definition?
    async fn save_validator(
        &mut self,
        validator: Validator,
        cur_rate_data: RateData,
        next_rate_data: RateData,
    ) {
        let validator_key = validator.identity_key.clone();
        let def_key = format!("{}/validators/{}/definition", DOMAIN_PREFIX, validator_key);
        let cur_rate_key = format!(
            "{}/validators/{}/current_rate",
            DOMAIN_PREFIX, validator_key
        );
        let next_rate_key = format!("{}/validators/{}/next_rate", DOMAIN_PREFIX, validator_key);

        self.overlay
            .put_domain(cur_rate_key.into(), cur_rate_data)
            .await;
        self.overlay
            .put_domain(next_rate_key.into(), next_rate_data)
            .await;

        self.overlay
            .put_domain(def_key.into(), validator.clone())
            .await;
    }

    async fn end_epoch(&mut self) -> Result<()> {
        // calculate rate data for next rate, move previous next rate to cur rate,
        // and save the next rate data. ensure that non-Active validators maintain constant rates.

        // TODO: encapsulate the delegations logic
        // let mut delegation_changes = self.reader.delegation_changes(prev_epoch.index).await?;
        // for (id_key, delta) in &self
        //     .block_changes
        //     .as_ref()
        //     .expect("block_changes should be initialized during begin_block")
        //     .delegation_changes
        // {
        //     // TODO: does this need to be copied back to `self.block_changes.delegation_changes`
        //     // at the end of this method, so that `commit_block` will be able to use any
        //     // changes?
        //     *delegation_changes.entry(id_key.clone()).or_insert(0) += delta;
        // }
        let unbonding_epochs = self.overlay.get_chain_params().await?.unbonding_epochs;
        let active_validator_limit = self
            .overlay
            .get_chain_params()
            .await?
            .active_validator_limit;

        tracing::debug!("processing base rate");
        // We are transitioning to the next epoch, so set "cur_base_rate" to the previous "next_base_rate", and
        // update "next_base_rate".
        let current_base_rate: BaseRateData = self
            .overlay
            .get_domain(format!("{}/next_base_rate", DOMAIN_PREFIX).into())
            .await?
            .unwrap();
        /// FIXME: set this less arbitrarily, and allow this to be set per-epoch
        /// 3bps -> 11% return over 365 epochs, why not
        const BASE_REWARD_RATE: u64 = 3_0000;

        let next_base_rate = current_base_rate.next(BASE_REWARD_RATE);

        // rename to curr_rate so it lines up with next_rate (same # chars)
        tracing::debug!(curr_base_rate = ?current_base_rate);
        tracing::debug!(?next_base_rate);

        // Update these in the JMT:
        self.overlay
            .put_domain(
                format!("{}/cur_base_rate", DOMAIN_PREFIX).into(),
                current_base_rate,
            )
            .await;
        self.overlay
            .put_domain(
                format!("{}/next_base_rate", DOMAIN_PREFIX).into(),
                next_base_rate,
            )
            .await;

        // let mut staking_token_supply = self
        //     .reader
        //     .asset_lookup(*STAKING_TOKEN_ASSET_ID)
        //     .await?
        //     .map(|info| info.total_supply)
        //     .unwrap();

        // let mut next_rates = Vec::new();
        // let mut reward_notes = Vec::new();
        // let mut supply_updates = BTreeMap::new();

        // // steps (foreach validator):
        // // - get the total token supply for the validator's delegation tokens
        // // - process the updates to the token supply:
        // //   - collect all delegations occurring in previous epoch and apply them (adds to supply);
        // //   - collect all undelegations started in previous epoch and apply them (reduces supply);
        // // - feed the updated (current) token supply into current_rates.voting_power()
        // // - persist both the current voting power and the current supply
        // //
        // for v in &mut self.cache.validator_set {
        //     let validator = v.1;
        //     let current_rate = validator.rate_data.clone();
        //     tracing::debug!(?validator, "processing validator rate updates");
        //     assert!(current_rate.epoch_index == current_epoch.index);

        //     let funding_streams = self
        //         .reader
        //         .funding_streams(validator.validator.identity_key.clone())
        //         .await?;

        //     let next_rate = current_rate.next(
        //         &next_base_rate,
        //         funding_streams.as_ref(),
        //         &validator.status.state,
        //     );
        //     assert!(next_rate.epoch_index == current_epoch.index + 1);
        //     let identity_key = validator.validator.identity_key.clone();

        //     let delegation_delta = delegation_changes.get(&identity_key).unwrap_or(&0i64);

        //     let delegation_amount = delegation_delta.abs() as u64;
        //     let unbonded_amount = current_rate.unbonded_amount(delegation_amount);

        //     let mut delegation_token_supply = self
        //         .reader
        //         .asset_lookup(identity_key.delegation_token().id())
        //         .await?
        //         .map(|info| info.total_supply)
        //         .unwrap_or(0);

        //     if *delegation_delta > 0 {
        //         // net delegation: subtract the unbonded amount from the staking token supply
        //         staking_token_supply = staking_token_supply.checked_sub(unbonded_amount).unwrap();
        //         delegation_token_supply = delegation_token_supply
        //             .checked_add(delegation_amount)
        //             .unwrap();
        //     } else {
        //         // net undelegation: add the unbonded amount to the staking token supply
        //         staking_token_supply = staking_token_supply.checked_add(unbonded_amount).unwrap();
        //         delegation_token_supply = delegation_token_supply
        //             .checked_sub(delegation_amount)
        //             .unwrap();
        //     }

        //     // update the delegation token supply
        //     supply_updates.insert(
        //         identity_key.delegation_token().id(),
        //         (
        //             identity_key.delegation_token().denom(),
        //             delegation_token_supply,
        //         ),
        //     );
        //     let voting_power =
        //         current_rate.voting_power(delegation_token_supply, &current_base_rate);
        //     tracing::debug!(?voting_power);

        //     // Update the status of the validator within the validator set
        //     // with the newly starting epoch's calculated voting rate and power.
        //     validator.rate_data = current_rate.clone();
        //     validator.status.voting_power = voting_power;

        //     // Only Active validators produce commission rewards
        //     if validator.status.state == ValidatorState::Active {
        //         // distribute validator commission
        //         for stream in funding_streams {
        //             let commission_reward_amount = stream.reward_amount(
        //                 delegation_token_supply,
        //                 &next_base_rate,
        //                 &current_base_rate,
        //             );

        //             reward_notes.push((commission_reward_amount, stream.address));
        //         }
        //     }

        //     // rename to curr_rate so it lines up with next_rate (same # chars)
        //     let delegation_denom = identity_key.delegation_token().denom();
        //     tracing::debug!(curr_rate = ?current_rate);
        //     tracing::debug!(?next_rate);
        //     tracing::debug!(?delegation_delta);
        //     tracing::debug!(?delegation_token_supply);
        //     tracing::debug!(?delegation_denom);

        //     // Update the validator voting power and rates in the db
        //     next_rates.push((next_rate, voting_power));
        // }

        // tracing::debug!(?staking_token_supply);
        // supply_updates.insert(
        //     *STAKING_TOKEN_ASSET_ID,
        //     (STAKING_TOKEN_DENOM.clone(), staking_token_supply),
        // );

        // // State transitions on epoch change are handled here
        // // after all rates have been calculated
        // self.process_epoch_transitions(active_validator_limit, unbonding_epochs)?;

        Ok(())
    }

    /// Called during `end_epoch`. Will perform state transitions to validators based
    /// on changes to voting power that occurred in this epoch.
    pub fn process_epoch_transitions(
        &mut self,
        active_validator_limit: u64,
        unbonding_epochs: u64,
    ) -> Result<()> {
        // Sort the next validator states by voting power.
        // Dislike this clone, but the borrow checker was complaining about the loop modifying itself
        // when I tried using the validators_info() iterator.
        // let mut validators_info = self
        //     .cache
        //     .validator_set
        //     .iter()
        //     .map(|(_, v)| (v.clone()))
        //     .collect::<Vec<_>>();
        // validators_info.sort_by(|a, b| {
        //     a.borrow()
        //         .status
        //         .voting_power
        //         .cmp(&b.borrow().status.voting_power)
        // });
        // let top_validators = validators_info
        //     .iter()
        //     .take(active_validator_limit as usize)
        //     .map(|v| v.borrow().validator.identity_key.clone())
        //     .collect::<Vec<_>>();
        // for vi in &validators_info {
        //     let validator_status = &vi.borrow().status.clone();
        //     if validator_status.state == ValidatorState::Inactive
        //         || matches!(
        //             validator_status.state,
        //             ValidatorState::Unbonding { unbonding_epoch: _ }
        //         )
        //     {
        //         // If an Inactive or Unbonding validator is in the top `active_validator_limit` based
        //         // on voting power and the delegation pool has a nonzero balance,
        //         // then the validator should be moved to the Active state.
        //         if top_validators.contains(&validator_status.identity_key) {
        //             // TODO: How do we check the delegation pool balance here?
        //             // https://github.com/penumbra-zone/penumbra/issues/445
        //             self.activate_validator(vi.borrow().validator.consensus_key.clone())?;
        //         }
        //     } else if validator_status.state == ValidatorState::Active {
        //         // An Active validator could also be displaced and move to the
        //         // Unbonding state.
        //         if !top_validators.contains(&validator_status.identity_key) {
        //             self.unbond_validator(
        //                 vi.borrow().validator.consensus_key.clone(),
        //                 self.epoch().index + unbonding_epochs,
        //             )?;
        //         }
        //     }

        //     // An Unbonding validator can become Inactive if the unbonding period expires
        //     // and the validator is still in Unbonding state
        //     if let ValidatorState::Unbonding { unbonding_epoch } = validator_status.state {
        //         if unbonding_epoch <= self.epoch().index {
        //             self.deactivate_validator(vi.borrow().validator.consensus_key.clone())?;
        //         }
        //     };
        // }

        Ok(())
    }

    // Returns the list of validator updates formatted for inclusion in the Tendermint `EndBlockResponse`
    pub async fn tm_validator_updates(&self) -> Result<Vec<ValidatorUpdate>> {
        // TODO: impl
        //
        // TODO: It could be more efficient to only return the power of
        // updated validators. This is difficult because of potentially newly added validators
        // that have never been reported to Tendermint.
        // let tm_validator_updates = self
        //     .validators_info()
        //     .map(|v| {
        //         let v = v.borrow();
        //         // if the validator is non-Active, set their voting power as
        //         // returned to Tendermint to 0. Only Active validators report
        //         // voting power to Tendermint.
        //         tracing::debug!(?v, "calculating validator power in end_block");
        //         let power = if v.status.state == ValidatorState::Active {
        //             v.status.voting_power as u64
        //         } else {
        //             0
        //         };
        //         let validator = &v.validator;
        //         let pub_key = validator.consensus_key;
        //         Ok((
        //             validator.clone(),
        //             tendermint::abci::types::ValidatorUpdate {
        //                 pub_key,
        //                 power: power.try_into()?,
        //             },
        //         ))
        //     })
        //     // There has *got* to be a better way to do this.
        //     .collect::<Result<Vec<(_, _)>>>()?
        //     .iter()
        //     .cloned()
        //     .unzip();

        Ok(Vec::new())
    }
}

#[async_trait]
impl Component for Staking {
    async fn new(overlay: Overlay) -> Result<Self> {
        Ok(Self { overlay })
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
            base_exchange_rate: 1,
        };
        let next_base_rate = BaseRateData {
            epoch_index: epoch_index + 1,
            base_reward_rate: 0,
            base_exchange_rate: 1,
        };
        self.overlay
            .put_domain(
                format!("{}/cur_base_rate", DOMAIN_PREFIX).into(),
                cur_base_rate,
            )
            .await;
        self.overlay
            .put_domain(
                format!("{}/next_base_rate", DOMAIN_PREFIX).into(),
                next_base_rate,
            )
            .await;

        // Add initial validators to the JMT
        // Validators are indexed in the JMT by their public key,
        // and there is a separate key containing the list of all validator keys.
        let mut validator_keys = Vec::new();
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

            self.save_validator(validator.validator.clone(), cur_rate_data, next_rate_data)
                .await;
            validator_keys.push(validator_key);
        }

        self.overlay
            .put_domain(
                format!("{}/validators/keys", DOMAIN_PREFIX).into(),
                ValidatorList(validator_keys),
            )
            .await;

        Ok(())
    }

    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock) -> Result<()> {
        tracing::debug!("Staking: begin_block");
        let cur_epoch = self.overlay.get_current_epoch().await?;
        // TODO: need to write proto impl for BlockChanges
        // self.overlay.put_domain(
        //     format!("staking/block_changes/{}", block_height).into(),
        //     block_changes,
        // );

        Ok(())
    }

    fn check_tx_stateless(tx: &Transaction) -> Result<()> {
        for action in tx.transaction_body().actions {
            match action {
                Action::Delegate(delegate) => {
                    // There are currently no stateless verification checks than the ones implied by
                    // the binding signature.
                    // TODO: impl?
                }
                Action::Undelegate(undelegate) => {
                    // TODO: impl?
                    // if undelegation.is_none() {
                    //     undelegation = Some(undelegate);
                    // } else {
                    //     return Err(anyhow::anyhow!("Multiple undelegations in one transaction"));
                    // }
                }
                Action::ValidatorDefinition(validator) => {
                    // TODO: impl?
                    // Perform stateless checks that the validator definition is valid.

                    // Validate that the transaction signature is valid and signed by the
                    // validator's identity key.
                    // let protobuf_serialized: ProtoValidator = validator.validator.clone().into();
                    // let v_bytes = protobuf_serialized.encode_to_vec();
                    // validator
                    //     .validator
                    //     .identity_key
                    //     .0
                    //     .verify(&v_bytes, &validator.auth_sig)
                    //     .context("validator definition signature failed to verify")?;

                    // // Validate that the definition's funding streams do not exceed 100% (10000bps)
                    // let total_funding_bps = validator
                    //     .validator
                    //     .funding_streams
                    //     // TODO: possible to remove this clone?
                    //     .clone()
                    //     .into_iter()
                    //     .map(|stream| stream.rate_bps as u64)
                    //     .sum::<u64>();

                    // if total_funding_bps > 10000 {
                    //     return Err(anyhow::anyhow!(
                    //         "Total validator definition funding streams exceeds 100%"
                    //     ));
                    // }

                    // // TODO: Any other stateless checks to apply to validator definitions?

                    // validator_definitions.push(validator);
                }
                _ => {
                    // Not an action handled by the staking component
                }
            }
        }
        Ok(())
    }

    async fn check_tx_stateful(&self, _tx: &Transaction) -> Result<()> {
        // TODO: impl?
        // // Tally the delegations and undelegations
        // let mut delegation_changes = BTreeMap::new();
        // for d in &transaction.delegations {
        //     let rate_data = self
        //         .next_rate_data_rx()
        //         .borrow()
        //         .get(&d.validator_identity)
        //         .ok_or_else(|| {
        //             anyhow::anyhow!("Unknown validator identity {}", d.validator_identity)
        //         })?
        //         .clone();

        //     // Check whether the epoch is correct first, to give a more helpful
        //     // error message if it's wrong.
        //     if d.epoch_index != rate_data.epoch_index {
        //         return Err(anyhow::anyhow!(
        //             "Delegation was prepared for next epoch {} but the next epoch is {}",
        //             d.epoch_index,
        //             rate_data.epoch_index
        //         ));
        //     }

        //     // Check whether the delegation is for a slashed validator
        //     if let Some(v) = block_validators
        //         .clone()
        //         .find(|v| v.borrow().validator.identity_key == d.validator_identity)
        //     {
        //         if v.borrow().status.state == ValidatorState::Slashed {
        //             return Err(anyhow::anyhow!(
        //                 "Delegation to slashed validator {}",
        //                 d.validator_identity
        //             ));
        //         }
        //     };

        //     // For delegations, we enforce correct computation (with rounding)
        //     // of the *delegation amount based on the unbonded amount*, because
        //     // users (should be) starting with the amount of unbonded stake they
        //     // wish to delegate, and computing the amount of delegation tokens
        //     // they receive.
        //     //
        //     // The direction of the computation matters because the computation
        //     // involves rounding, so while both
        //     //
        //     // (unbonded amount, rates) -> delegation amount
        //     // (delegation amount, rates) -> unbonded amount
        //     //
        //     // should give approximately the same results, they may not give
        //     // exactly the same results.
        //     let expected_delegation_amount = rate_data.delegation_amount(d.unbonded_amount);

        //     if expected_delegation_amount == d.delegation_amount {
        //         // The delegation amount is added to the delegation token supply.
        //         *delegation_changes
        //             .entry(d.validator_identity.clone())
        //             .or_insert(0) += i64::try_from(d.delegation_amount).unwrap();
        //     } else {
        //         return Err(anyhow::anyhow!(
        //             "Given {} unbonded stake, expected {} delegation tokens but description produces {}",
        //             d.unbonded_amount,
        //             expected_delegation_amount,
        //             d.delegation_amount
        //         ));
        //     }
        // }
        // if let Some(ref u) = transaction.undelegation {
        //     let rate_data = self
        //         .next_rate_data_rx()
        //         .borrow()
        //         .get(&u.validator_identity)
        //         .ok_or_else(|| {
        //             anyhow::anyhow!("Unknown validator identity {}", u.validator_identity)
        //         })?
        //         .clone();

        //     // Check whether the epoch is correct first, to give a more helpful
        //     // error message if it's wrong.
        //     if u.epoch_index != rate_data.epoch_index {
        //         return Err(anyhow::anyhow!(
        //             "Undelegation was prepared for next epoch {} but the next epoch is {}",
        //             u.epoch_index,
        //             rate_data.epoch_index
        //         ));
        //     }

        //     // For undelegations, we enforce correct computation (with rounding)
        //     // of the *unbonded amount based on the delegation amount*, because
        //     // users (should be) starting with the amount of delegation tokens they
        //     // wish to undelegate, and computing the amount of unbonded stake
        //     // they receive.
        //     //
        //     // The direction of the computation matters because the computation
        //     // involves rounding, so while both
        //     //
        //     // (unbonded amount, rates) -> delegation amount
        //     // (delegation amount, rates) -> unbonded amount
        //     //
        //     // should give approximately the same results, they may not give
        //     // exactly the same results.
        //     let expected_unbonded_amount = rate_data.unbonded_amount(u.delegation_amount);

        //     if expected_unbonded_amount == u.unbonded_amount {
        //         // TODO: in order to have exact tracking of the token supply, we probably
        //         // need to change this to record the changes to the unbonded stake and
        //         // the delegation token separately

        //         // The undelegation amount is subtracted from the delegation token supply.
        //         *delegation_changes
        //             .entry(u.validator_identity.clone())
        //             .or_insert(0) -= i64::try_from(u.delegation_amount).unwrap();
        //     } else {
        //         return Err(anyhow::anyhow!(
        //             "Given {} delegation tokens, expected {} unbonded stake but description produces {}",
        //             u.delegation_amount,
        //             expected_unbonded_amount,
        //             u.unbonded_amount,
        //         ));
        //     }
        // }

        // // Check that the sequence numbers of newly added validators are correct.
        // //
        // // Resolution of conflicting validator definitions is performed later in `end_block` after
        // // they've all been received.
        // let mut validator_definitions = Vec::new();
        // for v in &transaction.validator_definitions {
        //     let existing_v: Vec<&ValidatorInfo> = block_validators
        //         .clone()
        //         .filter(|z| z.borrow().validator.identity_key == v.validator.identity_key)
        //         // vvv that looks weird to me and seems like a potential anti-pattern
        //         .map(|z| *z.borrow())
        //         .collect();

        //     if existing_v.is_empty() {
        //         // This is a new validator definition.
        //         validator_definitions.push(v.clone().into());
        //         continue;
        //     } else {
        //         // This is an existing validator definition. Ensure that the highest
        //         // existing sequence number is less than the new sequence number.
        //         let current_seq = existing_v.iter().map(|z| z.validator.sequence_number).max().ok_or_else(|| {anyhow::anyhow!("Validator with this ID key existed but had no existing sequence numbers")})?;
        //         if v.validator.sequence_number <= current_seq {
        //             return Err(anyhow::anyhow!(
        //                 "Expected sequence numbers to be increasing. Current sequence number is {}",
        //                 current_seq
        //             ));
        //         }
        //     }

        //     // the validator definition has now passed all verification checks, so add it to the list
        //     validator_definitions.push(v.clone().into());
        // }
        Ok(())
    }

    async fn execute_tx(&mut self, _tx: &Transaction) -> Result<()> {
        // Any new validator definitions are added to the known validator set.
        // for v in &transaction.validator_definitions {
        //     self.validator_set.add_validator_definition(v.clone());
        // }
        // self.validator_set
        //     .update_delegations(&transaction.delegation_changes);

        Ok(())
    }

    async fn end_block(&mut self, _end_block: &abci::request::EndBlock) -> Result<()> {
        // If this is an epoch boundary, updated rates need to be calculated and set.
        let cur_epoch = self.overlay.get_current_epoch().await?;
        let cur_height = self.overlay.get_block_height().await?;

        if cur_epoch.is_epoch_boundary(cur_height) {
            self.end_epoch().await?;
        }

        Ok(())
    }
}
