use std::sync::Arc;

use anyhow::{ensure, Result};
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use cnidarium_component::ActionHandler;
use penumbra_ibc::StateReadExt as _;

use crate::{
    component::transfer::{Ics20TransferReadExt as _, Ics20TransferWriteExt as _},
    Ics20Withdrawal,
};

#[async_trait]
impl ActionHandler for Ics20Withdrawal {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        self.validate()
    }

    async fn check_historical<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        ensure!(
            state
                .get_ibc_params()
                .await?
                .outbound_ics20_transfers_enabled,
            "transaction an ICS20 withdrawal, but outbound ICS20 withdrawals are not enabled"
        );
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.withdrawal_check(self).await?;
        state.withdrawal_execute(self).await
    }
}
