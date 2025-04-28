use anyhow::anyhow;
use cnidarium::{StateRead, StateWrite};
use futures::stream::StreamExt as _;
use futures::{Stream, TryStreamExt};
use penumbra_sdk_asset::{asset, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_community_pool::component::StateWriteExt as _;
use penumbra_sdk_dex::{component::LqtRead as _, lp::position};
use penumbra_sdk_distributions::component::{StateReadExt as _, StateWriteExt as _};
use penumbra_sdk_keys::Address;
use penumbra_sdk_num::fixpoint::U128x128;
use penumbra_sdk_num::Amount;
use penumbra_sdk_proto::DomainType;
use penumbra_sdk_proto::StateWriteProto;
use penumbra_sdk_sct::component::clock::EpochRead as _;
use penumbra_sdk_stake::IdentityKey;
use penumbra_sdk_txhash::TransactionId;

use super::bank::Bank as _;
use super::votes::StateReadExt as _;
use crate::component::StateReadExt as _;
use crate::event;
use crate::params::LiquidityTournamentParameters;

fn create_share(portion: impl Into<U128x128>, total: impl Into<U128x128>) -> U128x128 {
    U128x128::ratio(portion.into(), total.into().max(U128x128::from(1u64)))
        .expect("max(x, 1) cannot be 0")
}

async fn relevant_votes_for_asset(
    state: impl StateRead,
    params: &LiquidityTournamentParameters,
    epoch: u64,
    asset: asset::Id,
) -> anyhow::Result<Amount> {
    let mut out = Amount::default();
    let mut stream = state.ranked_voters(epoch, asset).take(
        usize::try_from(params.max_delegators).expect("max delegators should fit in a usize"),
    );
    while let Some(((power, _voter, _), _tx)) = stream.try_next().await? {
        out += Amount::from(power);
    }
    Ok(out)
}

/// Return each asset along with the share of the relevant vote it received.
//
/// Each asset will only be featured once.
///
/// This will only count votes from the top N voters, as per the params,
/// and only include assets which clear the threshold.
#[tracing::instrument(skip(state))]
async fn asset_totals(
    state: &impl StateRead,
    params: &LiquidityTournamentParameters,
    epoch: u64,
) -> anyhow::Result<Vec<(asset::Id, Amount)>> {
    let total_votes = state.total_votes(epoch).await;
    tracing::debug!(?total_votes);
    // 100 should be ample, but also not a huge amount to allocate.
    let mut out = Vec::with_capacity(100);
    let mut stream = state.ranked_assets(epoch);
    while let Some((_, asset)) = stream.try_next().await? {
        let votes = relevant_votes_for_asset(state, params, epoch, asset).await?;
        tracing::debug!(?asset, ?votes, "found votes");
        // The assets are ranked by descending power, so we can stop here.
        if create_share(votes, total_votes) < U128x128::from(params.gauge_threshold) {
            break;
        }
        out.push((asset, votes));
    }
    Ok(out)
}

fn voter_shares_of_asset(
    state: &impl StateRead,
    params: &LiquidityTournamentParameters,
    epoch: u64,
    asset: asset::Id,
    total: Amount,
) -> impl Stream<Item = anyhow::Result<(Address, U128x128, IdentityKey, TransactionId)>> + Send + 'static
{
    state
        .ranked_voters(epoch, asset)
        .take(usize::try_from(params.max_delegators).expect("max delegators should fit in a usize"))
        .map_ok(move |((votes, voter, validator), tx)| {
            (voter, create_share(votes, total), validator, tx)
        })
}

async fn relevant_positions_total_volume(
    state: impl StateRead,
    params: &LiquidityTournamentParameters,
    epoch: u64,
    asset: asset::Id,
) -> anyhow::Result<Amount> {
    let mut stream = state
        .positions_by_volume_stream(epoch, asset)?
        .take(usize::try_from(params.max_positions).expect("max positions should fit in a usize"));
    let mut total = Amount::default();
    while let Some((_, _, volume)) = stream.try_next().await? {
        total += volume;
    }
    Ok(total)
}

fn position_shares(
    state: impl StateRead,
    params: &LiquidityTournamentParameters,
    epoch: u64,
    asset: asset::Id,
    total_volume: Amount,
) -> impl Stream<Item = anyhow::Result<(position::Id, Amount, U128x128)>> + Send + 'static {
    state
        .positions_by_volume_stream(epoch, asset)
        .expect("should be able to create positions by volume stream")
        .take(usize::try_from(params.max_positions).expect("max positions should fit in a usize"))
        .map_ok(move |(_, lp, volume)| (lp, volume, create_share(volume, total_volume)))
}

