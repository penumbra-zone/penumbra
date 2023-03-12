use anyhow::Result;
use ibc::timestamp::Timestamp;
use penumbra_chain::StateReadExt as _;
use penumbra_storage::StateRead;
use penumbra_transaction::Transaction;

use crate::shielded_pool::{consensus_rules, StateReadExt as _};

pub(super) async fn claimed_anchor_is_valid<S: StateRead>(
    state: S,
    transaction: &Transaction,
) -> Result<()> {
    state.check_claimed_anchor(transaction.anchor).await
}

pub(super) async fn fmd_parameters_valid<S: StateRead>(
    state: S,
    transaction: &Transaction,
) -> Result<()> {
    let previous_fmd_parameters = state
        .get_previous_fmd_parameters()
        .await
        .expect("chain params request must succeed");
    let current_fmd_parameters = state
        .get_current_fmd_parameters()
        .await
        .expect("chain params request must succeed");
    let height = state.get_block_height().await?;
    consensus_rules::stateful::fmd_precision_within_grace_period(
        transaction,
        previous_fmd_parameters,
        current_fmd_parameters,
        height,
    )
}

pub(super) async fn timestamps_are_valid<S: StateRead>(
    state: S,
    transaction: &Transaction,
) -> Result<()> {
    let current_time: Timestamp = state.get_block_timestamp().await?.into();

    let after = transaction.transaction_body().valid_after;

    let before = transaction.transaction_body().valid_before;

    if current_time.check_expiry(&after) == ibc::timestamp::Expiry::Expired {
        anyhow::bail!("Too late!");
    }

    if before.check_expiry(&current_time) == ibc::timestamp::Expiry::Expired {
        anyhow::bail!("Too early!");
    }

    if transaction.transaction_body().expiry_height != 0 {
        let current_block = state.get_block_height().await?;
        if current_block > transaction.transaction_body().expiry_height {
            anyhow::bail!("Too late!");
        }
    }

    Ok(())
}
