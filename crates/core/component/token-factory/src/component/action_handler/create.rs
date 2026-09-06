//! Action handler for ActionTokenFactoryCreate.

use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateWrite;
use cnidarium_component::ActionHandler;
use penumbra_sdk_proto::StateWriteProto as _;

use crate::{
    component::{event, StateReadExt as _, StateWriteExt as _},
    ActionTokenFactoryCreate,
};

#[async_trait]
impl ActionHandler for ActionTokenFactoryCreate {
    type CheckStatelessContext = ();

    async fn check_stateless(&self, _context: ()) -> Result<()> {
        // Stateless validation is done in the constructor:
        // - Initial supply limit
        // - Metadata base matches nonce
        // - Metadata field length limits
        Ok(())
    }

    async fn check_and_execute<S: StateWrite>(&self, mut state: S) -> Result<()> {
        // Governance gate: the factory is dormant until enabled, and the mintable
        // (unlimited-supply) path is enabled separately from fair-launch.
        let params = state.token_factory_parameters().await?;
        anyhow::ensure!(
            params.token_factory_enabled,
            "token factory is disabled; it must be enabled by governance"
        );
        if self.enable_mint {
            anyhow::ensure!(
                params.mintable_enabled,
                "mintable tokens are disabled; the mint-capability path must be enabled by governance"
            );
        }

        // Check that the nonce hasn't been used before and mark it as used.
        // This prevents replay attacks and ensures each token ID is unique.
        state.token_factory_mark_nonce_used(&self.nonce).await?;

        // Store the token metadata for queries
        state.token_factory_put_metadata(&self.nonce, &self.metadata);

        // Emit event for indexers
        state.record_proto(event::token_factory_create(self));

        // The value balance (fixed supply) is handled by the transaction's
        // balance check. No additional state changes needed.

        Ok(())
    }
}
