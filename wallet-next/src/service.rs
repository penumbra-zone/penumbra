use std::pin::Pin;

use penumbra_proto::{
    crypto as pbc,
    wallet::{self as pb, wallet_protocol_server::WalletProtocol},
};
use tonic::async_trait;

use crate::{Storage, Worker};

/// A service that synchronizes private chain state and responds to queries
/// about it.
///
/// The [`WalletService`] implements the Tonic-derived [`WalletProtocol`] trait,
/// so it can be used as a gRPC server, or called directly.  It spawns a task
/// internally that performs synchronization and scanning.  The
/// [`WalletService`] can be cloned; each clone will read from the same shared
/// state, but there will only be a single scanning task.
#[derive(Clone)]
pub struct WalletService {
    storage: Storage,
    // TODO: add a way for the WalletService to poll the state of its worker task, to determine if it failed
    // this probably looks like a handle for the task, wrapped in an arc'd mutex or something
    // error_slot: Arc<Mutex<Option<anyhow::Error>>>,
    // TODO: add a way for the WalletService to signal the worker that it should shut down
    // this probably looks like an Arc<oneshot::Sender<()>> or something,
    // where the receiver is held by the worker and the worker checks if it's closed (=> all sender handles dropped)
}

impl WalletService {
    /// Constructs a new [`WalletService`], spawning a sync task internally.
    ///
    /// To create multiple [`WalletService`]s, clone the [`WalletService`] returned
    /// by this method, rather than calling it multiple times.  That way, each clone
    /// will be backed by the same scanning task, rather than each spawning its own.
    pub fn new(storage: Storage) -> Self {
        let worker = Worker::new(storage.clone());
        tokio::spawn(worker.run());

        Self { storage }
    }

    async fn check_fvk(&self, fvk: Option<&pbc::FullViewingKeyHash>) -> Result<(), tonic::Status> {
        // TODO: check the fvk against the Storage
        // Takes an Option to avoid making the caller handle missing fields,
        // should error on None or wrong FVK hash
        Ok(())
    }

    async fn check_worker(&self) -> Result<(), tonic::Status> {
        // TODO: check whether the worker is still alive, else fail, when we have a way to do that
        Ok(())
    }
}

#[async_trait]
impl WalletProtocol for WalletService {
    type NotesStream =
        Pin<Box<dyn futures::Stream<Item = Result<pb::NoteRecord, tonic::Status>> + Send>>;

    async fn status(
        &self,
        request: tonic::Request<pb::StatusRequest>,
    ) -> Result<tonic::Response<pb::StatusResponse>, tonic::Status> {
        self.check_worker().await?;
        self.check_fvk(request.get_ref().fvk_hash.as_ref()).await?;

        todo!()
    }

    async fn notes(
        &self,
        request: tonic::Request<pb::NotesRequest>,
    ) -> Result<tonic::Response<Self::NotesStream>, tonic::Status> {
        self.check_worker().await?;
        self.check_fvk(request.get_ref().fvk_hash.as_ref()).await?;

        todo!()
    }

    async fn auth_paths(
        &self,
        request: tonic::Request<pb::AuthPathsRequest>,
    ) -> Result<tonic::Response<pb::AuthPathsResponse>, tonic::Status> {
        self.check_worker().await?;
        self.check_fvk(request.get_ref().fvk_hash.as_ref()).await?;

        todo!()
    }
}
