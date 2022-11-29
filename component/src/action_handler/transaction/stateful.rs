use std::sync::Arc;

use anyhow::Result;
use penumbra_chain::StateReadExt as _;
use penumbra_storage::State;
use penumbra_transaction::Transaction;

use crate::shielded_pool::{consensus_rules, StateReadExt as _};

pub(super) async fn claimed_anchor_is_valid(
    state: Arc<State>,
    transaction: &Transaction,
) -> Result<()> {
    state.check_claimed_anchor(transaction.anchor).await
}

pub(super) async fn fmd_parameters_valid(
    state: Arc<State>,
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
