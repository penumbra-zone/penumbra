//! Action handler for ActionTokenFactoryMint.

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sdk_proto::StateWriteProto as _;

use crate::{component::event, ActionTokenFactoryMint};

#[async_trait]
impl ActionHandler for ActionTokenFactoryMint {
    type CheckStatelessContext = ();

    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Stateless validation is done in the constructor:
        // - Amount is non-zero
        // - Amount doesn't exceed MAX_MINT_AMOUNT
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // Replay protection is provided by the value balance system:
        // - The action consumes mint NFT (seq=N)
        // - The action produces mint NFT (seq=N+1)
        //
        // Since each NFT with a specific sequence number can only exist once
        // in the shielded pool, the value balance check ensures that each
        // mint operation can only be performed once.

        // Emit event for indexers
        state.record_proto(event::token_factory_mint(self));

        Ok(())
    }
}
