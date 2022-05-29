use std::{
    pin::Pin,
    sync::{Arc, Mutex},
};

use anyhow::anyhow;
use async_stream::try_stream;
use futures::stream::{StreamExt, TryStreamExt};
use penumbra_crypto::{
    asset,
    keys::{DiversifierIndex, FullViewingKeyHash},
};
use penumbra_proto::{
    chain as pbp,
    client::oblivious::oblivious_query_client::ObliviousQueryClient,
    crypto as pbc, transaction as pbt,
    view::{self as pb, view_protocol_server::ViewProtocol, StatusResponse},
};
use penumbra_tct::{Commitment, Proof};
use penumbra_transaction::WitnessData;
use tokio::sync::{watch, RwLock};
use tokio_stream::wrappers::WatchStream;
use tonic::{async_trait, transport::Channel};
use tracing::instrument;

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
    fvk_hash: FullViewingKeyHash,
    // A copy of the NCT used by the worker task.
    note_commitment_tree: Arc<RwLock<penumbra_tct::Tree>>,
    // The address of the pd+tendermint node.
    node: String,
    // The port to use to speak to tendermint's RPC server.
    tendermint_port: u16,
    /// Used to watch for changes to the sync height.
    sync_height_rx: watch::Receiver<u64>,
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
        node: String,
        tendermint_port: u16,
    ) -> Result<Self, anyhow::Error> {
        // Create a shared error slot
        let error_slot = Arc::new(Mutex::new(None));

        // Create a channel for the worker to notify us of sync height changes.
        let (sync_height_tx, sync_height_rx) =
            watch::channel(storage.last_sync_height().await?.unwrap_or(0));

        let (worker, nct) =
            Worker::new(storage.clone(), client, error_slot.clone(), sync_height_tx).await?;

        tokio::spawn(worker.run());

        let fvk = storage.full_viewing_key().await?;
        let fvk_hash = fvk.hash();

        Ok(Self {
            storage,
            fvk_hash,
            error_slot,
            sync_height_rx,
            note_commitment_tree: nct,
            node,
            tendermint_port,
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

    /// Return the latest block height known by the fullnode or its peers, as
    /// well as whether the fullnode is caught up with that height.
    #[instrument(skip(self))]
    pub async fn latest_known_block_height(&self) -> Result<(u64, bool), anyhow::Error> {
        let client = reqwest::Client::new();

        let rsp: serde_json::Value = client
            .get(format!(
                r#"http://{}:{}/status"#,
                self.node, self.tendermint_port
            ))
            .send()
            .await?
            .json()
            .await?;

        tracing::debug!("{}", rsp);

        let sync_info = rsp
            .get("result")
            .and_then(|r| r.get("sync_info"))
            .ok_or_else(|| anyhow::anyhow!("could not parse sync_info in JSON response"))?;

        let latest_block_height = sync_info
            .get("latest_block_height")
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow::anyhow!("could not parse latest_block_height in JSON response"))?
            .parse()?;

        let max_peer_block_height = sync_info
            .get("max_peer_block_height")
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!("could not parse max_peer_block_height in JSON response")
            })?
            .parse()?;

        let node_catching_up = sync_info
            .get("catching_up")
            .and_then(|c| c.as_bool())
            .ok_or_else(|| anyhow::anyhow!("could not parse catching_up in JSON response"))?;

        let latest_known_block_height = std::cmp::max(latest_block_height, max_peer_block_height);

        tracing::debug!(
            ?latest_block_height,
            ?max_peer_block_height,
            ?node_catching_up,
            ?latest_known_block_height
        );

        Ok((latest_known_block_height, node_catching_up))
    }

    #[instrument(skip(self))]
    pub async fn status(&self) -> Result<StatusResponse, anyhow::Error> {
        let sync_height = self.storage.last_sync_height().await?.unwrap_or(0);

        let (latest_known_block_height, node_catching_up) =
            self.latest_known_block_height().await?;

        let height_diff = latest_known_block_height
            .checked_sub(sync_height)
            .ok_or_else(|| anyhow!("sync height ahead of node height"))?;

        let catching_up = match (node_catching_up, height_diff) {
            // We're synced to the same height as the node
            (false, 0) => false,
            // We're one block behind, and will learn about it soon, close enough
            (false, 1) => false,
            // We're behind the node
            (false, _) => true,
            // The node is behind the network
            (true, _) => true,
        };

        Ok(StatusResponse {
            sync_height,
            catching_up,
        })
    }
}

#[async_trait]
impl ViewProtocol for ViewService {
    type NotesStream =
        Pin<Box<dyn futures::Stream<Item = Result<pb::NoteRecord, tonic::Status>> + Send>>;
    type AssetsStream =
        Pin<Box<dyn futures::Stream<Item = Result<pbc::Asset, tonic::Status>> + Send>>;
    type StatusStreamStream = Pin<
        Box<dyn futures::Stream<Item = Result<pb::StatusStreamResponse, tonic::Status>> + Send>,
    >;

    async fn status(
        &self,
        request: tonic::Request<pb::StatusRequest>,
    ) -> Result<tonic::Response<pb::StatusResponse>, tonic::Status> {
        self.check_worker().await?;
        self.check_fvk(request.get_ref().fvk_hash.as_ref()).await?;

        Ok(tonic::Response::new(self.status().await.map_err(|_| {
            tonic::Status::unknown("unknown error getting status response")
        })?))
    }

    async fn status_stream(
        &self,
        request: tonic::Request<pb::StatusStreamRequest>,
    ) -> Result<tonic::Response<Self::StatusStreamStream>, tonic::Status> {
        self.check_worker().await?;
        self.check_fvk(request.get_ref().fvk_hash.as_ref()).await?;

        let (latest_known_block_height, _) =
            self.latest_known_block_height().await.map_err(|e| {
                tonic::Status::unknown(format!(
                    "unable to fetch latest known block height from fullnode: {}",
                    e
                ))
            })?;

        // Create a stream of sync height updates from our worker, and send them to the client
        // until we've reached the latest known block height at the time the request was made.
        let mut sync_height_stream = WatchStream::new(self.sync_height_rx.clone());
        let stream = try_stream! {
            while let Some(sync_height) = sync_height_stream.next().await {
                yield pb::StatusStreamResponse {
                    latest_known_block_height,
                    sync_height,
                };
                if sync_height == latest_known_block_height {
                    break;
                }
            }
        };

        Ok(tonic::Response::new(stream.boxed()))
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

    async fn assets(
        &self,
        _request: tonic::Request<pb::AssetRequest>,
    ) -> Result<tonic::Response<Self::AssetsStream>, tonic::Status> {
        self.check_worker().await?;

        // Fetch assets from storage.
        let assets = self
            .storage
            .assets()
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;

        let stream = try_stream! {
            for asset in assets {
                yield asset.into()
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

    async fn chain_params(
        &self,
        _request: tonic::Request<pb::ChainParamsRequest>,
    ) -> Result<tonic::Response<pbp::ChainParams>, tonic::Status> {
        self.check_worker().await?;

        let params = self
            .storage
            .chain_params()
            .await
            .map_err(|_| tonic::Status::unavailable("database error"))?;

        Ok(tonic::Response::new(params.into()))
    }
}
