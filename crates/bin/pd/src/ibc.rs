use anyhow::Result;
use cnidarium::StateRead;
use cnidarium::Storage;
use cnidarium_component::ChainStateReadExt;
use ibc_types::core::commitment::MerkleProof;
// implemented by [`SnapshotWrapper`]
use penumbra_chain::component::StateReadExt as _;

#[derive(wrapper_derive::StateRead, wrapper_derive::ChainStateReadExt, Clone)]
pub struct SnapshotWrapper<S: StateRead>(S);

#[async_trait::async_trait]
impl penumbra_ibc::component::rpc::Snapshot for SnapshotWrapper<cnidarium::Snapshot> {
    fn version(&self) -> u64 {
        self.0.version()
    }

    async fn get_with_proof(&self, key: Vec<u8>) -> Result<(Option<Vec<u8>>, MerkleProof)> {
        self.0.get_with_proof(key).await
    }
}

#[derive(Clone)]
pub struct StorageWrapper(pub Storage);

impl penumbra_ibc::component::rpc::Storage<SnapshotWrapper<cnidarium::Snapshot>>
    for StorageWrapper
{
    fn latest_snapshot(&self) -> SnapshotWrapper<cnidarium::Snapshot> {
        SnapshotWrapper(self.0.latest_snapshot())
    }
}
