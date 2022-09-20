use penumbra_tct::Position;
use std::collections::{BTreeMap, HashMap};
use tonic::transport::Channel;

use anyhow::{anyhow, Context, Result};
use penumbra_component::stake::rate::RateData;
use penumbra_component::stake::validator;
use penumbra_crypto::{
    asset::Denom,
    dex::TradingPair,
    dex::{swap::SwapPlaintext, BatchSwapOutputData},
    keys::AddressIndex,
    memo::MemoPlaintext,
    transaction::Fee,
    Address, FullViewingKey, Note, Value,
};
use penumbra_proto::{
    client::specific::{specific_query_client::SpecificQueryClient, BatchSwapOutputDataRequest},
    view::NotesRequest,
};
use penumbra_transaction::{
    action::{Proposal, ValidatorVote},
    plan::{MemoPlan, OutputPlan, SpendPlan, SwapClaimPlan, SwapPlan, TransactionPlan},
};
use penumbra_view::{SpendableNoteRecord, ViewClient};
use rand_core::{CryptoRng, RngCore};
use tracing::instrument;

pub mod balance;
mod planner;
pub use planner::{Balance, Planner};

pub async fn validator_definition<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    rng: R,
    new_validator: validator::Definition,
    fee: Fee,
    source_address: Option<u64>,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    Planner::new(rng)
        .fee(fee)
        .validator_definition(new_validator)
        .plan(view, fvk, source_address.map(Into::into))
        .await
        .context("can't build validator definition plan")
}

pub async fn validator_vote<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    rng: R,
    vote: ValidatorVote,
    fee: Fee,
    source_address: Option<u64>,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    Planner::new(rng)
        .fee(fee)
        .validator_vote(vote)
        .plan(view, fvk, source_address.map(Into::into))
        .await
        .context("can't build validator vote plan")
}

/// Generate a new transaction plan delegating stake
#[instrument(skip(fvk, view, rng, rate_data, unbonded_amount, fee, source_address))]
pub async fn delegate<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    rng: R,
    rate_data: RateData,
    unbonded_amount: u64,
    fee: Fee,
    source_address: Option<u64>,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    Planner::new(rng)
        .fee(fee)
        .delegate(unbonded_amount, rate_data)
        .plan(view, fvk, source_address.map(Into::into))
        .await
        .context("can't build delegate plan")
}

