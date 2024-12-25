use async_trait::async_trait;
use penumbra_sdk_ibc::component::HostInterface;
use penumbra_sdk_sct::component::clock::EpochRead;

use crate::app::StateReadExt;

/// The implementation of [`penumbra_sdk_ibc::component::HostInterface`] for Penumbra.
/// It encapsulates all of the chain-specific data that the ibc implementation needs.
#[derive(Clone)]
pub struct PenumbraHost {}

#[async_trait]
impl HostInterface for PenumbraHost {
    async fn get_chain_id<S: cnidarium::StateRead>(state: S) -> anyhow::Result<String> {
        state.get_chain_id().await
    }

    async fn get_revision_number<S: cnidarium::StateRead>(state: S) -> anyhow::Result<u64> {
        state.get_revision_number().await
    }

    async fn get_block_height<S: cnidarium::StateRead>(state: S) -> anyhow::Result<u64> {
        state.get_block_height().await
    }

    async fn get_block_timestamp<S: cnidarium::StateRead>(
        state: S,
    ) -> anyhow::Result<tendermint::Time> {
        state.get_current_block_timestamp().await
    }
}
