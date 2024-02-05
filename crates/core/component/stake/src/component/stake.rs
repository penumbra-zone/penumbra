use anyhow::Context;
use cnidarium::StateWrite;
use penumbra_num::Amount;
use penumbra_proto::StateWriteProto as _;
use penumbra_sct::component::clock::EpochRead;
use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use cnidarium_component::Component;

use tendermint::abci::{self};
use tracing::instrument;

use crate::{
    rate::BaseRateData, state_key, validator::Validator, CurrentConsensusKeys, StateReadExt as _,
};

use super::{validator_manager::ValidatorManager as _, StakingImpl as _, StateWriteExt as _};

pub struct Staking {}

impl Staking {}

#[async_trait]
impl Component for Staking {
    type AppState = (
        crate::genesis::Content,
        penumbra_shielded_pool::genesis::Content,
    );

    #[instrument(name = "staking", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            None => { /* perform upgrade specific check */ }
            Some((staking_genesis, sp_genesis)) => {
                state.put_stake_params(staking_genesis.stake_params.clone());

                let starting_height = state
                    .get_block_height()
                    .await
                    .expect("should be able to get initial block height");
                let starting_epoch = state
                    .get_epoch_by_height(starting_height)
                    .await
                    .expect("should be able to get initial epoch");
                let epoch_index = starting_epoch.index;

                let genesis_base_rate = BaseRateData {
                    epoch_index,
                    base_reward_rate: 0u128.into(),
                    base_exchange_rate: 1_0000_0000u128.into(),
                };
                state.set_base_rate(genesis_base_rate.clone());

                let mut genesis_allocations = BTreeMap::<_, Amount>::new();
                for allocation in &sp_genesis.allocations {
                    let value = allocation.value();
                    *genesis_allocations.entry(value.asset_id).or_default() += value.amount;
                }

                for validator in &staking_genesis.validators {
                    // Parse the proto into a domain type.
                    let validator = Validator::try_from(validator.clone())
                        .expect("should be able to parse genesis validator");

                    state
                        .add_genesis_validator(&genesis_allocations, &genesis_base_rate, validator)
                        .await
                        .expect("should be able to add genesis validator to state");
                }

                // First, "prime" the state with an empty set, so the build_ function can read it.
                state.put(
                    state_key::current_consensus_keys().to_owned(),
                    CurrentConsensusKeys::default(),
                );

                // Finally, record that there were no delegations in this block, so the data
                // isn't missing when we process the first epoch transition.
                state
                    .set_delegation_changes(
                        starting_height
                            .try_into()
                            .expect("should be able to convert u64 into block height"),
                        Default::default(),
                    )
                    .await;
            }
        }
        // Build the initial validator set update.
        state
            .build_tendermint_validator_updates()
            .await
            .expect("should be able to build initial tendermint validator updates");
    }

    #[instrument(name = "staking", skip(state, begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        begin_block: &abci::request::BeginBlock,
    ) {
        let state = Arc::get_mut(state).expect("state should be unique");
        // For each validator identified as byzantine by tendermint, update its
        // state to be slashed. If the validator is not tracked in the JMT, this
        // will be a no-op. See #2919 for more details.
        for evidence in begin_block.byzantine_validators.iter() {
            let _ = state.process_evidence(evidence).await.map_err(|e| {
                tracing::warn!(?e, "failed to process byzantine misbehavior evidence")
            });
        }

        state
            .track_uptime(&begin_block.last_commit_info)
            .await
            .expect("should be able to track uptime");
    }

    #[instrument(name = "staking", skip(state, end_block))]
    async fn end_block<S: StateWrite + 'static>(
        state: &mut Arc<S>,
        end_block: &abci::request::EndBlock,
    ) {
        let state = Arc::get_mut(state).expect("state should be unique");
        // Write the delegation changes for this block.
        state
            .set_delegation_changes(
                end_block
                    .height
                    .try_into()
                    .expect("should be able to convert i64 into block height"),
                state.get_delegation_changes().clone(),
            )
            .await;
    }

    #[instrument(name = "staking", skip(state))]
    async fn end_epoch<S: StateWrite + 'static>(state: &mut Arc<S>) -> anyhow::Result<()> {
        let state = Arc::get_mut(state).context("state should be unique")?;
        let epoch_ending = state
            .get_current_epoch()
            .await
            .context("should be able to get current epoch during end_epoch")?;
        state.end_epoch(epoch_ending).await?;
        // Since we only update the validator set at epoch boundaries,
        // we only need to build the validator set updates here in end_epoch.
        state
            .build_tendermint_validator_updates()
            .await
            .context("should be able to build tendermint validator updates")?;
        Ok(())
    }
}
