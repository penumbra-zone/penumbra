// Implementation of a pd component for the staking system.
use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use penumbra_chain::{genesis, Epoch, View as _};
use penumbra_component::Component;
use penumbra_crypto::{DelegationToken, IdentityKey, STAKING_TOKEN_ASSET_ID};
use penumbra_proto::Protobuf;
use penumbra_shielded_pool::{CommissionAmount, CommissionAmounts, View as _};
use penumbra_storage::{State, StateExt};
use penumbra_transaction::{
    action::{Delegate, Undelegate},
    Action, Transaction,
};
use sha2::{Digest, Sha256};
use tendermint::{
    abci::{
        self,
        types::{Evidence, LastCommitInfo, ValidatorUpdate},
    },
    block, PublicKey,
};
use tracing::instrument;

use crate::{
    rate::{BaseRateData, RateData},
    validator::{self, Validator},
    DelegationChanges, Uptime,
};

// Max validator power is 1152921504606846975 (i64::MAX / 8)
// https://github.com/tendermint/tendermint/blob/master/types/validator_set.go#L25
const MAX_VOTING_POWER: i64 = 1152921504606846975;

// Staking component
pub struct Staking {
    state: State,
    /// Delegation changes accumulated over the course of this block, to be
    /// persisted at the end of the block for processing at the end of the next
    /// epoch.
    delegation_changes: DelegationChanges,
    /// List of changes to the tendermint validator set accumulated throughout
    /// this block, to be returned during `EndBlock`.
    tm_validator_updates: Vec<ValidatorUpdate>,
}

impl Staking {
    #[instrument(skip(self))]
    fn update_tm_validator_power(&mut self, ck: &PublicKey, power: u64) -> Result<()> {
        // TODO: would it be more sensible to have `tm_validator_updates` be a
        // BTreeMap instead, and construct the `ValidatorUpdate` vec on-demand?
        let existing = self
            .tm_validator_updates
            .iter()
            .enumerate()
            .find(|(_i, v)| v.pub_key == *ck);

        match existing {
            Some((i, v)) => {
                // mem::replace?
                let mut v2 = v.clone();
                v2.power = power.try_into()?;
                self.tm_validator_updates[i] = v2;
            }
            None => {
                self.tm_validator_updates.append(&mut vec![ValidatorUpdate {
                    pub_key: ck.clone(),
                    power: power.try_into()?,
                }]);
            }
        };

        Ok(())
    }