/// Generate a new transaction plan undelegating stake
pub async fn undelegate<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    rng: R,
    rate_data: RateData,
    delegation_notes: Vec<SpendableNoteRecord>,
    fee: Fee,
    source_address: Option<u64>,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    let delegation_amount = delegation_notes
        .iter()
        .map(|record| record.note.amount())
        .sum();

    let mut planner = Planner::new(rng);
    planner.fee(fee).undelegate(delegation_amount, rate_data);
    for record in delegation_notes {
        planner.spend(record.note, record.position);
    }

    planner
        .plan(view, fvk, source_address.map(Into::into))
        .await
        .context("can't build undelegate plan")
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
#[instrument(skip(
    _fvk,
    view,
    rng,
    swap_plaintext,
    swap_nft_note,
    swap_nft_position,
    output_data
))]
pub async fn swap_claim<V, R>(
    _fvk: &FullViewingKey,
    view: &mut V,
    mut rng: R,
    swap_plaintext: SwapPlaintext,
    swap_nft_note: Note,
    swap_nft_position: Position,
    output_data: BatchSwapOutputData,
) -> Result<TransactionPlan, anyhow::Error>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    tracing::debug!(?swap_plaintext, ?swap_nft_note);

    let chain_params = view.chain_params().await?;

    let mut plan = TransactionPlan {
        chain_id: chain_params.chain_id,
        fee: swap_plaintext.claim_fee.clone(),
        // The transaction doesn't need a memo, because it's to ourselves.
        memo_plan: None,
        ..Default::default()
    };

    let epoch_duration = chain_params.epoch_duration;

    // Add a `SwapClaimPlan` action:
    plan.actions.push(
        SwapClaimPlan::new(
            &mut rng,
            swap_plaintext,
            swap_nft_note,
            swap_nft_position,
            epoch_duration,
            output_data,
        )
        .into(),
    );

    // Nothing needs to be spent, since the fee is pre-paid and the
    // swap NFT will be automatically consumed when the SwapClaim action
    // is processed by the validators.

    Ok(plan)
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(fvk, view, rng, input_value, swap_fee, swap_claim_fee, source_address))]
pub async fn swap<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    mut rng: R,
    input_value: Value,
    into_denom: Denom,
    swap_fee: Fee,
    swap_claim_fee: Fee,
    source_address: Option<u64>,
) -> Result<TransactionPlan, anyhow::Error>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    tracing::debug!(?input_value, ?swap_fee, ?swap_claim_fee, ?source_address);

    let chain_params = view.chain_params().await?;

    let mut plan = TransactionPlan {
        chain_id: chain_params.chain_id,
        fee: swap_fee.clone(),
        // Swap will create outputs, so we add a memo.
        memo_plan: Some(MemoPlan::new(&mut rng, MemoPlaintext::default())),
        ..Default::default()
    };

    let assets = view.assets().await?;
    let input_denom = assets.get(&input_value.asset_id).ok_or_else(|| {
        anyhow::anyhow!("unknown denomination for asset id {}", input_value.asset_id)
    })?;
    let swap_fee_denom = assets.get(&swap_fee.asset_id()).ok_or_else(|| {
        anyhow::anyhow!("unknown denomination for asset id {}", swap_fee.asset_id())
    })?;
    let swap_claim_fee_denom = assets.get(&swap_claim_fee.asset_id()).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown denomination for asset id {}",
            swap_claim_fee.asset_id()
        )
    })?;

    // Determine the canonical order for the assets being swapped.
    // This will determine whether the input amount is assigned to delta_1 or delta_2.
    let trading_pair = TradingPair::canonical_order_for((input_value.asset_id, into_denom.id()))?;

    // If `trading_pair.asset_1` is the input asset, then `delta_1` is the input amount,
    // and `delta_2` is 0.
    //
    // Otherwise, `delta_1` is 0, and `delta_2` is the input amount.
    let (delta_1, delta_2) = if trading_pair.asset_1() == input_value.asset_id {
        (input_value.amount, 0)
    } else {
        (0, input_value.amount)
    };

    // If there is no input, then there is no swap.
    if delta_1 == 0 && delta_2 == 0 {
        return Err(anyhow!("No input value for swap"));
    }

    // If a source address was specified, use it for the swap, otherwise,
    // use the default address.
    let (claim_address, _dtk_d) = fvk
        .incoming()
        .payment_address(source_address.unwrap_or(0).into());

    // Create the `SwapPlaintext` representing the swap to be performed:
    let swap_plaintext = SwapPlaintext::from_parts(
        trading_pair,
        delta_1,
        delta_2,
        swap_claim_fee.clone(),
        claim_address,
    )
    .map_err(|_| anyhow!("error generating swap plaintext"))?;

    // Add a `SwapPlan` action:
    plan.actions
        .push(SwapPlan::new(&mut rng, swap_plaintext).into());

    // The value we need to spend is the input value, plus fees.
    let mut value_to_spend: HashMap<Denom, u64> = HashMap::new();
    *value_to_spend.entry(input_denom.clone()).or_default() += input_value.amount;
    if swap_fee.amount() > 0 {
        *value_to_spend.entry(swap_fee_denom.clone()).or_default() += swap_fee.amount();
    }
    // The fee for the swap claim is pre-paid at this time.
    if swap_claim_fee.amount() > 0 {
        *value_to_spend
            .entry(swap_claim_fee_denom.clone())
            .or_default() += swap_claim_fee.amount();
    }

    // Add the required spends:
    for (denom, spend_amount) in value_to_spend {
        if spend_amount == 0 {
            continue;
        }

        let source_index: Option<AddressIndex> = source_address.map(Into::into);
        // Select a list of notes that provides at least the required amount.
        let notes_to_spend = view
            .notes(NotesRequest {
                account_id: Some(fvk.hash().into()),
                asset_id: Some(denom.id().into()),
                address_index: source_index.map(Into::into),
                amount_to_spend: spend_amount,
                include_spent: false,
            })
            .await?;
        if notes_to_spend.is_empty() {
            // Shouldn't happen because the other side checks this, but just in case...
            return Err(anyhow::anyhow!("not enough notes to spend",));
        }

        let change_address_index: u64 = fvk
            .incoming()
            .index_for_diversifier(
                notes_to_spend
                    .last()
                    .expect("notes_to_spend should never be empty")
                    .note
                    .diversifier(),
            )
            .try_into()?;

        let (change_address, _dtk) = fvk.incoming().payment_address(change_address_index.into());
        let spent: u64 = notes_to_spend
            .iter()
            .map(|note_record| note_record.note.amount())
            .sum();

        // Spend each of the notes we selected.
        for note_record in notes_to_spend {
            plan.actions
                .push(SpendPlan::new(&mut rng, note_record.note, note_record.position).into());
        }

        // Find out how much change we have and whether to add a change output.
        let change = spent - spend_amount;
        if change > 0 {
            plan.actions.push(
                OutputPlan::new(
                    &mut rng,
                    Value {
                        amount: change,
                        asset_id: denom.id(),
                    },
                    change_address,
                )
                .into(),
            );
        }
    }

    // Add clue plans for `Output`s.
    let fmd_params = view.fmd_parameters().await?;
    let precision_bits = fmd_params.precision_bits;
    plan.add_all_clue_plans(&mut rng, precision_bits.into());
    Ok(plan)
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(fvk, view, rng, values, fee, dest_address, source_address, tx_memo))]
pub async fn send<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    rng: R,
    values: &[Value],
    fee: Fee,
    dest_address: Address,
    source_address: Option<u64>,
    tx_memo: Option<String>,
) -> Result<TransactionPlan, anyhow::Error>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    tracing::debug!(?values, ?fee, ?dest_address, ?source_address, ?tx_memo);
    let memo = if let Some(input_memo) = tx_memo {
        input_memo.as_bytes().try_into()?
    } else {
        MemoPlaintext::default()
    };

    let mut planner = Planner::new(rng);
    planner.fee(fee);
    for value in values.iter().cloned() {
        planner.output(value, dest_address);
    }
    planner
        .memo(memo)
        .plan(view, fvk, source_address.map(Into::into))
        .await
        .context("can't build send transaction")
}

