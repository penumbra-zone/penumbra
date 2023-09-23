use penumbra_dex::swap_claim::SwapClaimPlan;
use penumbra_proto::view::v1alpha1::NotesRequest;
use std::collections::BTreeMap;

use anyhow::{Context, Result};
use ark_std::UniformRand;
use decaf377::Fq;
use penumbra_asset::Value;
use penumbra_fee::Fee;
use penumbra_governance::{proposal_state, Proposal, ValidatorVote};
use penumbra_keys::{
    keys::{AccountGroupId, AddressIndex},
    Address,
};
use penumbra_num::Amount;
use penumbra_stake::rate::RateData;
use penumbra_stake::validator;
use penumbra_transaction::{memo::MemoPlaintext, plan::TransactionPlan};
use penumbra_view::{SpendableNoteRecord, ViewClient};
use rand_core::{CryptoRng, RngCore};
use tracing::instrument;

pub use penumbra_view::Planner;

pub async fn validator_definition<V, R>(
    account_group_id: AccountGroupId,
    view: &mut V,
    rng: R,
    new_validator: validator::Definition,
    fee: Fee,
    source_address: AddressIndex,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    Planner::new(rng)
        .fee(fee)
        .validator_definition(new_validator)
        .plan(view, account_group_id, source_address)
        .await
        .context("can't build validator definition plan")
}

pub async fn validator_vote<V, R>(
    account_group_id: AccountGroupId,
    view: &mut V,
    rng: R,
    vote: ValidatorVote,
    fee: Fee,
    source_address: AddressIndex,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    Planner::new(rng)
        .fee(fee)
        .validator_vote(vote)
        .plan(view, account_group_id, source_address)
        .await
        .context("can't build validator vote plan")
}

/// Generate a new transaction plan delegating stake
#[instrument(skip(
    account_group_id,
    view,
    rng,
    rate_data,
    unbonded_amount,
    fee,
    source_address
))]
pub async fn delegate<V, R>(
    account_group_id: AccountGroupId,
    view: &mut V,
    rng: R,
    rate_data: RateData,
    unbonded_amount: u128,
    fee: Fee,
    source_address: AddressIndex,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    Planner::new(rng)
        .fee(fee)
        .delegate(unbonded_amount, rate_data)
        .plan(view, account_group_id, source_address)
        .await
        .context("can't build delegate plan")
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(
    account_group_id,
    view,
    rng,
    values,
    fee,
    dest_address,
    source_address,
    tx_memo
))]
pub async fn send<V, R>(
    account_group_id: AccountGroupId,
    view: &mut V,
    rng: R,
    values: &[Value],
    fee: Fee,
    dest_address: Address,
    source_address: AddressIndex,
    tx_memo: Option<MemoPlaintext>,
) -> anyhow::Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    tracing::debug!(?values, ?fee, ?dest_address, ?source_address, ?tx_memo);
    let mut planner = Planner::new(rng);
    planner.fee(fee);
    for value in values.iter().cloned() {
        planner.output(value, dest_address);
    }
    planner
        .memo(tx_memo.unwrap_or_default())?
        .plan(view, account_group_id, source_address)
        .await
        .context("can't build send transaction")
}

#[instrument(skip(account_group_id, view, rng))]
pub async fn sweep<V, R>(
    account_group_id: AccountGroupId,
    view: &mut V,
    mut rng: R,
) -> anyhow::Result<Vec<TransactionPlan>>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    let mut plans = Vec::new();

    // First, find any un-claimed swaps and add `SwapClaim` plans for them.
    plans.extend(claim_unclaimed_swaps(view, &mut rng).await?);

    // Finally, sweep dust notes by spending them to their owner's address.
    // This will consolidate small-value notes into larger ones.
    plans.extend(sweep_notes(account_group_id, view, &mut rng).await?);

    Ok(plans)
}

