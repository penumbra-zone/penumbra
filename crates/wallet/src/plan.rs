use std::collections::BTreeMap;

use anyhow::Context;
use decaf377::Fq;
use rand_core::{CryptoRng, RngCore};
use tracing::instrument;

use penumbra_sdk_dex::swap_claim::SwapClaimPlan;
use penumbra_sdk_keys::keys::AddressIndex;
use penumbra_sdk_proto::view::v1::NotesRequest;
use penumbra_sdk_transaction::{TransactionParameters, TransactionPlan};
pub use penumbra_sdk_view::Planner;
use penumbra_sdk_view::{SpendableNoteRecord, ViewClient};

pub const SWEEP_COUNT: usize = 8;

#[instrument(skip(view, rng))]
pub async fn sweep<V, R>(view: &mut V, mut rng: R) -> anyhow::Result<Vec<TransactionPlan>>
where
    V: ViewClient + Send,
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
async fn claim_unclaimed_swaps<V, R>(
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
        // We found an unclaimed swap, so we can claim it.
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
async fn sweep_notes<V, R>(view: &mut V, mut rng: R) -> anyhow::Result<Vec<TransactionPlan>>
where
    V: ViewClient + Send,
    R: RngCore + CryptoRng,
{
    let gas_prices = view.gas_prices().await?;

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
                planner.set_gas_prices(gas_prices);

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
