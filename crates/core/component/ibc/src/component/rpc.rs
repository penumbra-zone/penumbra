use cnidarium::Storage;
use tonic::transport::server::Routes;

mod client_query;
mod connection_query;
mod consensus_query;

// TODO: hide and replace with a routes() constructor that
// bundles up all the internal services
#[derive(Clone)]
pub struct IbcQuery(cnidarium::Storage);

impl IbcQuery {
    pub fn new(storage: cnidarium::Storage) -> Self {
        Self(storage)
    }
}

pub fn routes(_storage: Storage) -> Routes {
    unimplemented!("functionality we need is only in tonic 0.10")
}
