use anyhow::Result;
use cnidarium::StateRead;
use penumbra_fee::component::StateReadExt as _;
use penumbra_sct::component::clock::EpochRead;
use penumbra_sct::component::tree::VerificationExt;
use penumbra_shielded_pool::component::StateReadExt as _;
use penumbra_shielded_pool::fmd;
use penumbra_transaction::gas::GasCost;
use penumbra_transaction::Transaction;

const FMD_GRACE_PERIOD_BLOCKS: u64 = 10;

pub async fn fmd_parameters_valid<S: StateRead>(state: S, transaction: &Transaction) -> Result<()> {
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
        previous_fmd_parameters,
        current_fmd_parameters,
        height,
    )
}

#[tracing::instrument(
    skip_all,
    fields(
        current_fmd.precision_bits = current_fmd_parameters.precision_bits,
        previous_fmd.precision_bits = previous_fmd_parameters.precision_bits,
        previous_fmd.as_of_block_height = previous_fmd_parameters.as_of_block_height,
        block_height,
    )
)]
pub fn fmd_precision_within_grace_period(
    tx: &Transaction,
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
        // `FMD_GRACE_PERIOD_BLOCKS` of the previous `fmd::Parameters`.
        let clue_precision = clue.precision_bits();
        let using_current_precision = clue_precision == current_fmd_parameters.precision_bits;
        let using_previous_precision = clue_precision == previous_fmd_parameters.precision_bits;
        let within_grace_period =
            block_height < previous_fmd_parameters.as_of_block_height + FMD_GRACE_PERIOD_BLOCKS;
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
    let current_gas_prices = state
        .get_gas_prices()
        .await
        .expect("gas prices must be present in state");

    let transaction_base_price = current_gas_prices.fee(&transaction.gas_cost());

    if transaction
        .transaction_body()
        .transaction_parameters
        .fee
        .amount()
        >= transaction_base_price
    {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "consensus rule violated: paid transaction fee must be greater than or equal to transaction's base fee"
        ))
    }
}
