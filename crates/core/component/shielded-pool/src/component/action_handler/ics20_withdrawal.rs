use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;

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

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        state.withdrawal_check(self).await?;
        state.withdrawal_execute(self).await
    }
}
