use std::sync::Arc;

use anyhow::{ensure, Result};
use cnidarium::{StateRead, StateWrite};
use penumbra_sdk_ibc::{component::HostInterface, StateReadExt as _};

use crate::component::transfer::{Ics20TransferReadExt as _, Ics20TransferWriteExt as _};
use crate::component::Ics20WithdrawalWithHandler;

impl<HI: HostInterface> Ics20WithdrawalWithHandler<HI> {
    pub async fn check_stateless(&self, _context: ()) -> Result<()> {
        self.action().validate()
    }

    pub async fn check_historical<S: StateRead + 'static>(&self, state: Arc<S>) -> Result<()> {
        ensure!(
            state
                .get_ibc_params()
                .await?
                .outbound_ics20_transfers_enabled,
            "transaction an ICS20 withdrawal, but outbound ICS20 withdrawals are not enabled"
        );
        Ok(())
    }

    pub async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        let current_block_time = HI::get_block_timestamp(&state).await?;
        state
            .withdrawal_check(self.action(), current_block_time)
            .await?;
        state.withdrawal_execute(self.action()).await
    }
}
