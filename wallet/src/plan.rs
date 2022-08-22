use std::collections::{BTreeMap, HashMap};

use anyhow::Result;
use penumbra_component::stake::rate::RateData;
use penumbra_component::stake::validator;
use penumbra_crypto::{
    asset::Denom, dex::TradingPair, keys::AddressIndex, memo::MemoPlaintext, transaction::Fee,
    Address, DelegationToken, FullViewingKey, Note, Value, STAKING_TOKEN_ASSET_ID,
    STAKING_TOKEN_DENOM,
};
use penumbra_proto::view::NotesRequest;
use penumbra_transaction::plan::{
    ActionPlan, OutputPlan, SpendPlan, SwapClaimPlan, SwapPlan, TransactionPlan,
};
use penumbra_view::{NoteRecord, ViewClient};
use rand_core::{CryptoRng, RngCore};
use tracing::instrument;

pub async fn validator_definition<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    mut rng: R,
    new_validator: validator::Definition,
    fee: u64,
    source_address: Option<u64>,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    // If the source address is set, send fee change to the same
    // address; otherwise, send it to the default address.
    let (self_address, _dtk) = fvk
        .incoming()
        .payment_address(source_address.unwrap_or(0).into());

    let chain_params = view.chain_params().await?;

    let mut plan = TransactionPlan {
        chain_id: chain_params.chain_id,
        fee: Fee(fee),
        ..Default::default()
    };

    plan.actions
        .push(ActionPlan::ValidatorDefinition(new_validator.into()));

    // Add the required spends, and track change:
    let spend_amount = fee;
    let mut spent_amount = 0;
    let source_index: Option<AddressIndex> = source_address.map(Into::into);
    let notes_to_spend = view
        .notes(NotesRequest {
            fvk_hash: Some(fvk.hash().into()),
            asset_id: Some((*STAKING_TOKEN_ASSET_ID).into()),
            address_index: source_index.map(Into::into),
            amount_to_spend: spend_amount,
            include_spent: false,
        })
        .await?;
    for note_record in notes_to_spend {
        spent_amount += note_record.note.amount();
        plan.actions
            .push(SpendPlan::new(&mut rng, note_record.note, note_record.position).into());
    }
    // Add a change note if we have change left over:
    let change_amount = spent_amount - spend_amount;
    // TODO: support dummy notes, and produce a change output unconditionally.
    // let change_note = if change_amount > 0 { ... } else { /* dummy note */}
    if change_amount > 0 {
        plan.actions.push(
            OutputPlan::new(
                &mut rng,
                Value {
                    amount: change_amount,
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                },
                self_address,
                MemoPlaintext::default(),
            )
            .into(),
        );
    }

    Ok(plan)
}

