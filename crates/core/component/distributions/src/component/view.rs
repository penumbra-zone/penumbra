use async_trait::async_trait;

use crate::component::state_key;
use anyhow::Result;
use penumbra_num::Amount;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn total_issued(&self) -> Result<Option<u64>> {
        self.get_proto(&state_key::total_issued()).await
    }
}

impl<T: StateRead> StateReadExt for T {}

#[async_trait]

pub trait StateWriteExt: StateWrite + StateReadExt {
    /// Set the total amount of staking tokens issued.
    fn set_total_issued(&mut self, total_issued: u64) {
        let total = Amount::from(total_issued);
        self.put(state_key::total_issued().to_string(), total)
    }
}
impl<T: StateWrite> StateWriteExt for T {}
