use anyhow::anyhow;
use cnidarium::{StateRead, StateWrite};
use futures::stream::StreamExt as _;
use penumbra_sdk_asset::{asset, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_community_pool::component::StateWriteExt as _;
use penumbra_sdk_dex::{component::LqtRead as _, lp::position};
use penumbra_sdk_distributions::component::StateReadExt as _;
use penumbra_sdk_keys::address::ADDRESS_LEN_BYTES;
use penumbra_sdk_keys::Address;
use penumbra_sdk_sct::component::clock::EpochRead as _;
use penumbra_sdk_txhash::TransactionId;

use super::bank::Bank as _;
use super::votes::StateReadExt as _;
use crate::component::StateReadExt as _;

mod gauge;
use gauge::{Gauge, Share};

async fn position_shares(
    state: impl StateRead,
    epoch: u64,
    asset: asset::Id,
    top_n: usize,
) -> anyhow::Result<Vec<(Share, position::Id)>> {
    let mut stream = state.positions_by_volume_stream(epoch, asset)?.take(top_n);
    let mut total = 0u128;
    // This will end up containing the volume of each lp.
    let mut tallies = Vec::new();
    while let Some(x) = stream.next().await {
        let (_, lp, volume) = x?;
        let volume = volume.value();
        total += volume;
        tallies.push((Share::from(volume), lp));
    }
    // Then divide each volume by the total volume, to get a relevant share.
    for x in &mut tallies {
        // If we divide by 0, just treat all positions as having 0 share.
        let share = (x.0 / Share::from(total)).unwrap_or_default();
        *x = (share, x.1);
    }
    Ok(tallies)
}

pub async fn distribute_rewards(mut state: impl StateWrite + Sized) -> anyhow::Result<()> {
    let current_epoch = state.get_current_epoch().await?;
    let params = state.get_funding_params().await?;

    // Because of borrow checking shenanigans, we first collect all the votes,
    // in the form of a big blob of bytes for voter addresses, and individual tuples
    // for the asset, power tallies.

    // 80 KB of memory should be enough for anybody.
    let mut all_voter_bytes = Vec::<u8>::with_capacity(ADDRESS_LEN_BYTES * 1000);
    let mut tallies = Vec::new();
    let mut vote_receipts = state.vote_receipts(current_epoch.index);
    while let Some(x) = vote_receipts.next().await {
        let (asset, power, voter_bytes) = x?;
        all_voter_bytes.extend_from_slice(&voter_bytes);
        tallies.push((asset, power));
    }
    // Now actually tally things up.
    let mut gauge = Gauge::empty();
    for ((asset, power), voter) in tallies
        .into_iter()
        .zip(all_voter_bytes.chunks_exact(ADDRESS_LEN_BYTES))
    {
        gauge.tally(asset, power, voter);
    }
    let finalized = gauge.finalize(
        params.liquidity_tournament.gauge_threshold,
        usize::try_from(params.liquidity_tournament.max_delegators)?,
    );

    // Get the initial budget, and immediately withdraw it from the community pool.
    let initial_budget = state
        .get_lqt_reward_issuance_for_epoch(current_epoch.index)
        .await
        .unwrap_or_default();
    state
        .community_pool_withdraw(Value {
            asset_id: *STAKING_TOKEN_ASSET_ID,
            amount: initial_budget,
        })
        .await?;
    // Now, we keep a running mutable current budget, because we distribute fractions of
    // the initial budget, which should not be modified.
    let mut current_budget = initial_budget;

    // First, distribute rewards to voters.
    for (voter_share, voter) in finalized.voter_shares() {
        let voter_addr = Address::try_from(voter)?;
        let reward = (Share::from(params.liquidity_tournament.delegator_share) * voter_share)?
            .apply_to_amount(&initial_budget)?;
        // TODO: use a real transaction id or ids.
        state
            .reward_to_voter(reward, &voter_addr, TransactionId::default())
            .await?;
        current_budget = current_budget
            .checked_sub(&reward)
            .ok_or(anyhow!("LQT rewards exceeded budget"))?;
    }

    // Next, distribute rewards to positions.
    let lp_reward_share = Share::from(params.liquidity_tournament.delegator_share.complement());
    for (asset_share, asset) in finalized.asset_shares() {
        for (lp_share, lp) in position_shares(
            &state,
            current_epoch.index,
            asset,
            params.liquidity_tournament.max_positions.try_into()?,
        )
        .await?
        {
            // What fraction goes to lps, then of that, to this asset, then of that, to this lp.
            let reward =
                ((lp_reward_share * asset_share)? * lp_share)?.apply_to_amount(&initial_budget)?;
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

    Ok(())
}
