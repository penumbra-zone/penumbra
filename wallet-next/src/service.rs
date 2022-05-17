use std::{
    pin::Pin,
    sync::{Arc, Mutex},
};

use penumbra_crypto::keys::FullViewingKeyHash;
use penumbra_proto::{
    client::oblivious::oblivious_query_client::ObliviousQueryClient,
    crypto as pbc,
    wallet::{self as pb, wallet_protocol_server::WalletProtocol, StatusResponse},
};
use tokio::sync::mpsc;
use tonic::{async_trait, transport::Channel};

use crate::{Storage, Worker};

/// A service that synchronizes private chain state and responds to queries
/// about it.
///
/// The [`WalletService`] implements the Tonic-derived [`WalletProtocol`] trait,
/// so it can be used as a gRPC server, or called directly.  It spawns a task
/// internally that performs synchronization and scanning.  The
/// [`WalletService`] can be cloned; each clone will read from the same shared
/// state, but there will only be a single scanning task.

pub struct WalletService {
    storage: Storage,
    // A shared error slot for errors bubbled up by the worker. This is a regular Mutex
    // rather than a Tokio Mutex because it should be uncontended.
    error_slot: Arc<Mutex<Option<anyhow::Error>>>,
    // When all the senders have dropped, the worker will stop.
    worker_shutdown_tx: mpsc::Sender<()>,
    fvk_hash: FullViewingKeyHash,
}

impl WalletService {
    /// Constructs a new [`WalletService`], spawning a sync task internally.
    ///
    /// The sync task uses the provided `client` to sync with the chain.
    ///
    /// To create multiple [`WalletService`]s, clone the [`WalletService`] returned
    /// by this method, rather than calling it multiple times.  That way, each clone
    /// will be backed by the same scanning task, rather than each spawning its own.
    pub async fn new(
        storage: Storage,
        client: ObliviousQueryClient<Channel>,
    ) -> Result<Self, anyhow::Error> {
        // Create a shared error slot
        let error_slot = Arc::new(Mutex::new(None));

        // Create a means of communicating shutdown with the worker task
        let (tx, mut rx) = mpsc::channel(1);

        let worker = Worker::new(storage.clone(), client, error_slot.clone(), rx).await?;

        tokio::spawn(worker.run());

        let fvk = storage.full_viewing_key().await?;
        let fvk_hash = fvk.hash();

        Ok(Self {
            storage,
            fvk_hash,
            error_slot,
            worker_shutdown_tx: tx,
        })
    }

    async fn check_fvk(&self, fvk: Option<&pbc::FullViewingKeyHash>) -> Result<(), tonic::Status> {
        // Takes an Option to avoid making the caller handle missing fields,
        // should error on None or wrong FVK hash
        match fvk {
            Some(fvk) => {
                if fvk != &self.fvk_hash.into() {
                    return Err(tonic::Status::new(
                        tonic::Code::InvalidArgument,
                        "Invalid FVK hash",
                    ));
                }

                Ok(())
            }
            None => Err(tonic::Status::new(
                tonic::Code::InvalidArgument,
                "Missing FVK",
            )),
        }
    }

    async fn check_worker(&self) -> Result<(), tonic::Status> {
        // If the shared error slot is set, then an error has occurred in the worker
        // that we should bubble up.
        if self.error_slot.lock().unwrap().is_some() {
            return Err(tonic::Status::new(
                tonic::Code::Internal,
                format!(
                    "Worker failed: {}",
                    self.error_slot.lock().unwrap().as_ref().unwrap()
                ),
            ));
        }

        // TODO: check whether the worker is still alive, else fail, when we have a way to do that
        // (if the worker is to crash without setting the error_slot, the service should die as well)

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

        let last_sync_height = self
            .storage
            .last_sync_height()
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?
            .unwrap_or(0);

        // TODO: we need to determine how to get the `chain_height` from the full node
        // until we have that, we can't fully implement this.
        Ok(tonic::Response::new(StatusResponse {
            synchronized: true,
            chain_height: last_sync_height,
            sync_height: last_sync_height,
        }))
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
