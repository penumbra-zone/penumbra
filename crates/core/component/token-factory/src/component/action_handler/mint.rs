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
        // Re-validate here rather than trusting the constructor.
        //
        // These are PROTOCOL invariants, not deserialisation side effects. The
        // constructor is currently the only path (fields are private, and
        // TryFrom<proto> calls it), but a future refactor that builds an action
        // internally would silently drop the mint cap. check_stateless is the
        // documented place for exactly this, and it is cheap.
        if self.amount().value() == 0 {
            anyhow::bail!("token factory mint amount must be non-zero");
        }
        if self.amount().value() > crate::MAX_MINT_AMOUNT {
            anyhow::bail!(
                "token factory mint amount {} exceeds maximum {}",
                self.amount().value(),
                crate::MAX_MINT_AMOUNT
            );
        }
        if self.current_seq() == u64::MAX {
            anyhow::bail!("token factory mint sequence would overflow");
        }
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