/// Generate a new transaction plan delegating stake
#[instrument(skip(fvk, view, rng, rate_data, unbonded_amount, fee, source_address))]
pub async fn delegate<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    mut rng: R,
    rate_data: RateData,
    unbonded_amount: u64,
    fee: u64,
    source_address: Option<u64>,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    // If the source address is set, send the delegation tokens to the same
    // address; otherwise, send them to the default address.
    let (self_address, _dtk) = fvk
        .incoming()
        .payment_address(source_address.unwrap_or(0).into());

    let chain_params = view.chain_params().await?;

    let mut plan = TransactionPlan {
        chain_id: chain_params.chain_id,
        fee: Fee(fee),
        ..Default::default()
    };

    // Add the delegation action itself:
    plan.actions
        .push(rate_data.build_delegate(unbonded_amount).into());

    // Add an output to ourselves to record the delegation:
    plan.actions.push(
        OutputPlan::new(
            &mut rng,
            Value {
                amount: rate_data.delegation_amount(unbonded_amount),
                asset_id: DelegationToken::new(rate_data.identity_key).id(),
            },
            self_address,
            MemoPlaintext::default(),
        )
        .into(),
    );

    // Get a list of notes to spend from the view service:
    let spend_amount = unbonded_amount + fee;
    let source_index: Option<AddressIndex> = source_address.map(Into::into);
    let notes_to_spend = view
        .notes(NotesRequest {
            fvk_hash: Some(fvk.hash().into()),
            asset_id: Some((*STAKING_TOKEN_ASSET_ID).into()),
            address_index: source_index.map(Into::into),
            amount_to_spend: spend_amount,
            include_spent: false,
        })
        .await?;

    // Add the required spends, and track change:
    let mut spent_amount = 0;
    for note_record in notes_to_spend {
        spent_amount += note_record.note.amount();
        plan.actions
            .push(SpendPlan::new(&mut rng, note_record.note, note_record.position).into());
    }

    if spent_amount < spend_amount {
        return Err(anyhow::anyhow!(
            "not enough notes to delegate: wanted to delegate {}, have {}",
            spend_amount,
            spent_amount
        ));
    }

    // Add a change note if we have change left over:
    let change_amount = spent_amount - spend_amount;

    // TODO: support dummy notes, and produce a change output unconditionally.
    // let change_note = if change_amount > 0 { ... } else { /* dummy note */}
    if change_amount > 0 {
        plan.actions.push(
            OutputPlan::new(
                &mut rng,
                Value {
                    amount: change_amount,
                    asset_id: *STAKING_TOKEN_ASSET_ID,
                },
                self_address,
                MemoPlaintext::default(),
            )
            .into(),
        );
    }

    Ok(plan)
}

