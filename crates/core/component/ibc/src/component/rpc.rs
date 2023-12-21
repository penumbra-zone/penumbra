use anyhow::Result;
use async_trait::async_trait;
use ibc_types::core::commitment::MerkleProof;
use std::marker::PhantomData;

use cnidarium_component::ChainStateReadExt;
use tonic::transport::server::Routes;

mod client_query;
mod connection_query;
mod consensus_query;

// Implemented by [`cnidarium::Storage`].
// Used as a wrapper so external crates can implemented their own [`ChainStateReadExt`].
pub trait Storage<C>: Send + Sync + 'static {
    fn latest_snapshot(&self) -> C;
}

// Implemented by [`cnidarium::Snapshot`].
#[async_trait]
pub trait Snapshot {
    fn version(&self) -> u64;
    async fn get_with_proof(&self, key: Vec<u8>) -> Result<(Option<Vec<u8>>, MerkleProof)>;
}

// TODO: hide and replace with a routes() constructor that
// bundles up all the internal services
#[derive(Clone)]
pub struct IbcQuery<C, S>(S, PhantomData<C>);

impl<C: ChainStateReadExt + Snapshot, S: Storage<C>> IbcQuery<C, S> {
    pub fn new(storage: S) -> Self {
        Self(storage, PhantomData)
    }
}

pub fn routes(_storage: cnidarium::Storage) -> Routes {
    unimplemented!("functionality we need is only in tonic 0.10")
}
