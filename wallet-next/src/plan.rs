use anyhow::Result;
use penumbra_chain::params::ChainParams;
use penumbra_crypto::{
    keys::DiversifierIndex, memo::MemoPlaintext, DelegationToken, FullViewingKey, Value,
    STAKING_TOKEN_ASSET_ID, STAKING_TOKEN_DENOM,
};
use penumbra_proto::view::NotesRequest;
use penumbra_stake::rate::RateData;
use penumbra_stake::validator;
use penumbra_transaction::{
    plan::{ActionPlan, OutputPlan, SpendPlan, TransactionPlan},
    Fee,
};
use penumbra_view::ViewClient;
use rand_core::{CryptoRng, RngCore};
use tracing::instrument;

pub async fn validator_definition<V, R>(
    fvk: &FullViewingKey,
    mut view: V,
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

    // TODO: add this to the view service
    //let chain_params = view.chain_params().await?;
    let chain_params = ChainParams::default();

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
    let source_index: Option<DiversifierIndex> = source_address.map(Into::into);
    let notes_to_spend = view
        .notes(NotesRequest {
            fvk_hash: Some(fvk.hash().into()),
            asset_id: Some((*STAKING_TOKEN_ASSET_ID).into()),
            diversifier_index: source_index.map(Into::into),
            amount_to_spend: spend_amount,
            include_spent: false,
        })
        .await?;
    for note_record in notes_to_spend {
        let note = note_record.note;
        spent_amount += note.amount();
        plan.actions
            .push(SpendPlan::new(&mut rng, note.clone(), note_record.position).into());
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
    mut view: V,
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

    // TODO: add this to the view service
    //let chain_params = view.chain_params().await?;
    let chain_params = ChainParams::default();

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
    let source_index: Option<DiversifierIndex> = source_address.map(Into::into);
    let notes_to_spend = view
        .notes(NotesRequest {
            fvk_hash: Some(fvk.hash().into()),
            asset_id: Some((*STAKING_TOKEN_ASSET_ID).into()),
            diversifier_index: source_index.map(Into::into),
            amount_to_spend: spend_amount,
            include_spent: false,
        })
        .await?;

    // Add the required spends, and track change:
    let mut spent_amount = 0;
    for note_record in notes_to_spend {
        tracing::debug!(?note_record, ?spent_amount);
        spent_amount += note_record.note.amount();
        plan.actions.push(
            SpendPlan::new(
                &mut rng,
                note_record.note,
                0u64.into(), // TODO: record the position in the NoteRecord so we don't have to make this up
            )
            .into(),
        );
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
