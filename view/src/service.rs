use std::{
    pin::Pin,
    sync::{Arc, Mutex},
};

use async_stream::try_stream;
use futures::stream::{StreamExt, TryStreamExt};
use penumbra_crypto::{
    asset,
    keys::{DiversifierIndex, FullViewingKeyHash},
};
use penumbra_proto::{
    client::oblivious::oblivious_query_client::ObliviousQueryClient,
    crypto as pbc, transaction as pbt,
    view::{self as pb, view_protocol_server::ViewProtocol, StatusResponse},
};
use penumbra_tct::{Commitment, Proof};
use penumbra_transaction::WitnessData;
use tokio::sync::{mpsc, RwLock};
use tonic::{async_trait, transport::Channel};

use crate::{Storage, Worker};

/// A service that synchronizes private chain state and responds to queries
/// about it.
///
/// The [`ViewService`] implements the Tonic-derived [`ViewProtocol`] trait,
/// so it can be used as a gRPC server, or called directly.  It spawns a task
/// internally that performs synchronization and scanning.  The
/// [`ViewService`] can be cloned; each clone will read from the same shared
/// state, but there will only be a single scanning task.

pub struct ViewService {
    storage: Storage,
    // A shared error slot for errors bubbled up by the worker. This is a regular Mutex
    // rather than a Tokio Mutex because it should be uncontended.
    error_slot: Arc<Mutex<Option<anyhow::Error>>>,
    // When all the senders have dropped, the worker will stop.
    worker_shutdown_tx: mpsc::Sender<()>,
    fvk_hash: FullViewingKeyHash,
    // A copy of the NCT used by the worker task.
    note_commitment_tree: Arc<RwLock<penumbra_tct::Tree>>,
}

impl ViewService {
    /// Constructs a new [`ViewService`], spawning a sync task internally.
    ///
    /// The sync task uses the provided `client` to sync with the chain.
    ///
    /// To create multiple [`ViewService`]s, clone the [`ViewService`] returned
    /// by this method, rather than calling it multiple times.  That way, each clone
    /// will be backed by the same scanning task, rather than each spawning its own.
    pub async fn new(
        storage: Storage,
        client: ObliviousQueryClient<Channel>,
    ) -> Result<Self, anyhow::Error> {
        // Create a shared error slot
        let error_slot = Arc::new(Mutex::new(None));

        // Create a means of communicating shutdown with the worker task
        let (tx, rx) = mpsc::channel(1);

        let (worker, nct) = Worker::new(storage.clone(), client, error_slot.clone(), rx).await?;

        tokio::spawn(worker.run());

        let fvk = storage.full_viewing_key().await?;
        let fvk_hash = fvk.hash();

        Ok(Self {
            storage,
            fvk_hash,
            error_slot,
            worker_shutdown_tx: tx,
            note_commitment_tree: nct,
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
impl ViewProtocol for ViewService {
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

        let include_spent = request.get_ref().include_spent;
        let asset_id = request
            .get_ref()
            .asset_id
            .to_owned()
            .map(asset::Id::try_from)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|_| tonic::Status::invalid_argument("invalid asset id"))?;
        let diversifier_index = request
            .get_ref()
            .diversifier_index
            .to_owned()
            .map(DiversifierIndex::try_from)
            .map_or(Ok(None), |v| v.map(Some))
            .map_err(|_| tonic::Status::invalid_argument("invalid diversifier index"))?;
        let amount_to_spend = request.get_ref().amount_to_spend;

        let notes = self
            .storage
            .notes(include_spent, asset_id, diversifier_index, amount_to_spend)
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;

        let stream = try_stream! {
            for note in notes {
                yield note.into()
            }
        };

        Ok(tonic::Response::new(
            stream
                .map_err(|_: anyhow::Error| tonic::Status::unavailable("database error"))
                .boxed(),
        ))
    }

    async fn witness(
        &self,
        request: tonic::Request<pb::WitnessRequest>,
    ) -> Result<tonic::Response<pbt::WitnessData>, tonic::Status> {
        self.check_worker().await?;
        self.check_fvk(request.get_ref().fvk_hash.as_ref()).await?;

        // Acquire a read lock for the NCT that will live for the entire request,
        // so that all auth paths are relative to the same NCT root.
        let nct = self.note_commitment_tree.read().await;

        // Read the NCT root
        let anchor = nct.root();

        // Obtain an auth path for each requested note commitment
        let requested_note_commitments = request
            .get_ref()
            .note_commitments
            .iter()
            .map(|nc| Commitment::try_from(nc.clone()))
            .collect::<Result<Vec<Commitment>, _>>()
            .map_err(|_| {
                tonic::Status::new(
                    tonic::Code::InvalidArgument,
                    "Unable to deserialize note commitment",
                )
            })?;
        let auth_paths: Vec<Proof> = requested_note_commitments
            .iter()
            .map(|nc| {
                nct.witness(*nc).ok_or_else(|| {
                    tonic::Status::new(tonic::Code::InvalidArgument, "Note commitment missing")
                })
            })
            .collect::<Result<Vec<Proof>, tonic::Status>>()?;

        // Release the read lock on the NCT
        drop(nct);

        let witness_data = WitnessData {
            anchor,
            note_commitment_proofs: auth_paths,
        };
        Ok(tonic::Response::new(witness_data.into()))
    }
}
