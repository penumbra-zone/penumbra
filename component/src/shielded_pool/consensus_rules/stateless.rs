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
pub fn check_memo_exists_if_outputs_absent_if_not(tx: &Transaction) -> anyhow::Result<()> {
    let num_outputs = tx.outputs().count();
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
}

/// Ensure that we either have at least 2 spends/outputs or none.
pub fn check_never_one_spend_or_one_output(tx: &Transaction) -> anyhow::Result<()> {
    if tx.outputs().count() == 1 {
        Err(anyhow::anyhow!(
            "consensus rule violated: must have either 0 or more than 2 outputs"
        ))
    } else if tx.spends().count() == 1 {
        Err(anyhow::anyhow!(
            "consensus rule violated: must have either 0 or more than 2 spends"
        ))
    } else {
        Ok(())
    }
}