#[instrument(skip(view, rng))]
pub async fn claim_unclaimed_swaps<V, R>(
    view: &mut V,
    mut rng: R,
) -> anyhow::Result<Vec<TransactionPlan>>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    let mut plans = Vec::new();
    // fetch all transactions
    // check if they contain Swap actions
    // if they do, check if the associated notes are unspent
    // if they are, decrypt the SwapCiphertext in the Swap action and construct a SwapClaim

    let chain_params = view.chain_params().await?;
    let epoch_duration = chain_params.clone().epoch_duration;

    let unclaimed_swaps = view.unclaimed_swaps().await?;

    for swap in unclaimed_swaps {
        // We found an unspent swap NFT, so we can claim it.
        let swap_plaintext = swap.swap;

        let output_data = swap.output_data;

        let mut plan = TransactionPlan {
            chain_id: chain_params.clone().chain_id,
            fee: swap_plaintext.claim_fee.clone(),
            // The transaction doesn't need a memo, because it's to ourselves.
            memo_plan: None,
            ..Default::default()
        };

        let action_plan = SwapClaimPlan {
            swap_plaintext,
            position: swap.position,
            output_data,
            epoch_duration,
            proof_blinding_r: Fq::rand(&mut rng),
            proof_blinding_s: Fq::rand(&mut rng),
        };
        plan.actions.push(action_plan.into());
        plans.push(plan);
    }

    Ok(plans)
}

#[instrument(skip(account_group_id, view, rng))]
pub async fn sweep_notes<V, R>(
    account_group_id: AccountGroupId,
    view: &mut V,
    mut rng: R,
) -> anyhow::Result<Vec<TransactionPlan>>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    const SWEEP_COUNT: usize = 8;

    let all_notes = view
        .notes(NotesRequest {
            account_group_id: Some(account_group_id.into()),
            ..Default::default()
        })
        .await?;

    let mut notes_by_addr_and_denom: BTreeMap<AddressIndex, BTreeMap<_, Vec<SpendableNoteRecord>>> =
        BTreeMap::new();

    for record in all_notes {
        notes_by_addr_and_denom
            .entry(record.address_index)
            .or_default()
            .entry(record.note.asset_id())
            .or_default()
            .push(record);
    }

    let mut plans = Vec::new();

    for (index, notes_by_denom) in notes_by_addr_and_denom {
        tracing::info!(?index, "processing address");

        for (asset_id, mut records) in notes_by_denom {
            tracing::debug!(?asset_id, "processing asset");

            // Sort notes by amount, ascending, so the biggest notes are at the end...
            records.sort_by(|a, b| a.note.value().amount.cmp(&b.note.value().amount));
            // ... so that when we use chunks_exact, we get SWEEP_COUNT sized
            // chunks, ignoring the biggest notes in the remainder.
            for group in records.chunks_exact(SWEEP_COUNT) {
                let mut planner = Planner::new(&mut rng);
                planner.memo(MemoPlaintext::default())?;

                for record in group {
                    planner.spend(record.note.clone(), record.position);
                }

                let plan = planner
                    .plan(view, account_group_id, index)
                    .await
                    .context("can't build sweep transaction")?;

                tracing::debug!(?plan);
                plans.push(plan);
            }
        }
    }

    Ok(plans)
}

#[instrument(skip(account_group_id, view, rng))]
pub async fn proposal_submit<V, R>(
    account_group_id: AccountGroupId,
    view: &mut V,
    rng: R,
    proposal: Proposal,
    fee: Fee,
    source_address: AddressIndex,
) -> anyhow::Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    Planner::new(rng)
        .fee(fee)
        .proposal_submit(proposal, view.chain_params().await?.proposal_deposit_amount)
        .plan(view, account_group_id, source_address)
        .await
        .context("can't build proposal submit transaction")
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(account_group_id, view, rng))]
pub async fn proposal_withdraw<V, R>(
    account_group_id: AccountGroupId,
    view: &mut V,
    rng: R,
    proposal_id: u64,
    reason: String,
    fee: Fee,
    source_address: AddressIndex,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    Planner::new(rng)
        .fee(fee)
        .proposal_withdraw(proposal_id, reason)
        .plan(view, account_group_id, source_address)
        .await
        .context("can't build proposal withdraw transaction")
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(account_group_id, view, rng))]
pub async fn proposal_deposit_claim<V, R>(
    account_group_id: AccountGroupId,
    view: &mut V,
    rng: R,
    proposal_id: u64,
    deposit_amount: Amount,
    outcome: proposal_state::Outcome<()>,
    fee: Fee,
    source_address: AddressIndex,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    Planner::new(rng)
        .fee(fee)
        .proposal_deposit_claim(proposal_id, deposit_amount, outcome)
        .plan(view, account_group_id, source_address)
        .await
        .context("can't build proposal withdraw transaction")
}
