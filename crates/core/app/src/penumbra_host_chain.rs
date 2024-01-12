use std::str::FromStr;

use anyhow::Context;
use async_trait::async_trait;
use ibc_types::core::connection::ChainId;
use penumbra_chain::{component::StateReadExt, state_key};
use penumbra_ibc::component::HostInterface;
use penumbra_proto::StateReadProto;
use tendermint::Time;

// ibc-async HostChainInterface for the Penumbra chain
#[derive(Clone)]
pub struct PenumbraHost {}

#[async_trait]
impl HostInterface for PenumbraHost {
    async fn get_chain_id<S: cnidarium::StateRead>(state: S) -> anyhow::Result<String> {
        // this might be a bit wasteful -- does it matter?  who knows, at this
        // point. but having it be a separate method means we can do a narrower
        // load later if we want
        state.get_chain_params().await.map(|params| params.chain_id)
    }

    async fn get_revision_number<S: cnidarium::StateRead>(state: S) -> anyhow::Result<u64> {
        let cid_str = state.get_chain_id().await?;

        Ok(ChainId::from_string(&cid_str).version())
    }

    async fn get_block_height<S: cnidarium::StateRead>(state: S) -> anyhow::Result<u64> {
        let height: u64 = state
            .get_proto(state_key::block_height())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Missing block_height"))?;

        Ok(height)
    }

    async fn get_block_timestamp<S: cnidarium::StateRead>(
        state: S,
    ) -> anyhow::Result<tendermint::Time> {
        let timestamp_string: String = state
            .get_proto(state_key::block_timestamp())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Missing block_timestamp"))?;

        Ok(Time::from_str(&timestamp_string)
            .context("block_timestamp was an invalid RFC3339 time string")?)
    }
}
