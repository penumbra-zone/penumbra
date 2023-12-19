use cnidarium::Storage;
use penumbra_proto::core::component::sct::v1alpha1::query_service_server::QueryService;

// TODO: Hide this and only expose a Router?
pub struct Server {
    _storage: Storage,
}

impl Server {
    pub fn new(_storage: Storage) -> Self {
        Self { _storage }
    }
}

#[tonic::async_trait]
impl QueryService for Server {}