    #[instrument(skip(self, epoch_to_end), fields(index = epoch_to_end.index))]
    async fn end_epoch(&mut self, epoch_to_end: Epoch) -> Result<()> {
        // calculate rate data for next rate, move previous next rate to cur rate,
        // and save the next rate data. ensure that non-Active validators maintain constant rates.
        let mut delegations_by_validator = BTreeMap::<IdentityKey, Vec<Delegate>>::new();
        let mut undelegations_by_validator = BTreeMap::<IdentityKey, Vec<Undelegate>>::new();
        for height in epoch_to_end.start_height().value()..=epoch_to_end.end_height().value() {
            let changes = self
                .state
                .delegation_changes(height.try_into().unwrap())
                .await?;
            for d in changes.delegations {
                delegations_by_validator
                    .entry(d.validator_identity.clone())
                    .or_insert_with(Vec::new)
                    .push(d);
            }
            for u in changes.undelegations {
                undelegations_by_validator
                    .entry(u.validator_identity.clone())
                    .or_insert_with(Vec::new)
                    .push(u);
            }
        }
        tracing::debug!(
            total_delegations = ?delegations_by_validator
                .iter()
                .map(|(_, v)| v.len())
                .sum::<usize>(),
            total_undelegations = ?undelegations_by_validator
                .iter()
                .map(|(_, v)| v.len())
                .sum::<usize>(),
        );

        let chain_params = self.state.get_chain_params().await?;
        let unbonding_epochs = chain_params.unbonding_epochs;
        let active_validator_limit = chain_params.active_validator_limit;

        tracing::debug!("processing base rate");
        // We are transitioning to the next epoch, so set "cur_base_rate" to the previous "next_base_rate", and
        // update "next_base_rate".
        let current_base_rate = self.state.next_base_rate().await?;

        let next_base_rate = current_base_rate.next(chain_params.base_reward_rate);

        // rename to curr_rate so it lines up with next_rate (same # chars)
        tracing::debug!(curr_base_rate = ?current_base_rate);
        tracing::debug!(?next_base_rate);

        // Update the base rates in the JMT:
        self.state
            .set_base_rates(current_base_rate.clone(), next_base_rate.clone())
            .await;

        let mut commission_amounts = Vec::new();
        let validator_list = self.state.validator_list().await?;
        for v in &validator_list {
            let validator = self.state.validator(v).await?.ok_or_else(|| {
                anyhow::anyhow!("validator had ID in validator_list but not found in JMT")
            })?;
            // The old epoch's "next rate" is now the "current rate".
            let current_rate = self.state.next_validator_rate(v).await?.ok_or_else(|| {
                anyhow::anyhow!("validator had ID in validator_list but rate not found in JMT")
            })?;

            let validator_state = self.state.validator_state(v).await?.ok_or_else(|| {
                anyhow::anyhow!("validator had ID in validator_list but state not found in JMT")
            })?;
            tracing::debug!(?validator, "processing validator rate updates");

            let funding_streams = validator.funding_streams;

            let next_rate =
                current_rate.next(&next_base_rate, funding_streams.as_ref(), &validator_state);
            assert!(next_rate.epoch_index == epoch_to_end.index + 2);

            let total_delegations = delegations_by_validator
                .get(&validator.identity_key)
                .into_iter()
                .flat_map(|ds| ds.iter().map(|d| d.delegation_amount))
                .sum::<u64>();
            let total_undelegations = undelegations_by_validator
                .get(&validator.identity_key)
                .into_iter()
                .flat_map(|us| us.iter().map(|u| u.delegation_amount))
                .sum::<u64>();
            let delegation_delta = (total_delegations as i64) - (total_undelegations as i64);

            tracing::debug!(
                validator = ?validator.identity_key,
                total_delegations,
                total_undelegations,
                delegation_delta
            );

            let abs_unbonded_amount =
                current_rate.unbonded_amount(delegation_delta.abs() as u64) as i64;
            let staking_delta = if delegation_delta >= 0 {
                // Net delegation: subtract the unbonded amount from the staking token supply
                -abs_unbonded_amount
            } else {
                // Net undelegation: add the unbonded amount to the staking token supply
                abs_unbonded_amount
            };

            // update the delegation token supply in the JMT
            self.state
                .update_token_supply(&DelegationToken::from(v).id(), delegation_delta)
                .await?;
            // update the staking token supply in the JMT
            self.state
                .update_token_supply(&STAKING_TOKEN_ASSET_ID, staking_delta)
                .await?;

            let delegation_token_supply = self
                .state
                .token_supply(&DelegationToken::from(v).id())
                .await?
                .expect("delegation token should be known");

            // Calculate the voting power in the newly beginning epoch
            let voting_power =
                current_rate.voting_power(delegation_token_supply, &current_base_rate);
            tracing::debug!(?voting_power);

            // Update the status of the validator within the validator set
            // with the newly starting epoch's calculated voting rate and power.
            self.state
                .set_validator_rates(v, current_rate.clone(), next_rate.clone())
                .await;
            self.state.set_validator_power(v, voting_power).await?;

            // Only Active validators produce commission rewards
            // The validator *may* drop out of Active state during the next epoch,
            // but the commission rewards for the ending epoch in which it was Active
            // should still be rewarded.
            if validator_state == validator::State::Active {
                // distribute validator commission
                for stream in funding_streams {
                    let commission_reward_amount = stream.reward_amount(
                        delegation_token_supply,
                        &next_base_rate,
                        &current_base_rate,
                    );

                    // A note needs to be minted by the ShieldedPool component. Add it to the
                    // JMT here so it can be processed during the ShieldedPool's end_block phase.
                    commission_amounts.push(CommissionAmount {
                        amount: commission_reward_amount,
                        destination: stream.address,
                    });
                }
            }

            // rename to curr_rate so it lines up with next_rate (same # chars)
            let delegation_denom = DelegationToken::from(v).denom();
            tracing::debug!(curr_rate = ?current_rate);
            tracing::debug!(?next_rate);
            tracing::debug!(?delegation_delta);
            tracing::debug!(?delegation_token_supply);
            tracing::debug!(?delegation_denom);
        }

        // Now that all the voting power has been calculated for the upcoming epoch,
        // we can determine which validators are Active for the next epoch.
        self.process_epoch_transitions(epoch_to_end, active_validator_limit, unbonding_epochs)
            .await?;

        // The pending delegation changes should be empty at the beginning of the next epoch.
        self.delegation_changes = Default::default();

        // Set the pending reward notes on the JMT for the current block height
        // so they can be processed by the ShieldedPool.
        self.state
            .set_commission_amounts(
                self.state.get_block_height().await?,
                CommissionAmounts {
                    notes: commission_amounts,
                },
            )
            .await;

        Ok(())
    }

