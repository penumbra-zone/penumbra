use anyhow::Result;
use async_trait::async_trait;
use cnidarium::StateRead;
use cnidarium::StateWrite;
use cnidarium_component::ChainStateReadExt;
use penumbra_chain::component::StateReadExt;
use std::sync::Arc;

pub(crate) struct StateDeltaWrapper<'a, S: StateRead + StateWrite>(pub(crate) &'a mut Arc<S>);

impl<'a, S: StateRead + StateWrite> StateWrite for StateDeltaWrapper<'a, S> {
    fn put_raw(&mut self, key: String, value: Vec<u8>) {
        let state = Arc::get_mut(&mut self.0).expect("state should be unique");
        state.put_raw(key, value)
    }

    fn delete(&mut self, key: String) {
        let state = Arc::get_mut(&mut self.0).expect("state should be unique");
        state.delete(key)
    }

    fn nonverifiable_delete(&mut self, key: Vec<u8>) {
        let state = Arc::get_mut(&mut self.0).expect("state should be unique");
        state.nonverifiable_delete(key)
    }

    fn nonverifiable_put_raw(&mut self, key: Vec<u8>, value: Vec<u8>) {
        let state = Arc::get_mut(&mut self.0).expect("state should be unique");
        state.nonverifiable_put_raw(key, value)
    }

    fn object_put<T: Clone + std::any::Any + Send + Sync>(&mut self, key: &'static str, value: T) {
        let state = Arc::get_mut(&mut self.0).expect("state should be unique");
        state.object_put(key, value)
    }

    fn object_delete(&mut self, key: &'static str) {
        let state = Arc::get_mut(&mut self.0).expect("state should be unique");
        state.object_delete(key)
    }

    fn object_merge(
        &mut self,
        objects: std::collections::BTreeMap<
            &'static str,
            Option<Box<dyn std::any::Any + Send + Sync>>,
        >,
    ) {
        let state = Arc::get_mut(&mut self.0).expect("state should be unique");
        state.object_merge(objects)
    }

    fn record(&mut self, event: tendermint::abci::Event) {
        let state = Arc::get_mut(&mut self.0).expect("state should be unique");
        state.record(event)
    }
}

impl<'a, S: StateRead + StateWrite> StateRead for StateDeltaWrapper<'a, S> {
    type GetRawFut = S::GetRawFut;
    type PrefixRawStream = S::PrefixRawStream;
    type PrefixKeysStream = S::PrefixKeysStream;
    type NonconsensusPrefixRawStream = S::NonconsensusPrefixRawStream;
    type NonconsensusRangeRawStream = S::NonconsensusRangeRawStream;

    fn get_raw(&self, key: &str) -> Self::GetRawFut {
        self.0.get_raw(key)
    }

    fn prefix_raw(&self, prefix: &str) -> S::PrefixRawStream {
        self.0.prefix_raw(prefix)
    }

    fn prefix_keys(&self, prefix: &str) -> S::PrefixKeysStream {
        self.0.prefix_keys(prefix)
    }

    fn nonverifiable_prefix_raw(&self, prefix: &[u8]) -> S::NonconsensusPrefixRawStream {
        self.0.nonverifiable_prefix_raw(prefix)
    }

    fn nonverifiable_range_raw(
        &self,
        prefix: Option<&[u8]>,
        range: impl std::ops::RangeBounds<Vec<u8>>,
    ) -> anyhow::Result<Self::NonconsensusRangeRawStream> {
        self.0.nonverifiable_range_raw(prefix, range)
    }

    fn nonverifiable_get_raw(&self, key: &[u8]) -> Self::GetRawFut {
        self.0.nonverifiable_get_raw(key)
    }

    fn object_get<T: std::any::Any + Send + Sync + Clone>(&self, key: &'static str) -> Option<T> {
        self.0.object_get(key)
    }

    fn object_type(&self, key: &'static str) -> Option<std::any::TypeId> {
        self.0.object_type(key)
    }
}

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
