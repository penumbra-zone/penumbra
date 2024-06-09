use anyhow::{ensure, Result};
use cnidarium::StateRead;
use penumbra_fee::component::StateReadExt as _;
use penumbra_sct::component::clock::EpochRead;
use penumbra_sct::component::tree::VerificationExt;
use penumbra_shielded_pool::component::StateReadExt as _;
use penumbra_shielded_pool::fmd;
use penumbra_transaction::gas::GasCost;
use penumbra_transaction::{Transaction, TransactionParameters};

use crate::app::StateReadExt;

pub async fn tx_parameters_historical_check<S: StateRead>(
    state: S,
    transaction: &Transaction,
) -> Result<()> {
    let TransactionParameters {
        chain_id,
        expiry_height,
        // This is checked in `fee_greater_than_base_fee` against the whole
        // transaction, for convenience.
        fee: _,
        // IMPORTANT: Adding a transaction parameter? Then you **must** add a SAFETY
        // argument here to justify why it is safe to validate against a historical
        // state.
    } = transaction.transaction_parameters();

    // SAFETY: This is safe to do in a **historical** check because the chain's actual
    // id cannot change during transaction processing.
    chain_id_is_correct(&state, chain_id).await?;
    // SAFETY: This is safe to do in a **historical** check because the chain's current
    // block height cannot change during transaction processing.
    expiry_height_is_valid(&state, expiry_height).await?;
    // SAFETY: This is safe to do in a **historical** check as long as the current gas prices
    // are static, or set in the previous block.
    fee_greater_than_base_fee(&state, transaction).await?;

    Ok(())
}

pub async fn chain_id_is_correct<S: StateRead>(state: S, tx_chain_id: String) -> Result<()> {
    let chain_id = state.get_chain_id().await?;

    // The chain ID in the transaction must exactly match the current chain ID.
    ensure!(
        tx_chain_id == chain_id,
        "transaction chain ID '{}' must match the current chain ID '{}'",
        tx_chain_id,
        chain_id
    );
    Ok(())
}

pub async fn expiry_height_is_valid<S: StateRead>(state: S, expiry_height: u64) -> Result<()> {
    let current_height = state.get_block_height().await?;

    // A zero expiry height means that the transaction is valid indefinitely.
    if expiry_height == 0 {
        return Ok(());
    }

    // Otherwise, the expiry height must be greater than or equal to the current block height.
    ensure!(
        expiry_height >= current_height,
        "transaction expiry height '{}' must be greater than or equal to the current block height '{}'",
        expiry_height,
        current_height
    );

    Ok(())
}

pub async fn fmd_parameters_valid<S: StateRead>(state: S, transaction: &Transaction) -> Result<()> {
    let meta_params = state
        .get_shielded_pool_params()
        .await
        .expect("chain params request must succeed")
        .fmd_meta_params;
    let previous_fmd_parameters = state
        .get_previous_fmd_parameters()
        .await
        .expect("chain params request must succeed");
    let current_fmd_parameters = state
        .get_current_fmd_parameters()
        .await
        .expect("chain params request must succeed");
    let height = state.get_block_height().await?;
    fmd_precision_within_grace_period(
        transaction,
        meta_params,
        previous_fmd_parameters,
        current_fmd_parameters,
        height,
    )
}

#[tracing::instrument(
    skip_all,
    fields(
        current_fmd.precision_bits = current_fmd_parameters.precision.bits(),
        previous_fmd.precision_bits = previous_fmd_parameters.precision.bits(),
        previous_fmd.as_of_block_height = previous_fmd_parameters.as_of_block_height,
        block_height,
    )
)]
pub fn fmd_precision_within_grace_period(
    tx: &Transaction,
    meta_params: fmd::MetaParameters,
    previous_fmd_parameters: fmd::Parameters,
    current_fmd_parameters: fmd::Parameters,
    block_height: u64,
) -> anyhow::Result<()> {
    for clue in tx
        .transaction_body()
        .detection_data
        .unwrap_or_default()
        .fmd_clues
    {
        // Clue must be using the current `fmd::Parameters`, or be within
        // `fmd_grace_period_blocks` of the previous `fmd::Parameters`.
        let clue_precision = clue.precision()?;
        let using_current_precision = clue_precision == current_fmd_parameters.precision;
        let using_previous_precision = clue_precision == previous_fmd_parameters.precision;
        let within_grace_period = block_height
            < previous_fmd_parameters.as_of_block_height + meta_params.fmd_grace_period_blocks;
        if using_current_precision || (using_previous_precision && within_grace_period) {
            continue;
        } else {
            tracing::error!(
                %clue_precision,
                %using_current_precision,
                %using_previous_precision,
                %within_grace_period,
                "invalid clue precision"
            );
            anyhow::bail!("consensus rule violated: invalid clue precision");
        }
    }
    Ok(())
}

pub async fn claimed_anchor_is_valid<S: StateRead>(
    state: S,
    transaction: &Transaction,
) -> Result<()> {
    state.check_claimed_anchor(transaction.anchor).await
}

pub async fn fee_greater_than_base_fee<S: StateRead>(
    state: S,
    transaction: &Transaction,
) -> Result<()> {
    // Check whether the user is requesting to pay fees in the native token
    // or in an alternative fee token.
    let user_supplied_fee = transaction.transaction_body().transaction_parameters.fee;

    let current_gas_prices =
        if user_supplied_fee.asset_id() == *penumbra_asset::STAKING_TOKEN_ASSET_ID {
            state
                .get_gas_prices()
                .await
                .expect("gas prices must be present in state")
        } else {
            let alt_gas_prices = state
                .get_alt_gas_prices()
                .await
                .expect("alt gas prices must be present in state");
            alt_gas_prices
                .into_iter()
                .find(|prices| prices.asset_id == user_supplied_fee.asset_id())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "fee token {} not recognized by the chain",
                        user_supplied_fee.asset_id()
                    )
                })?
        };

    // Double check that the gas price assets match.
    ensure!(
        current_gas_prices.asset_id == user_supplied_fee.asset_id(),
        "unexpected mismatch between fee and queried gas prices (expected: {}, found: {})",
        user_supplied_fee.asset_id(),
        current_gas_prices.asset_id,
    );

    let transaction_base_fee = current_gas_prices.fee(&transaction.gas_cost());

    ensure!(
        user_supplied_fee.amount() >= transaction_base_fee.amount(),
        "fee must be greater than or equal to the transaction base price (supplied: {}, base: {})",
        user_supplied_fee.amount(),
        transaction_base_fee.amount(),
    );

    Ok(())
}
