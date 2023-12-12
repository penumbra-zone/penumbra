use crate::{state_key, CompactBlock};
use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use futures::Stream;
use futures::StreamExt;
use penumbra_proto::DomainType;
use std::pin::Pin;

#[async_trait]
pub trait StateReadExt: StateRead {
    /// Returns a stream of [`CompactBlock`]s starting from `start_height`.
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

#[async_trait]
pub trait StateWriteExt: StateWrite {
    fn set_compact_block(&mut self, compact_block: CompactBlock) {
        let height = compact_block.height;
        self.nonverifiable_put_raw(
            state_key::compact_block(height).into_bytes(),
            compact_block.encode_to_vec(),
        );
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