    /// Called during `end_epoch`. Will perform state transitions to validators based
    /// on changes to voting power that occurred in this epoch.
    pub async fn process_epoch_transitions(
        &mut self,
        epoch_to_end: Epoch,
        active_validator_limit: u64,
        unbonding_epochs: u64,
    ) -> Result<()> {
        // Sort the next validator states by voting power.
        struct VPower {
            identity_key: IdentityKey,
            power: u64,
            state: validator::State,
        }

        let mut validator_power_list = Vec::new();
        for v in self.state.validator_list().await?.iter() {
            let power = self
                .state
                .validator_power(v)
                .await?
                .ok_or_else(|| anyhow::anyhow!("validator missing power"))?;
            let state = self
                .state
                .validator_state(v)
                .await?
                .ok_or_else(|| anyhow::anyhow!("validator missing state"))?;
            validator_power_list.push(VPower {
                identity_key: v.clone(),
                power,
                state,
            });
        }

        // Sort by voting power descending.
        validator_power_list.sort_by(|a, b| b.power.cmp(&a.power));

        // Grab the top `active_validator_limit` validators
        let top_validators = validator_power_list
            .iter()
            .take(active_validator_limit as usize)
            // The top validators should never include a validator without voting power
            .filter(|v| v.power > 0)
            .map(|v| v.identity_key.clone())
            .collect::<Vec<_>>();

        // Iterate every validator and update according to their state and voting power.
        for vp in &validator_power_list {
            if vp.state == validator::State::Inactive {
                // If an Inactive validator is in the top `active_validator_limit` based
                // on voting power and the delegation pool has a nonzero balance (meaning non-zero voting power),
                // then the validator should be moved to the Active state.
                if top_validators.contains(&vp.identity_key) {
                    tracing::debug!(identity_key = ?vp.identity_key, "validator is in top validators and will now enter active state");
                    self.state
                        .set_validator_state(&vp.identity_key, validator::State::Active)
                        .await?;
                    // Start tracking the validator's uptime as it becomes active
                    let uptime = Uptime::new(
                        self.state.get_block_height().await?,
                        self.state.signed_blocks_window_len().await? as usize,
                    );
                    self.state
                        .set_validator_uptime(&vp.identity_key, uptime)
                        .await;

                    // The validator should no longer be Unbonding if it was
                    self.state
                        .clear_validator_unbonding_status(&vp.identity_key)
                        .await?;

                    let validator = self
                        .state
                        .validator(&vp.identity_key)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("validator missing"))?;
                    // The now-Active validator should report its voting power to tendermint this block
                    self.update_tm_validator_power(&validator.consensus_key, vp.power)?;
                }
            } else if vp.state == validator::State::Active {
                // An Active validator could also be displaced and move to the
                // Inactive state and begin Unbonding.
                if !top_validators.contains(&vp.identity_key) {
                    tracing::debug!(identity_key = ?vp.identity_key, "validator left active set and will now enter unbonding");
                    // Unbonding the validator means that it can no longer participate
                    // in consensus, so its voting power is set to 0.
                    self.state.set_validator_power(&vp.identity_key, 0).await?;
                    self.state
                        .set_validator_state(&vp.identity_key, validator::State::Inactive)
                        .await?;
                    let validator = self
                        .state
                        .validator(&vp.identity_key)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("validator missing"))?;
                    // TODO: the validator needs to begin Unbonding
                    // validator::State::Unbonding {
                    //     unbonding_epoch: epoch_to_end.index + unbonding_epochs,
                    // },
                    // The now-Unbonding validator should report 0 voting power to tendermint this block
                    self.update_tm_validator_power(&validator.consensus_key, 0)?;
                } else {
                    // This validator remains active, and we should report its latest voting
                    // power to tendermint.
                    let validator = self
                        .state
                        .validator(&vp.identity_key)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("validator missing"))?;
                    let power = self
                        .state
                        .validator_power(&vp.identity_key)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("active validator should have power"))?;
                    self.update_tm_validator_power(&validator.consensus_key, power)?;
                }
            }

            // An Unbonding validator can stop unbonding if the unbonding period expires
            // TODO: implement
            // if let validator::State::Unbonding { unbonding_epoch } = vp.state {
            //     if unbonding_epoch <= epoch_to_end.index {
            //         tracing::debug!(identity_key = ?vp.identity_key, "validator unbonding period over and validator entering inactive state");
            //         self.state
            //             .set_validator_state(&vp.identity_key, validator::State::Inactive)
            //             .await?;
            //     }
            // };
        }

        Ok(())
    }

    // Returns the list of validator updates formatted for inclusion in the Tendermint `EndBlockResponse`
    pub async fn tm_validator_updates(&self) -> Result<Vec<ValidatorUpdate>> {
        Ok(self.tm_validator_updates.clone())
    }

    #[instrument(skip(self, last_commit_info))]
    async fn track_uptime(&mut self, last_commit_info: &LastCommitInfo) -> Result<()> {
        // Note: this probably isn't the correct height for the LastCommitInfo,
        // which is about the *last* commit, but at least it'll be consistent,
        // which is all we need to count signatures.
        let height = self.state.get_block_height().await?;
        let params = self.state.get_chain_params().await?;

        // Build a mapping from addresses (20-byte truncated SHA256(pubkey)) to vote statuses.
        let did_address_vote = last_commit_info
            .votes
            .iter()
            .map(|vote| (vote.validator.address, vote.signed_last_block))
            .collect::<BTreeMap<[u8; 20], bool>>();

        // Since we don't have a lookup from "addresses" to identity keys,
        // iterate over our app's validators, and match them up with the vote data.
        for v in self.state.validator_list().await?.iter() {
            let info = self
                .state
                .validator_info(v)
                .await?
                .ok_or_else(|| anyhow::anyhow!("validator missing status"))?;

            if info.status.state == validator::State::Active {
                // for some reason last_commit_info has truncated sha256 hashes
                let ck_bytes = info.validator.consensus_key.to_bytes();
                let addr: [u8; 20] = Sha256::digest(&ck_bytes).as_slice()[0..20]
                    .try_into()
                    .unwrap();

                let voted = did_address_vote.get(&addr).cloned().unwrap_or(false);
                let mut uptime = self
                    .state
                    .validator_uptime(v)
                    .await?
                    .ok_or_else(|| anyhow!("missing uptime for active validator {}", v))?;

                tracing::debug!(
                    ?voted,
                    num_missed_blocks = ?uptime.num_missed_blocks(),
                    identity_key = ?v,
                    ?params.missed_blocks_maximum,
                    "recorded vote info"
                );

                uptime.mark_height_as_signed(height, voted).unwrap();
                if uptime.num_missed_blocks() as u64 >= params.missed_blocks_maximum {
                    tracing::info!(identity_key = ?v, "slashing for downtime");
                    self.slash_validator(info.validator, params.slashing_penalty_downtime_bps)
                        .await?;
                } else {
                    self.state.set_validator_uptime(v, uptime).await;
                }
            }
        }

        Ok(())
    }

    async fn slash_validator(&mut self, validator: Validator, slashing_penalty: u64) -> Result<()> {
        // Update the validator to return 0 power to Tendermint for this block.
        self.update_tm_validator_power(&validator.consensus_key, 0)?;

        self.state
            .slash_validator(validator, slashing_penalty)
            .await
    }

    /// Add a validator during genesis, which will start in Active
    /// state with power assigned.
    async fn add_genesis_validator(
        &mut self,
        validator: Validator,
        cur_rate_data: RateData,
        next_rate_data: RateData,
        power: u64,
    ) -> Result<()> {
        // Update the validator to return its power to Tendermint for this block.
        self.update_tm_validator_power(&validator.consensus_key, power)?;

        self.state
            .add_validator(
                validator.clone(),
                cur_rate_data,
                next_rate_data,
                // All genesis validators start in the "Active" state:
                validator::State::Active,
                power,
            )
            .await
    }

    /// Add a validator after genesis, which will start in Inactive
    /// state with no power assigned.
    async fn add_validator(
        &mut self,
        validator: Validator,
        cur_rate_data: RateData,
        next_rate_data: RateData,
    ) -> Result<()> {
        // We explicitly do not call `update_tm_validator_power` here,
        // as a post-genesis validator should not have power reported
        // to Tendermint until it becomes Active.
        self.state
            .add_validator(
                validator.clone(),
                cur_rate_data,
                next_rate_data,
                // All post-genesis validators start in the "Inactive" state:
                validator::State::Inactive,
                0,
            )
            .await
    }

    async fn slash_validator_by_evidence(&mut self, evidence: &Evidence) -> Result<()> {
        let ck = tendermint::PublicKey::from_raw_ed25519(&evidence.validator.address)
            .ok_or_else(|| anyhow::anyhow!("invalid ed25519 consensus pubkey from tendermint"))
            .unwrap();

        let validator = self
            .state
            .validator_by_consensus_key(&ck)
            .await?
            .ok_or_else(|| anyhow::anyhow!("attempted to slash validator not found in JMT"))?;

        let slashing_penalty = self
            .state
            .get_chain_params()
            .await?
            .slashing_penalty_misbehavior_bps;

        self.slash_validator(validator, slashing_penalty).await
    }
}