#[instrument(skip(fvk, view, rng))]
pub async fn sweep<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    mut rng: R,
    specific_client: SpecificQueryClient<Channel>,
) -> Result<Vec<TransactionPlan>, anyhow::Error>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    let mut plans = Vec::new();

    // First, find any un-claimed swaps and add `SwapClaim` plans for them.
    plans.extend(claim_unclaimed_swaps(fvk, view, &mut rng, specific_client).await?);

    // Finally, sweep dust notes by spending them to their owner's address.
    // This will consolidate small-value notes into larger ones.
    plans.extend(sweep_notes(fvk, view, &mut rng).await?);

    Ok(plans)
}

#[instrument(skip(fvk, view, rng))]
pub async fn claim_unclaimed_swaps<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    mut rng: R,
    mut specific_client: SpecificQueryClient<Channel>,
) -> Result<Vec<TransactionPlan>, anyhow::Error>
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

    // Unclaimed swaps will appear as Swap NFT notes.
    // We can find unclaimed swaps by first searching for all transactions containing a swap,
    // then finding all unspent notes associated with a swap transaction.
    let txs = view.transactions(None, None).await?;

    // TODO: should we do some tokio magic to make this concurrent?
    // Fetch all spendable notes ahead of time so we can see which swap NFTs are unspent.
    let all_notes = view
        .notes(NotesRequest {
            account_id: Some(fvk.hash().into()),
            ..Default::default()
        })
        .await?;
    for (block_height, tx) in txs.iter() {
        for swap in tx.swaps() {
            // See if the swap is unspent.
            let swap_nft = swap.body.swap_nft.clone();
            let swap_nft_record = all_notes
                .iter()
                .find(|note_record| note_record.note_commitment == swap_nft.note_commitment)
                .cloned();

            if let Some(swap_nft_record) = swap_nft_record {
                assert!(*block_height == swap_nft_record.height_created);
                // We found an unspent swap NFT, so we can claim it.
                // Decrypt the swap ciphertext and construct a SwapClaim.
                let swap_ciphertext = swap.body.swap_ciphertext.clone();
                let epk = swap.body.swap_nft.ephemeral_key;
                let ivk = fvk.incoming();
                let swap_plaintext = swap_ciphertext.decrypt2(ivk, &epk)?;

                let output_data = specific_client
                    .batch_swap_output_data(BatchSwapOutputDataRequest {
                        height: swap_nft_record.height_created,
                        trading_pair: Some(swap_plaintext.trading_pair.into()),
                    })
                    .await?
                    .into_inner()
                    .try_into()
                    .context("cannot parse batch swap output data")?;

                let mut plan = TransactionPlan {
                    chain_id: chain_params.clone().chain_id,
                    fee: swap_plaintext.claim_fee.clone(),
                    // The transaction doesn't need a memo, because it's to ourselves.
                    memo_plan: None,
                    ..Default::default()
                };
                let action_plan = SwapClaimPlan::new(
                    &mut rng,
                    swap_plaintext,
                    swap_nft_record.note,
                    swap_nft_record.position,
                    epoch_duration,
                    output_data,
                )
                .into();
                plan.actions.push(action_plan);
                plans.push(plan);
            }
        }
    }

    Ok(plans)
}

#[instrument(skip(fvk, view, rng))]
pub async fn sweep_notes<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    mut rng: R,
) -> Result<Vec<TransactionPlan>, anyhow::Error>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    const SWEEP_COUNT: usize = 8;

    let all_notes = view
        .notes(NotesRequest {
            account_id: Some(fvk.hash().into()),
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
                planner.memo(MemoPlaintext::default());

                for record in group {
                    planner.spend(record.note.clone(), record.position);
                }

                let plan = planner
                    .plan(view, fvk, Some(index))
                    .await
                    .context("can't build sweep transaction")?;

                tracing::debug!(?plan);
                plans.push(plan);
            }
        }
    }

    Ok(plans)
}

#[instrument(skip(fvk, view, rng))]
pub async fn proposal_submit<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    rng: R,
    proposal: Proposal,
    fee: Fee,
    source_address: Option<u64>,
) -> anyhow::Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    Planner::new(rng)
        .fee(fee)
        .proposal_submit(proposal)
        .plan(view, fvk, source_address.map(Into::into))
        .await
        .context("can't build proposal submit transaction")
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(fvk, view, rng))]
pub async fn proposal_withdraw<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    rng: R,
    proposal_id: u64,
    deposit_refund_address: Address,
    reason: String,
    fee: Fee,
    source_address: Option<u64>,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    Planner::new(rng)
        .fee(fee)
        .proposal_withdraw(proposal_id, deposit_refund_address, reason)
        .plan(view, fvk, source_address.map(Into::into))
        .await
        .context("can't build proposal withdraw transaction")
}
