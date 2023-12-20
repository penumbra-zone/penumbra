use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateRead;

#[async_trait]
pub trait ChainStateReadExt: StateRead {
    async fn get_chain_id(&self) -> Result<String>;
    async fn get_revision_number(&self) -> Result<u64>;
    async fn get_block_height(&self) -> Result<u64>;
    async fn get_block_timestamp(&self) -> Result<tendermint::Time>;
}
