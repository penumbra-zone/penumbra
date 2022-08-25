use penumbra_crypto::NotePayload;
use penumbra_transaction::Transaction;

pub fn num_clues_equal_to_num_outputs(tx: &Transaction) -> anyhow::Result<()> {
    if tx.transaction_body().fmd_clues.len() != tx.note_payloads().count() {
        return Err(anyhow::anyhow!(
            "consensus rule violated: must have equal number of outputs and FMD clues"
        ));
    } else {
        Ok(())
    }
}