/// Generate a new transaction plan undelegating stake
pub async fn undelegate<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    mut rng: R,
    rate_data: RateData,
    delegation_notes: Vec<NoteRecord>,
    fee: u64,
    source_address: Option<u64>,
) -> Result<TransactionPlan>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    let (self_address, _dtk) = fvk
        .incoming()
        .payment_address(source_address.unwrap_or(0).into());

    let chain_params = view.chain_params().await?;

    let delegation_amount = delegation_notes
        .iter()
        .map(|record| record.note.amount())
        .sum();

    let spend_amount = delegation_amount;

    // Because the outputs of an undelegation are quarantined, we want to
    // avoid any unnecessary change outputs, so we pay fees out of the
    // unbonded amount.
    let unbonded_amount = rate_data.unbonded_amount(delegation_amount);
    let output_amount = unbonded_amount.checked_sub(fee).ok_or_else(|| {
        anyhow::anyhow!(
            "unbonded amount {} from delegation amount {} is insufficient to pay fees {}",
            unbonded_amount,
            delegation_amount,
            fee
        )
    })?;

    let mut plan = TransactionPlan {
        chain_id: chain_params.chain_id,
        fee: Fee(fee),
        ..Default::default()
    };

    // add the undelegation action itself
    plan.actions
        .push(rate_data.build_undelegate(delegation_amount).into());

    // add the outputs for the undelegation
    plan.actions.push(
        OutputPlan::new(
            &mut rng,
            Value {
                amount: output_amount,
                asset_id: *STAKING_TOKEN_ASSET_ID,
            },
            self_address,
            MemoPlaintext::default(),
        )
        .into(),
    );

    let mut spent_amount = 0;
    for note_record in delegation_notes {
        tracing::debug!(?note_record, ?spend_amount);
        spent_amount += note_record.note.amount();
        plan.actions
            .push(SpendPlan::new(&mut rng, note_record.note, note_record.position).into());
    }

    if spent_amount < spend_amount {
        Err(anyhow::anyhow!(
            "not enough delegated tokens to undelegate: wanted to undelegate {}, have {}",
            spend_amount,
            spent_amount,
        ))?;
    }

    Ok(plan)
}

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
#[instrument(skip(fvk, view, rng, swap_nft_note, fee, source_address))]
pub async fn swap_claim<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    mut rng: R,
    swap_nft_note: Note,
    fee: u64,
    source_address: Option<u64>,
) -> Result<TransactionPlan, anyhow::Error>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    tracing::debug!(?swap_nft_note, ?fee, ?source_address);

    return Err(anyhow::anyhow!("not implemented"));

    let chain_params = view.chain_params().await?;

    // TODO: need to fetch clearing price (`BatchSwapOutputData`) for the
    // swap.

    let mut plan = TransactionPlan {
        chain_id: chain_params.chain_id,
        fee: Fee(fee),
        ..Default::default()
    };

    // Add a `SwapClaimPlan` action:
    // plan.actions.push(
    //     SwapClaimPlan::new(
    //         &mut rng,
    //         swap_nft_note,
    //         swap_nft_position,
    //         // The `fvk` is always the claim address.
    //         fvk.address(),
    //         fee,
    //         output_data,
    //         anchor,
    //         trading_pair,
    //     )
    //     .into(),
    // );

    // The value we need to spend is 1 unit of the swap NFT, plus fees.
    let mut value_to_spend: HashMap<Denom, u64> = HashMap::new();
    // *value_to_spend.entry(swap_nft_note.denom()).or_default() += input_value;
    // if fee > 0 {
    //     *value_to_spend
    //         .entry(STAKING_TOKEN_DENOM.clone())
    //         .or_default() += fee;
    // }

    // Add the required spends:
    for (denom, spend_amount) in value_to_spend {
        if spend_amount == 0 {
            continue;
        }

        let source_index: Option<AddressIndex> = source_address.map(Into::into);
        // Select a list of notes that provides at least the required amount.
        let notes_to_spend = view
            .notes(NotesRequest {
                fvk_hash: Some(fvk.hash().into()),
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
                &notes_to_spend
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
                    MemoPlaintext::default(),
                )
                .into(),
            );
        }
    }

    Ok(plan)
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(fvk, view, rng, input_value, fee, source_address))]
pub async fn swap<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    mut rng: R,
    input_value: Value,
    into_denom: Denom,
    fee: u64,
    source_address: Option<u64>,
) -> Result<TransactionPlan, anyhow::Error>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    tracing::debug!(?input_value, ?fee, ?source_address);

    let chain_params = view.chain_params().await?;

    let mut plan = TransactionPlan {
        chain_id: chain_params.chain_id,
        fee: Fee(fee),
        ..Default::default()
    };

    let assets = view.assets().await?;
    let input_denom = assets.get(&input_value.asset_id).ok_or_else(|| {
        anyhow::anyhow!("unknown denomination for asset id {}", input_value.asset_id)
    })?;

    // Determine the canonical order for the assets being swapped.
    // This will determine whether the input amount is assigned to delta_1 or delta_2.
    let trading_pair = TradingPair::canonical_order_for((input_value.asset_id, into_denom.id()))?;

    // If `trading_pair.asset_1` is the input asset, then `delta_1` is the input amount,
    // and `delta_2` is 0.
    //
    // Otherwise, `delta_1` is 0, and `delta_2` is the input amount.
    let delta_1 = if trading_pair.asset_1() == input_value.asset_id {
        input_value.amount
    } else {
        0
    };
    let delta_2 = if trading_pair.asset_1() == input_value.asset_id {
        0
    } else {
        input_value.amount
    };

    // If there is no input, then there is no swap.
    if delta_1 == 0 && delta_1 == 0 {
        return Ok(plan);
    }

    // Add a `SwapPlan` action:
    plan.actions.push(
        SwapPlan::new(
            &mut rng,
            trading_pair,
            delta_1,
            delta_2,
            Fee(fee),
            // The `fvk` is always the claim address.
            // TODO: this should probably select a random address index.
            fvk.incoming().payment_address(0u64.into()).0,
        )
        .into(),
    );

    // The value we need to spend is the input value, plus fees.
    let mut value_to_spend: HashMap<Denom, u64> = HashMap::new();
    *value_to_spend.entry(input_denom.clone()).or_default() += input_value.amount;
    if fee > 0 {
        *value_to_spend
            .entry(STAKING_TOKEN_DENOM.clone())
            .or_default() += fee;
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
                fvk_hash: Some(fvk.hash().into()),
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
                &notes_to_spend
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
                    MemoPlaintext::default(),
                )
                .into(),
            );
        }
    }

    Ok(plan)
}

