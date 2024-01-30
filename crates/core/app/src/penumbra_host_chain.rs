
use anyhow::Context;
use async_trait::async_trait;
use penumbra_ibc::component::HostInterface;
use tendermint::Time;

/// The implementation of HostInterface for Penumbra. This encapsulates all of the chain-specific
/// data that the ibc implementation needs.
#[derive(Clone)]
pub struct PenumbraHost {}

#[async_trait]
impl HostInterface for PenumbraHost {
    async fn get_chain_id<S: cnidarium::StateRead>(state: S) -> anyhow::Result<String> {
        todo!("MERGEBLOCK(erwan)")
    }

    async fn get_revision_number<S: cnidarium::StateRead>(state: S) -> anyhow::Result<u64> {
        todo!()
    }

    async fn get_block_height<S: cnidarium::StateRead>(state: S) -> anyhow::Result<u64> {
        todo!("MERGEBLOCK(erwan)")
    }

    async fn get_block_timestamp<S: cnidarium::StateRead>(
        state: S,
    ) -> anyhow::Result<tendermint::Time> {
        todo!()
    }
}
