use anyhow::Result;
use async_trait::async_trait;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

use crate::{state_key, CompactBlock};

#[async_trait]
pub trait StateReadExt: StateRead {
    // formerly compact block methods

    async fn compact_block(&self, height: u64) -> Result<Option<CompactBlock>> {
        self.get(&state_key::compact_block(height)).await
    }

    fn stub_compact_block(&self) -> CompactBlock {
        self.object_get(state_key::stub_compact_block())
            .unwrap_or_default()
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}

#[async_trait]
pub trait StateWriteExt: StateWrite {
    fn stub_put_compact_block(&mut self, compact_block: CompactBlock) {
        self.object_put(state_key::stub_compact_block(), compact_block);
    }

    fn set_compact_block(&mut self, compact_block: CompactBlock) {
        let height = compact_block.height;
        self.put(state_key::compact_block(height), compact_block);
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