#[allow(clippy::too_many_arguments)]
#[instrument(skip(fvk, view, rng, values, fee, dest_address, source_address, tx_memo))]
pub async fn send<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    mut rng: R,
    values: &[Value],
    fee: u64,
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

    let chain_params = view.chain_params().await?;

    let mut plan = TransactionPlan {
        chain_id: chain_params.chain_id,
        fee: Fee(fee),
        ..Default::default()
    };

    let assets = view.assets().await?;
    // Track totals of the output values rather than just processing
    // them individually, so we can plan the required spends.
    let mut output_value = HashMap::<Denom, u64>::new();
    for Value { amount, asset_id } in values {
        let denom = assets
            .get(asset_id)
            .ok_or_else(|| anyhow::anyhow!("unknown denomination for asset id {}", asset_id))?;
        output_value.insert(denom.clone(), *amount);
    }

    // Add outputs for the funds we want to send:
    for (denom, amount) in &output_value {
        plan.actions.push(
            OutputPlan::new(
                &mut rng,
                Value {
                    amount: *amount,
                    asset_id: denom.id(),
                },
                dest_address,
                memo.clone(),
            )
            .into(),
        );
    }

    // The value we need to spend is the output value, plus fees.
    let mut value_to_spend = output_value;
    if fee > 0 {
        *value_to_spend
            .entry(STAKING_TOKEN_DENOM.clone())
            .or_default() += fee;
    }

    // Add the required spends:
    for (denom, spend_amount) in value_to_spend {
        // Only produce an output if the amount is greater than zero
        if spend_amount == 0 {
            continue;
        }

        let source_index: Option<AddressIndex> = source_address.map(Into::into);
        // Select a list of notes that provides at least the required amount.
        let notes_to_spend = view
            .notes(NotesRequest {
                fvk_hash: Some(fvk.hash().into()),
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
                &notes_to_spend
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
                    MemoPlaintext::default(),
                )
                .into(),
            );
        }
    }

    Ok(plan)
}

#[instrument(skip(fvk, view, rng))]
pub async fn sweep<V, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    mut rng: R,
) -> Result<Vec<TransactionPlan>, anyhow::Error>
where
    V: ViewClient,
    R: RngCore + CryptoRng,
{
    const SWEEP_COUNT: usize = 8;

    let chain_id = view.chain_params().await?.chain_id;

    let all_notes = view
        .notes(NotesRequest {
            fvk_hash: Some(fvk.hash().into()),
            ..Default::default()
        })
        .await?;

    let mut notes_by_addr_and_denom: BTreeMap<AddressIndex, BTreeMap<_, Vec<NoteRecord>>> =
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
        let (addr, _dtk) = fvk.incoming().payment_address(index);

        for (asset_id, mut records) in notes_by_denom {
            // Sort notes by amount, ascending, so the biggest notes are at the end...
            records.sort_by(|a, b| a.note.value().amount.cmp(&b.note.value().amount));
            // ... so that when we use chunks_exact, we get SWEEP_COUNT sized
            // chunks, ignoring the biggest notes in the remainder.
            for group in records.chunks_exact(SWEEP_COUNT) {
                let mut plan = TransactionPlan {
                    chain_id: chain_id.clone(),
                    fee: Fee(0),
                    ..Default::default()
                };

                for record in group {
                    plan.actions.push(
                        SpendPlan::new(&mut rng, record.note.clone(), record.position).into(),
                    );
                }
                plan.actions.push(
                    OutputPlan::new(
                        &mut rng,
                        Value {
                            amount: group.iter().map(|record| record.note.amount()).sum(),
                            asset_id,
                        },
                        addr,
                        MemoPlaintext::default(),
                    )
                    .into(),
                );

                tracing::debug!(?plan);
                plans.push(plan);
            }
        }
    }

    Ok(plans)
}
