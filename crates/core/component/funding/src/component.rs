mod state_key;
pub mod view;
use penumbra_asset::{Value, STAKING_TOKEN_ASSET_ID};
use penumbra_stake::component::validator_handler::ValidatorDataRead;
pub use view::{StateReadExt, StateWriteExt};

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::Component;
use tendermint::v0_37::abci;
use tracing::instrument;

use crate::genesis;

pub struct Funding {}

#[async_trait]
impl Component for Funding {
    type AppState = genesis::Content;

    #[instrument(name = "funding", skip(state, app_state))]
    async fn init_chain<S: StateWrite>(mut state: S, app_state: Option<&Self::AppState>) {
        match app_state {
            None => { /* Checkpoint -- no-op */ }
            Some(genesis) => {
                state.put_funding_params(genesis.funding_params.clone());
            }
        };
    }

    #[instrument(name = "funding", skip(_state, _begin_block))]
    async fn begin_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _begin_block: &abci::request::BeginBlock,
    ) {
    }

    #[instrument(name = "funding", skip(_state, _end_block))]
    async fn end_block<S: StateWrite + 'static>(
        _state: &mut Arc<S>,
        _end_block: &abci::request::EndBlock,
    ) {
    }

    #[instrument(name = "funding", skip(state))]
    async fn end_epoch<S: StateWrite + 'static>(state: &mut Arc<S>) -> Result<()> {
        // TODO(erwan): scoping these strictly will make it easy to refactor
        // this code when we introduce additional funding processing logic
        // e.g. for proposer tips.
        use penumbra_community_pool::StateWriteExt as _;
        use penumbra_distributions::component::StateReadExt as _;
        use penumbra_sct::CommitmentSource;
        use penumbra_shielded_pool::component::NoteManager;
        use penumbra_stake::funding_stream::Recipient;
        use penumbra_stake::StateReadExt as _;

        let state = Arc::get_mut(state).expect("state should be unique");

        // Here, we want to process the funding rewards for the epoch that just ended. To do this,
        // we pull the funding queue that the staking component has prepared for us, as well as the
        // base rate data for the epoch that just ended.
        let funding_queue = state.get_funding_queue().unwrap_or_default();
        let funding_queue_len = funding_queue.len();

        let Some(base_rate) = state.get_previous_base_rate() else {
            tracing::error!("the ending epoch's base rate has not beed found in object storage, computing rewards is not possible");
            return Ok(());
        };

        // As we process funding rewards, we want to make sure that we are always below the issuance
        // budget for the epoch.
        let staking_issuance_budget = state
            .get_staking_token_issuance_for_epoch()
            .expect("staking token issuance MUST be set");
        let mut total_staking_rewards_for_epoch = 0u128;

        for (index, (validator_identity, funding_streams, delegation_token_supply)) in
            funding_queue.into_iter().enumerate()
        {
            let Some(validator_rate) = state.get_prev_validator_rate(&validator_identity).await
            else {
                tracing::error!(
                    %validator_identity,
                    index,
                    funding_queue_len,
                    ending_epoch_base_rate = ?base_rate,
                    "the ending epoch's rate data for the validator has not been found in storage, computing rewards is not possible"
                );
                continue;
            };

            for stream in funding_streams {
                // We compute the reward amount for this specific funding stream, it is based
                // on the ending epoch's rate data.
                let reward_amount_for_stream =
                    stream.reward_amount(&base_rate, &validator_rate, delegation_token_supply);

                total_staking_rewards_for_epoch = total_staking_rewards_for_epoch
                    .saturating_add(reward_amount_for_stream.value());

                if total_staking_rewards_for_epoch > staking_issuance_budget.value() {
                    tracing::warn!(
                        %total_staking_rewards_for_epoch,
                        %staking_issuance_budget,
                        %reward_amount_for_stream,
                        "the sum of staking rewards for the epoch has exceeded the issuance budget"
                    );

                    tracing::warn!(%validator_identity,
                        index,
                        funding_queue_len,
                        ending_epoch_base_rate = ?base_rate,
                        funding_stream = ?stream,
                        delegation_token_supply = ?delegation_token_supply, "debugging information for the funding stream that caused the error");
                }

                match stream.recipient() {
                    // If the recipient is an address, mint a note to that address
                    Recipient::Address(address) => {
                        state
                            .mint_note(
                                Value {
                                    amount: reward_amount_for_stream.into(),
                                    asset_id: *STAKING_TOKEN_ASSET_ID,
                                },
                                &address,
                                CommitmentSource::FundingStreamReward {
                                    epoch_index: base_rate.epoch_index,
                                },
                            )
                            .await?;
                    }
                    // If the recipient is the Community Pool, deposit the funds into the Community Pool
                    Recipient::CommunityPool => {
                        state
                            .community_pool_deposit(Value {
                                amount: reward_amount_for_stream.into(),
                                asset_id: *STAKING_TOKEN_ASSET_ID,
                            })
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }
}