#[tracing::instrument(skip(state))]
pub async fn distribute_rewards(mut state: impl StateWrite) -> anyhow::Result<()> {
    let current_epoch = state.get_current_epoch().await?.index;
    let params = state.get_funding_params().await?;

    // Get the initial budget, and immediately withdraw it from the community pool.
    let initial_budget = state
        .get_lqt_reward_issuance_for_epoch(current_epoch)
        .await
        .unwrap_or_default();
    if initial_budget <= 0u64.into() {
        tracing::info!("no budget for LQT, so not distributing any rewards");
        return Ok(());
    }
    state
        .community_pool_withdraw(Value {
            asset_id: *STAKING_TOKEN_ASSET_ID,
            amount: initial_budget,
        })
        .await?;
    tracing::debug!(?initial_budget);
    // Now, we keep a running mutable current budget, because we distribute fractions of
    // the initial budget, which should not be modified.
    let mut current_budget = initial_budget;

    // First, figure out the total votes for each asset, after culling unpopular assets,
    // and insufficiently highly ranked voters.
    let asset_totals = asset_totals(&state, &params.liquidity_tournament, current_epoch).await?;
    let total_votes: Amount = asset_totals.iter().map(|(_, v)| *v).sum();
    // Now, iterate over each asset, and it's share of the total.
    for (incentivized_asset, asset_votes) in asset_totals {
        let asset_share = create_share(asset_votes, total_votes);
        // Next, distribute rewards to voters.
        let mut voter_stream = voter_shares_of_asset(
            &state,
            &params.liquidity_tournament,
            current_epoch,
            incentivized_asset,
            asset_votes,
        );
        while let Some((voter, voter_share, identity_key, tx)) = voter_stream.try_next().await? {
            // We compute the reward denominated in staking tokens.
            let unbonded_reward_amount =
                ((U128x128::from(params.liquidity_tournament.delegator_share) * asset_share)?
                    * voter_share)?
                    .apply_to_amount(&initial_budget)?;

            // We ask the bank to mint rewards.
            state
                .reward_to_voter(
                    unbonded_reward_amount,
                    identity_key,
                    &voter,
                    tx,
                    incentivized_asset,
                )
                .await?;
            current_budget = current_budget
                .checked_sub(&unbonded_reward_amount)
                .ok_or(anyhow!("LQT rewards exceeded budget"))?;
        }
        // Then, distribute rewards to LPs.
        let total_volume = relevant_positions_total_volume(
            &state,
            &params.liquidity_tournament,
            current_epoch,
            incentivized_asset,
        )
        .await?;
        let mut lp_stream = position_shares(
            &state,
            &params.liquidity_tournament,
            current_epoch,
            incentivized_asset,
            total_volume,
        );
        while let Some((lp, lp_volume, lp_share)) = lp_stream.try_next().await? {
            // What fraction goes to lps, then of that, to this asset, then of that, to this lp.
            let lp_reward_share =
                U128x128::from(params.liquidity_tournament.delegator_share.complement());
            let reward =
                ((lp_reward_share * asset_share)? * lp_share)?.apply_to_amount(&initial_budget)?;

            // TODO(erwan): we should probably internalize this.
            let event = event::EventLqtPositionReward {
                epoch_index: current_epoch,
                reward_amount: reward,
                position_id: lp,
                incentivized_asset_id: incentivized_asset,
                tournament_volume: total_volume,
                position_volume: lp_volume,
            };
            state.record_proto(event.to_proto());

            state.reward_to_position(reward, lp).await?;
            current_budget = current_budget
                .checked_sub(&reward)
                .ok_or(anyhow!("LQT rewards exceeded budget"))?;
        }
    }

    // Finally, move whatever budget remains back into the community pool.
    // We expect this to be some dust amount, because of rounding.
    state
        .community_pool_deposit(Value {
            asset_id: *STAKING_TOKEN_ASSET_ID,
            amount: current_budget,
        })
        .await;
    // Set up the next epoch to have whatever rewards are left.
    state.set_lqt_reward_issuance_for_epoch(current_epoch + 1, current_budget);

    Ok(())
}
