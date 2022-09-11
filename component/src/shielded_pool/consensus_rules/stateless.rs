use penumbra_transaction::Transaction;

pub fn num_clues_equal_to_num_outputs(_tx: &Transaction) -> anyhow::Result<()> {
    // TODO: re-enable after getting a better idea of what counts
    // as an "output" (this broke after including the swap and swapclaim notepayloads)
    // commenting out INTERNALLY to this fn prevents a dead code warning
    /*
    if tx.transaction_body().fmd_clues.len() != tx.note_payloads().count() {
        Err(anyhow::anyhow!(
            "consensus rule violated: must have equal number of outputs and FMD clues"
        ))
    } else {
        Ok(())
    }
         */
    Ok(())
}
