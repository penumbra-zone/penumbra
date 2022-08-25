use penumbra_chain::params::FmdParameters;
use penumbra_transaction::Transaction;

// TODO: Currently this is checking that the Clues were made using the most recent
// `FmdParameters` or the parameters from the previous epoch. It would be better
// to check based on height.
pub fn fmd_precision_within_grace_period(
    tx: &Transaction,
    previous_fmd_parameters: FmdParameters,
    current_fmd_parameters: FmdParameters,
) -> anyhow::Result<()> {
    for clue in tx.transaction_body().fmd_clues {
        if clue.precision_bits() == current_fmd_parameters.precision_bits
            || clue.precision_bits() == previous_fmd_parameters.precision_bits
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