#[async_trait]
impl Component for Staking {
    #[instrument(name = "staking", skip(state))]
    async fn new(state: State) -> Self {
        Self {
            state,
            delegation_changes: Default::default(),
            tm_validator_updates: Default::default(),
        }
    }

    #[instrument(name = "staking", skip(self, app_state))]
    async fn init_chain(&mut self, app_state: &genesis::AppState) {
        let starting_height = self.state.get_block_height().await.unwrap();
        let starting_epoch = Epoch::from_height(
            starting_height,
            self.state.get_epoch_duration().await.unwrap(),
        );
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
        self.state
            .set_base_rates(cur_base_rate.clone(), next_base_rate)
            .await;

        let mut allocations_by_validator = HashMap::new();
        for allocation in &app_state.allocations {
            if allocation.amount == 0 {
                continue;
            }

            let amount = allocations_by_validator
                .entry(&allocation.denom)
                .or_insert(0);
            *amount += allocation.amount;
        }

        // Add initial validators to the JMT
        // Validators are indexed in the JMT by their public key,
        // and there is a separate key containing the list of all validator keys.
        for validator in &app_state.validators {
            // Parse the proto into a domain type.
            let validator = Validator::try_from(validator.clone()).unwrap();
            let validator_key = validator.identity_key.clone();

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

            // The initial allocations to the validator are not available on the JMT yet,
            // because the ShieldedPool component executes last.
            //
            // This means that we need to iterate the app_state to calculate the initial
            // delegation token allocations for the genesis validators, to determine voting power.
            let delegation_denom = DelegationToken::from(validator_key).denom().to_string();
            let total_delegation_tokens = allocations_by_validator
                .get(&delegation_denom)
                .copied()
                .unwrap_or(0);
            let power = cur_rate_data.voting_power(total_delegation_tokens, &cur_base_rate);

            self.add_genesis_validator(validator.clone(), cur_rate_data, next_rate_data, power)
                .await
                .unwrap();
            // We also need to start tracking uptime of the genesis validators:
            self.state
                .set_validator_uptime(
                    &validator.identity_key,
                    Uptime::new(0, app_state.chain_params.signed_blocks_window_len as usize),
                )
                .await;
        }

        // Finally, record that there were no delegations in this block, so the data
        // isn't missing when we process the first epoch transition.
        self.state
            .set_delegation_changes(0u32.into(), Default::default())
            .await;
    }

