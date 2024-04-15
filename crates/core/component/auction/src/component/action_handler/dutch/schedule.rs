use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;

use crate::auction::dutch::actions::ActionDutchAuctionSchedule;

#[async_trait]
impl ActionHandler for ActionDutchAuctionSchedule {
    type CheckStatelessContext = ();
    async fn check_stateless(&self, _context: ()) -> Result<()> {
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut _state: S) -> Result<()> {
        Ok(())
    }
}
