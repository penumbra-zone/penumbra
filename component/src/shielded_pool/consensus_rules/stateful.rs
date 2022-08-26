use penumbra_chain::params::FmdParameters;
use penumbra_transaction::Transaction;

const FMD_GRACE_PERIOD_BLOCKS: u64 = 10;

pub fn fmd_precision_within_grace_period(
    tx: &Transaction,
    previous_fmd_parameters: FmdParameters,
    current_fmd_parameters: FmdParameters,
    block_height: u64,
) -> anyhow::Result<()> {
    for clue in tx.transaction_body().fmd_clues {
        // Clue must be using the current `FmdParameters`, or be within
        // `FMD_GRACE_PERIOD_BLOCKS` of the previous `FmdParameters`.
        if clue.precision_bits() == current_fmd_parameters.precision_bits
            || (clue.precision_bits() == previous_fmd_parameters.precision_bits
                && block_height
                    < previous_fmd_parameters.as_of_block_height + FMD_GRACE_PERIOD_BLOCKS)
        {
            continue;
        } else {
            return Err(anyhow::anyhow!(
                "consensus rule violated: invalid clue precision",
            ));
        }
    }
    Ok(())
}