    #[instrument(name = "staking", skip(self, begin_block))]
    async fn begin_block(&mut self, begin_block: &abci::request::BeginBlock) {
        // For each validator identified as byzantine by tendermint, update its
        // state to be slashed.
        for evidence in begin_block.byzantine_validators.iter() {
            self.slash_validator_by_evidence(evidence).await.unwrap();
        }

        self.track_uptime(&begin_block.last_commit_info)
            .await
            .unwrap();
    }

    #[instrument(name = "staking", skip(tx))]
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

        // We prohibit actions other than `Spend`, `Delegate`, `Output` and `Undelegate` in
        // transactions that contain `Undelegate`, to avoid having to quarantine them.
        if undelegation_identities.len() == 1 {
            use Action::*;
            for action in tx.transaction_body().actions {
                if !matches!(action, Undelegate(_) | Delegate(_) | Spend(_) | Output(_)) {
                    return Err(anyhow::anyhow!("transaction contains an undelegation, but also contains an action other than Spend, Delegate, Output or Undelegate"));
                }
            }
        }

        // Check that validator definitions are correctly signed and well-formed:
        for definition in tx.validator_definitions() {
            let definition = validator::Definition::try_from(definition.clone())
                .context("Supplied proto is not a valid definition")?;
            // First, check the signature:
            let definition_bytes = definition.validator.encode_to_vec();
            definition
                .validator
                .identity_key
                .0
                .verify(&definition_bytes, &definition.auth_sig)
                .context("Validator definition signature failed to verify")?;

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
                    "Validator defined {} bps of funding streams, greater than 10000bps = 100%",
                    total_funding_bps
                ));
            }
        }

        Ok(())
    }

    #[instrument(name = "staking", skip(self, tx))]
    async fn check_tx_stateful(&self, tx: &Transaction) -> Result<()> {
        // Tally the delegations and undelegations
        let mut delegation_changes = BTreeMap::new();
        for d in tx.delegations() {
            let next_rate_data = self
                .state
                .next_validator_rate(&d.validator_identity)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("Unknown validator identity {}", d.validator_identity)
                })?
                .clone();

            // Check whether the epoch is correct first, to give a more helpful
            // error message if it's wrong.
            if d.epoch_index != next_rate_data.epoch_index {
                return Err(anyhow::anyhow!(
                    "Delegation was prepared for epoch {} but the next epoch is {}",
                    d.epoch_index,
                    next_rate_data.epoch_index
                ));
            }

            // Check whether the delegation is for a slashed validator
            let validator_state = self
                .state
                .validator_state(&d.validator_identity)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing state for validator"))?;
            if validator_state == validator::State::Slashed {
                return Err(anyhow::anyhow!(
                    "Delegation to slashed validator {}",
                    d.validator_identity
                ));
            };

            // For delegations, we enforce correct computation (with rounding)
            // of the *delegation amount based on the unbonded amount*, because
            // users (should be) starting with the amount of unbonded stake they
            // wish to delegate, and computing the amount of delegation tokens
            // they receive.
            //
            // The direction of the computation matters because the computation
            // involves rounding, so while both
            //
            // (unbonded amount, rates) -> delegation amount
            // (delegation amount, rates) -> unbonded amount
            //
            // should give approximately the same results, they may not give
            // exactly the same results.
            let expected_delegation_amount = next_rate_data.delegation_amount(d.unbonded_amount);

            if expected_delegation_amount == d.delegation_amount {
                // The delegation amount is added to the delegation token supply.
                *delegation_changes
                    .entry(d.validator_identity.clone())
                    .or_insert(0) += i64::try_from(d.delegation_amount).unwrap();
            } else {
                return Err(anyhow::anyhow!(
                    "Given {} unbonded stake, expected {} delegation tokens but description produces {}",
                    d.unbonded_amount,
                    expected_delegation_amount,
                    d.delegation_amount
                ));
            }
        }
        for u in tx.undelegations() {
            let rate_data = self
                .state
                .next_validator_rate(&u.validator_identity)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!("Unknown validator identity {}", u.validator_identity)
                })?;

            // Check whether the epoch is correct first, to give a more helpful
            // error message if it's wrong.
            if u.epoch_index != rate_data.epoch_index {
                return Err(anyhow::anyhow!(
                    "Undelegation was prepared for next epoch {} but the next epoch is {}",
                    u.epoch_index,
                    rate_data.epoch_index
                ));
            }

            // For undelegations, we enforce correct computation (with rounding)
            // of the *unbonded amount based on the delegation amount*, because
            // users (should be) starting with the amount of delegation tokens they
            // wish to undelegate, and computing the amount of unbonded stake
            // they receive.
            //
            // The direction of the computation matters because the computation
            // involves rounding, so while both
            //
            // (unbonded amount, rates) -> delegation amount
            // (delegation amount, rates) -> unbonded amount
            //
            // should give approximately the same results, they may not give
            // exactly the same results.
            let expected_unbonded_amount = rate_data.unbonded_amount(u.delegation_amount);

            if expected_unbonded_amount == u.unbonded_amount {
                // TODO: in order to have exact tracking of the token supply, we probably
                // need to change this to record the changes to the unbonded stake and
                // the delegation token separately

                // The undelegation amount is subtracted from the delegation token supply.
                *delegation_changes
                    .entry(u.validator_identity.clone())
                    .or_insert(0) -= i64::try_from(u.delegation_amount).unwrap();
            } else {
                return Err(anyhow::anyhow!(
                    "Given {} delegation tokens, expected {} unbonded stake but description produces {}",
                    u.delegation_amount,
                    expected_unbonded_amount,
                    u.unbonded_amount,
                ));
            }
        }

        // Check that the sequence numbers of updated validators are correct.
        for v in tx.validator_definitions() {
            let v = validator::Definition::try_from(v.clone())
                .context("Supplied proto is not a valid definition")?;
            let existing_v = self.state.validator(&v.validator.identity_key).await?;

            if let Some(existing_v) = existing_v {
                // This is an existing validator definition. Ensure that the highest
                // existing sequence number is less than the new sequence number.
                let current_seq = existing_v.sequence_number;
                if v.validator.sequence_number <= current_seq {
                    return Err(anyhow::anyhow!(
                        "Expected sequence numbers to be increasing. Current sequence number is {}",
                        current_seq
                    ));
                }
            } else {
                // This is a new validator definition.
                continue;
            }

            // the validator definition has now passed all verification checks
        }

        Ok(())
    }

    #[instrument(name = "staking", skip(self, tx))]
    async fn execute_tx(&mut self, tx: &Transaction) {
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

        // The validator definitions have been completely verified, so we can add them to the JMT
        let definitions = tx.validator_definitions().map(|v| v.to_owned());
        let cur_epoch = self.state.get_current_epoch().await.unwrap();

        for v in definitions {
            let v = validator::Definition::try_from(v.clone())
                .expect("we already checked that this was a valid proto");
            if self
                .state
                .validator(&v.validator.identity_key)
                .await
                .unwrap()
                .is_some()
            {
                // This is an existing validator definition.
                // This means that only the Validator struct itself needs updating, not any rates/power/state.
                self.state.update_validator(v.validator).await.unwrap();
            } else {
                // This is a new validator definition.
                // Set the default rates and state.
                let validator_key = v.validator.identity_key.clone();

                // Delegations require knowing the rates for the
                // next epoch, so pre-populate with 0 reward => exchange rate 1 for
                // the current and next epochs.
                let cur_rate_data = RateData {
                    identity_key: validator_key.clone(),
                    epoch_index: cur_epoch.index,
                    validator_reward_rate: 0,
                    validator_exchange_rate: 1_0000_0000, // 1 represented as 1e8
                };
                let next_rate_data = RateData {
                    identity_key: validator_key.clone(),
                    epoch_index: cur_epoch.index + 1,
                    validator_reward_rate: 0,
                    validator_exchange_rate: 1_0000_0000, // 1 represented as 1e8
                };

                self.add_validator(v.validator.clone(), cur_rate_data, next_rate_data)
                    .await
                    .unwrap();
            }
        }
    }

    #[instrument(name = "staking", skip(self, end_block))]
    async fn end_block(&mut self, end_block: &abci::request::EndBlock) {
        // Write the delegation changes for this block.
        self.state
            .set_delegation_changes(
                end_block.height.try_into().unwrap(),
                std::mem::take(&mut self.delegation_changes),
            )
            .await;

        // If this is an epoch boundary, updated rates need to be calculated and set.
        let cur_epoch = self.state.get_current_epoch().await.unwrap();
        let cur_height = self.state.get_block_height().await.unwrap();

        if cur_epoch.is_epoch_end(cur_height) {
            self.end_epoch(cur_epoch).await.unwrap();
        }
    }
}

