use crate::state_key;
use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateRead;
use futures::Stream;
use futures::StreamExt;
use penumbra_sdk_proto::{penumbra::core::component::compact_block::v1::CompactBlock, Message};
use std::pin::Pin;

#[async_trait]
pub trait StateReadExt: StateRead {
    /// Returns a stream of [`CompactBlock`]s starting from `start_height`.
    ///
    /// Note: this method returns the proto type from `penumbra_sdk_proto`, rather
    /// than deserializing into the domain type, because the primary use is in
    /// serving RPC requests, where the proto type will be re-serialized and
    /// sent to clients.
    fn stream_compact_block(
        &self,
        start_height: u64,
    ) -> Pin<Box<dyn Stream<Item = Result<CompactBlock>> + Send + 'static>> {
        self.nonverifiable_range_raw(
            Some(state_key::prefix().as_bytes()),
            state_key::height(start_height).as_bytes().to_vec()..,
        )
        .expect("valid range is provided")
        .map(|result| {
            result.and_then(|(_, v)| {
                CompactBlock::decode(&mut v.as_slice())
                    .map_err(Error::from)
                    .context("failed to decode compact block")
            })
        })
        .boxed()
    }

    /// Returns a single [`CompactBlock`] at the given `height`.
    ///
    /// Note: this method returns the proto type from `penumbra_sdk_proto`, rather
    /// than deserializing into the domain type, because the primary use is in
    /// serving RPC requests, where the proto type will be re-serialized and
    /// sent to clients.
    async fn compact_block(&self, height: u64) -> Result<Option<CompactBlock>> {
        Ok(self
            .nonverifiable_get_raw(state_key::compact_block(height).as_bytes())
            .await?
            .map(|bytes| {
                CompactBlock::decode(&mut bytes.as_slice()).expect("failed to decode compact block")
            }))
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
