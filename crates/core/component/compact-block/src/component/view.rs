use anyhow::Result;
use async_trait::async_trait;
use penumbra_proto::DomainType;
use penumbra_storage::{StateRead, StateWrite};

use crate::{state_key, CompactBlock};

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn compact_block(&self, height: u64) -> Result<Option<CompactBlock>> {
        Ok(self
            .nonconsensus_get_raw(&state_key::compact_block(height).as_bytes())
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
        self.nonconsensus_put_raw(
            state_key::compact_block(height).into_bytes(),
            compact_block.encode_to_vec(),
        );
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