/// Extension trait providing read/write access to staking data.
///
/// TODO: should this be split into Read and Write traits?
#[async_trait]
pub trait View: StateExt {
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
    async fn set_validator_power(
        &self,
        identity_key: &IdentityKey,
        voting_power: u64,
    ) -> Result<()> {
        tracing::debug!("setting validator power");
        if voting_power as i64 > MAX_VOTING_POWER || (voting_power as i64) < 0 {
            return Err(anyhow::anyhow!("invalid voting power"));
        }

        self.put_proto(
            format!("staking/validators/{}/power", identity_key).into(),
            voting_power,
        )
        .await;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn validator_power(&self, identity_key: &IdentityKey) -> Result<Option<u64>> {
        self.get_proto(format!("staking/validators/{}/power", identity_key).into())
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

    #[instrument(skip(self))]
    async fn set_validator_state(
        &self,
        identity_key: &IdentityKey,
        state: validator::State,
    ) -> Result<()> {
        // Enforce state machine semantics here and update voting powers
        // for tendermint appropriately

        let cur_state = self.validator_state(&identity_key).await?.ok_or_else(|| {
            anyhow::anyhow!("validator to have state change did not have state in JMT")
        })?;

        // Ensure that the state transitions are valid.
        // TODO: there are other semantics we could possibly enforce here
        // that are currently being enforced by upstream callers, for example
        // that to become Active a validator must appear in the top N validators
        // by voting power. Having all checks enforced through this single method
        // makes bugs relating to improperly setting state less likely, though
        // moving them here might mean duplicating checks in some cases (how do
        // you know to call this method unless you've checked the criteria?).
        // Is the View method even the right place to enforce these checks?
        match state {
            validator::State::Slashed => match cur_state {
                validator::State::Active => {}
                validator::State::Inactive => {}
                _ => {
                    return Err(anyhow::anyhow!(
                        "only validators in the active or inactive state may be slashed"
                    ))
                }
            },
            validator::State::Active => match cur_state {
                validator::State::Inactive => {}
                _ => {
                    return Err(anyhow::anyhow!(
                        "only validators in the inactive state may become Active"
                    ))
                }
            },
            validator::State::Inactive => match cur_state {
                validator::State::Active => {}
                _ => {
                    return Err(anyhow::anyhow!(
                        "only validators in the Active state may become Inactive"
                    ))
                }
            },
            _ => return Err(anyhow::anyhow!("cannot set validator state to {:?}", state)),
        };

        tracing::debug!("setting validator state");
        self.put_domain(
            format!("staking/validators/{}/state", identity_key).into(),
            state,
        )
        .await;

        Ok(())
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

    async fn slash_validator(&self, validator: Validator, slashing_penalty: u64) -> Result<()> {
        tracing::info!(?validator, ?slashing_penalty, "slashing validator");

        // Mark the state as "slashed" in the JMT, and apply the slashing penalty.
        self.set_validator_state(&validator.identity_key, validator::State::Slashed)
            .await?;

        let mut cur_rate = self
            .current_validator_rate(&validator.identity_key)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("validator to be slashed did not have current rate in JMT")
            })?;

        cur_rate = cur_rate.slash(slashing_penalty);

        // TODO: would it be better to call `current_base_rate.next`? the same logic exists
        // within there, but it requires passing in the current base rates & funding streams,
        // which aren't actually used because the rate is held constant. So, doing it this way
        // avoids a couple unnecessary JMT reads that the `current_base_rate.next` API would require.
        //
        // At any rate, the next rate is held constant for slashed validators.
        let mut next_rate = cur_rate.clone();
        next_rate.epoch_index += 1;

        self.set_validator_rates(&validator.identity_key, cur_rate, next_rate)
            .await;

        Ok(())
    }

    // Used for updating an existing validator's definition.
    async fn update_validator(&self, validator: Validator) -> Result<()> {
        tracing::debug!(?validator);
        let id = validator.identity_key.clone();
        // If the validator isn't already in the JMT, we can't update it.
        self.validator(&id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("updated validator not found in JMT"))?;

        self.put_domain(format!("staking/validators/{}", id).into(), validator)
            .await;

        Ok(())
    }

    // Used for adding a new validator to the JMT. May be either
    // Active (a genesis validator) on Inactive (a validator added
    // post-genesis).
    async fn add_validator(
        &self,
        validator: Validator,
        current_rates: RateData,
        next_rates: RateData,
        state: validator::State,
        power: u64,
    ) -> Result<()> {
        tracing::debug!(?validator);
        let id = validator.identity_key.clone();

        self.put_domain(format!("staking/validators/{}", id).into(), validator)
            .await;
        self.register_denom(&DelegationToken::from(&id).denom())
            .await?;

        self.set_validator_rates(&id, current_rates, next_rates)
            .await;

        // We can't call `set_validator_state` here because it requires an existing validator state,
        // so we manually initialize the state for new validators.
        self.put_domain(format!("staking/validators/{}/state", &id).into(), state)
            .await;
        self.set_validator_power(&id, power).await?;

        let mut validator_list = self.validator_list().await?;
        validator_list.push(id);
        tracing::debug!(?validator_list);
        self.set_validator_list(validator_list).await;

        Ok(())
    }

    async fn validator_info(&self, identity_key: &IdentityKey) -> Result<Option<validator::Info>> {
        let validator = self.validator(identity_key).await?;
        let status = self.validator_status(identity_key).await?;
        let rate_data = self.next_validator_rate(identity_key).await?;
        match (validator, status, rate_data) {
            (Some(validator), Some(status), Some(rate_data)) => Ok(Some(validator::Info {
                validator,
                status,
                rate_data,
            })),
            _ => Ok(None),
        }
    }

    async fn validator_state(
        &self,
        identity_key: &IdentityKey,
    ) -> Result<Option<validator::State>> {
        self.get_domain(format!("staking/validators/{}/state", identity_key).into())
            .await
    }

    async fn validator_unbonding_status(
        &self,
        identity_key: &IdentityKey,
    ) -> Result<Option<validator::UnbondingStatus>> {
        self.get_domain(format!("staking/validators/{}/unbonding_status", identity_key).into())
            .await
    }

    /// Convenience method to assemble a [`ValidatorStatus`].
    async fn validator_status(
        &self,
        identity_key: &IdentityKey,
    ) -> Result<Option<validator::Status>> {
        // TODO: replace w/ using the higher level `ValidatorStatus` struct to store all this data together
        let unbonding_status = self.validator_unbonding_status(identity_key).await?;
        let state = self.validator_state(identity_key).await?;
        let power = self.validator_power(identity_key).await?;
        let identity_key = identity_key.clone();
        match (state, power) {
            (Some(state), Some(voting_power)) => Ok(Some(validator::Status {
                identity_key,
                state,
                voting_power,
                unbonding_status,
            })),
            _ => Ok(None),
        }
    }

    async fn validator_list(&self) -> Result<Vec<IdentityKey>> {
        Ok(self
            .get_domain("staking/validators/list".into())
            .await?
            .map(|list: validator::List| list.0)
            .unwrap_or_default())
    }

    async fn set_validator_list(&self, validators: Vec<IdentityKey>) {
        self.put_domain(
            "staking/validators/list".into(),
            validator::List(validators),
        )
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

    async fn validator_uptime(&self, identity_key: &IdentityKey) -> Result<Option<Uptime>> {
        self.get_domain(format!("staking/validator_uptime/{}", identity_key).into())
            .await
    }

    async fn set_validator_uptime(&self, identity_key: &IdentityKey, uptime: Uptime) {
        self.put_domain(
            format!("staking/validator_uptime/{}", identity_key).into(),
            uptime,
        )
        .await
    }

    async fn clear_validator_unbonding_status(&self, identity_key: &IdentityKey) {
        self.put_domain(
            format!("staking/validators/{}/unbonding_status", identity_key).into(),
            None,
        )
        .await
    }

    async fn signed_blocks_window_len(&self) -> Result<u64> {
        Ok(self.get_chain_params().await?.signed_blocks_window_len)
    }

    async fn missed_blocks_maximum(&self) -> Result<u64> {
        Ok(self.get_chain_params().await?.missed_blocks_maximum)
    }
}

impl<T: StateExt + Send + Sync> View for T {}
