use cnidarium::Storage;
use tonic::transport::server::Routes;

use super::HostInterface;

mod client_query;
mod connection_query;
mod consensus_query;
mod utils;

use std::marker::PhantomData;

// TODO: hide and replace with a routes() constructor that
// bundles up all the internal services
#[derive(Clone)]
pub struct IbcQuery<HI: HostInterface> {
    storage: cnidarium::Storage,
    _marker: PhantomData<HI>,
}

impl<HI: HostInterface> IbcQuery<HI> {
    pub fn new(storage: cnidarium::Storage) -> Self {
        Self {
            storage,
            _marker: PhantomData,
        }
    }
}

pub fn routes(_storage: Storage) -> Routes {
    unimplemented!("functionality we need is only in tonic 0.10")
}
