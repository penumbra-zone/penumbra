use async_trait::async_trait;
use cnidarium::StateRead;

#[async_trait]
pub trait HostInterface {
    async fn get_chain_id<S: StateRead>(state: S) -> anyhow::Result<String>;
    async fn get_revision_number<S: StateRead>(state: S) -> anyhow::Result<u64>;
    async fn get_block_height<S: StateRead>(state: S) -> anyhow::Result<u64>;
    async fn get_block_timestamp<S: StateRead>(state: S) -> anyhow::Result<tendermint::Time>;
}
