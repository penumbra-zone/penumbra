use penumbra_transaction::Transaction;

pub fn num_clues_equal_to_num_outputs(tx: &Transaction) -> anyhow::Result<()> {
    if tx.transaction_body().fmd_clues.len() != tx.outputs().count() {
        Err(anyhow::anyhow!(
            "consensus rule violated: must have equal number of outputs and FMD clues"
        ))
    } else {
        Ok(())
    }
}

#[allow(clippy::if_same_then_else)]
pub fn check_memo_exists_if_outputs_absent_if_not(_tx: &Transaction) -> anyhow::Result<()> {
    /*
    let num_outputs = tx.outputs().count();
    Disabling due to: https://github.com/penumbra-zone/penumbra/issues/1416
    if num_outputs > 0 && tx.transaction_body().memo.is_none() {
        Err(anyhow::anyhow!(
            "consensus rule violated: must have memo if outputs present"
        ))
    } else if num_outputs > 0 && tx.transaction_body().memo.is_some() {
        Ok(())
    } else if num_outputs == 0 && tx.transaction_body().memo.is_none() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "consensus rule violated: cannot have memo if no outputs present"
        ))
    }
    */
    Ok(())
}
