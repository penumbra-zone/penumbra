use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateRead;
use cnidarium::StateWrite;
use cnidarium_component::ChainStateReadExt;
use penumbra_chain::component::StateReadExt;

#[derive(wrapper_derive::StateRead, wrapper_derive::StateWrite)]
pub(crate) struct StateDeltaWrapper<'a, S: StateRead + StateWrite>(pub(crate) &'a mut S);

#[async_trait]
impl<'a, S: StateRead + StateWrite> ChainStateReadExt for StateDeltaWrapper<'a, S> {
    async fn get_chain_id(&self) -> Result<String> {
        self.0.get_chain_id().await
    }

    async fn get_revision_number(&self) -> Result<u64> {
        self.0.get_revision_number().await
    }

    async fn get_block_height(&self) -> Result<u64> {
        self.0.get_block_height().await
    }

    async fn get_block_timestamp(&self) -> Result<tendermint::Time> {
        self.0.get_block_timestamp().await
    }
}
