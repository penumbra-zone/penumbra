use std::collections::BTreeMap;

use anyhow::{Context, Result};
use decaf377::Fq;
use rand_core::{CryptoRng, RngCore};
use tracing::instrument;

use penumbra_asset::Value;
use penumbra_dex::swap_claim::SwapClaimPlan;
use penumbra_fee::Fee;
use penumbra_governance::{proposal_state, Proposal, ValidatorVote};
use penumbra_keys::{keys::AddressIndex, Address};
use penumbra_num::Amount;
use penumbra_proto::view::v1::NotesRequest;
use penumbra_stake::rate::RateData;
use penumbra_stake::validator;
use penumbra_transaction::{memo::MemoPlaintext, TransactionParameters, TransactionPlan};
pub use penumbra_view::Planner;
use penumbra_view::{SpendableNoteRecord, ViewClient};

pub async fn validator_definition<V, R>(
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
        .plan(view, source_address)
        .await
        .context("can't build validator definition plan")
}

pub async fn validator_vote<V, R>(
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
        .plan(view, source_address)
        .await
        .context("can't build validator vote plan")
}

/// Generate a new transaction plan delegating stake
#[instrument(skip(view, rng, rate_data, unbonded_amount, fee, source_address))]
pub async fn delegate<V, R>(
    view: &mut V,
    rng: R,
    rate_data: RateData,
    unbonded_amount: Amount,
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
        .plan(view, source_address)
        .await
        .context("can't build delegate plan")
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(view, rng, values, fee, dest_address, source_address_index, tx_memo))]
pub async fn send<V, R>(
    view: &mut V,
    rng: R,
    values: &[Value],
    fee: Fee,
    dest_address: Address,
    source_address_index: AddressIndex,
    tx_memo: Option<MemoPlaintext>,
) -> anyhow::Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    tracing::debug!(
        ?values,
        ?fee,
        ?dest_address,
        ?source_address_index,
        ?tx_memo
    );
    let mut planner = Planner::new(rng);
    planner.fee(fee);
    for value in values.iter().cloned() {
        planner.output(value, dest_address);
    }
    let source_address = view.address_by_index(source_address_index).await?;
    planner
        .memo(tx_memo.unwrap_or_else(|| MemoPlaintext::blank_memo(source_address)))?
        .plan(view, source_address_index)
        .await
        .context("can't build send transaction")
}

#[instrument(skip(view, rng))]
pub async fn sweep<V, R>(view: &mut V, mut rng: R) -> anyhow::Result<Vec<TransactionPlan>>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    let mut plans = Vec::new();

    // First, find any un-claimed swaps and add `SwapClaim` plans for them.
    plans.extend(claim_unclaimed_swaps(view, &mut rng).await?);

    // Finally, sweep dust notes by spending them to their owner's address.
    // This will consolidate small-value notes into larger ones.
    plans.extend(sweep_notes(view, &mut rng).await?);

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

    let app_params = view.app_params().await?;
    let epoch_duration = app_params.sct_params.epoch_duration;

    let unclaimed_swaps = view.unclaimed_swaps().await?;

    for swap in unclaimed_swaps {
        // We found an unspent swap NFT, so we can claim it.
        let swap_plaintext = swap.swap;

        let output_data = swap.output_data;

        let mut plan = TransactionPlan {
            transaction_parameters: TransactionParameters {
                chain_id: app_params.clone().chain_id,
                fee: swap_plaintext.claim_fee.clone(),
                ..Default::default()
            },
            // The transaction doesn't need a memo, because it's to ourselves.
            memo: None,
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

#[instrument(skip(view, rng))]
pub async fn sweep_notes<V, R>(view: &mut V, mut rng: R) -> anyhow::Result<Vec<TransactionPlan>>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    const SWEEP_COUNT: usize = 8;

    let all_notes = view
        .notes(NotesRequest {
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
                let sender_addr = view.address_by_index(index).await?;
                planner.memo(MemoPlaintext::blank_memo(sender_addr))?;

                for record in group {
                    planner.spend(record.note.clone(), record.position);
                }

                let plan = planner
                    .plan(view, index)
                    .await
                    .context("can't build sweep transaction")?;

                tracing::debug!(?plan);
                plans.push(plan);
            }
        }
    }

    Ok(plans)
}

#[instrument(skip(view, rng))]
pub async fn proposal_submit<V, R>(
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
        .proposal_submit(
            proposal,
            view.app_params()
                .await?
                .governance_params
                .proposal_deposit_amount,
        )
        .plan(view, source_address)
        .await
        .context("can't build proposal submit transaction")
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(view, rng))]
pub async fn proposal_withdraw<V, R>(
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
        .plan(view, source_address)
        .await
        .context("can't build proposal withdraw transaction")
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(view, rng))]
pub async fn proposal_deposit_claim<V, R>(
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
        .plan(view, source_address)
        .await
        .context("can't build proposal withdraw transaction")
}
