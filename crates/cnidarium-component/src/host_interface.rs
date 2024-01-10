use async_trait::async_trait;
use cnidarium::StateRead;

#[async_trait]
pub trait HostInterface: StateRead {
    async fn get_chain_id(&self) -> anyhow::Result<String>;
    async fn get_revision_number(&self) -> anyhow::Result<u64>;
    async fn get_block_height(&self) -> anyhow::Result<u64>;
    async fn get_block_timestamp(&self) -> anyhow::Result<tendermint::Time>;
}
