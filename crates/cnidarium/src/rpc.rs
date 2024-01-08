// Autogen code isn't clippy clean:
#[allow(clippy::unwrap_used)]
pub mod proto {
    pub mod v1alpha1 {
        include!("gen/penumbra.cnidarium.v1alpha1.rs");
        include!("gen/penumbra.cnidarium.v1alpha1.serde.rs");
    }

    // https://github.com/penumbra-zone/penumbra/issues/3038#issuecomment-1722534133
    pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("gen/proto_descriptor.bin.no_lfs");
}

pub struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}
use std::pin::Pin;

use crate::read::StateRead;
use crate::rpc::proto::v1alpha1::{
    key_value_response::Value, query_service_server::QueryService, watch_response as wr,
    KeyValueRequest, KeyValueResponse, PrefixValueRequest, PrefixValueResponse, WatchRequest,
    WatchResponse,
};
use futures::{StreamExt, TryStreamExt};
use tokio_stream::wrappers::ReceiverStream;
use tonic::Status;
use tracing::instrument;

use crate::Storage;

#[tonic::async_trait]
impl QueryService for Server {
    #[instrument(skip(self, request))]
    async fn key_value(
        &self,
        request: tonic::Request<KeyValueRequest>,
    ) -> Result<tonic::Response<KeyValueResponse>, Status> {
        let state = self.storage.latest_snapshot();
        // We map the error here to avoid including `tonic` as a dependency
        // in the `chain` crate, to support its compilation to wasm.
        let request = request.into_inner();
        tracing::debug!(?request, "processing key_value request");

        if request.key.is_empty() {
            return Err(Status::invalid_argument("key is empty"));
        }

        // TODO(erwan): Don't generate the proof if the request doesn't ask for it. Tracked in #2647.
        let (some_value, proof) = state
            .get_with_proof(request.key.into_bytes())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(KeyValueResponse {
            value: some_value.map(|value| Value { value }),
            proof: if request.proof {
                Some(ibc_proto::ibc::core::commitment::v1::MerkleProof {
                    proofs: proof
                        .proofs
                        .into_iter()
                        .map(|p| {
                            let mut encoded = Vec::new();
                            prost::Message::encode(&p, &mut encoded).expect("able to encode proof");
                            prost::Message::decode(&*encoded).expect("able to decode proof")
                        })
                        .collect(),
                })
            } else {
                None
            },
        }))
    }

    type PrefixValueStream =
        Pin<Box<dyn futures::Stream<Item = Result<PrefixValueResponse, tonic::Status>> + Send>>;

    #[instrument(skip(self, request))]
    async fn prefix_value(
        &self,
        request: tonic::Request<PrefixValueRequest>,
    ) -> Result<tonic::Response<Self::PrefixValueStream>, Status> {
        let state = self.storage.latest_snapshot();
        let request = request.into_inner();
        tracing::debug!(?request);

        if request.prefix.is_empty() {
            return Err(Status::invalid_argument("prefix is empty"));
        }

        Ok(tonic::Response::new(
            state
                .prefix_raw(&request.prefix)
                .map_ok(|i: (String, Vec<u8>)| {
                    let (key, value) = i;
                    PrefixValueResponse { key, value }
                })
                .map_err(|e: anyhow::Error| {
                    tonic::Status::unavailable(format!(
                        "error getting prefix value from storage: {e}"
                    ))
                })
                .boxed(),
        ))
    }

    type WatchStream = ReceiverStream<Result<WatchResponse, tonic::Status>>;

    #[instrument(skip(self, request))]
    async fn watch(
        &self,
        request: tonic::Request<WatchRequest>,
    ) -> Result<tonic::Response<Self::WatchStream>, Status> {
        let request = request.into_inner();
        tracing::debug!(?request);

        const MAX_REGEX_LEN: usize = 1024;

        let key_regex = regex::RegexBuilder::new(&request.key_regex)
            .size_limit(MAX_REGEX_LEN)
            .build()
            .map_err(|e| Status::invalid_argument(format!("invalid key_regex: {}", e)))?;

        // Use the `bytes` regex to allow matching byte strings.
        let nv_key_regex = regex::bytes::RegexBuilder::new(&request.key_regex)
            .size_limit(MAX_REGEX_LEN)
            .unicode(false)
            .build()
            .map_err(|e| Status::invalid_argument(format!("invalid nv_key_regex: {}", e)))?;

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<WatchResponse, tonic::Status>>(10);

        tokio::spawn(watch_changes(
            self.storage.clone(),
            key_regex,
            nv_key_regex,
            tx,
        ));

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}

async fn watch_changes(
    storage: Storage,
    key_regex: regex::Regex,
    nv_key_regex: regex::bytes::Regex,
    tx: tokio::sync::mpsc::Sender<Result<WatchResponse, tonic::Status>>,
) -> anyhow::Result<()> {
    let mut changes_rx = storage.subscribe_changes();
    loop {
        // Wait for a new set of changes, reporting an error if we don't get one.
        if let Err(e) = changes_rx.changed().await {
            tx.send(Err(tonic::Status::internal(e.to_string()))).await?;
        }
        let (version, changes) = changes_rx.borrow_and_update().clone();

        for (key, value) in changes.unwritten_changes().iter() {
            if key_regex.is_match(key) {
                tx.send(Ok(WatchResponse {
                    version,
                    entry: Some(wr::Entry::Kv(wr::KeyValue {
                        key: key.clone(),
                        value: value.as_ref().cloned().unwrap_or_default(),
                        deleted: value.is_none(),
                    })),
                }))
                .await?;
            }
        }

        for (key, value) in changes.nonverifiable_changes().iter() {
            if nv_key_regex.is_match(key) {
                tx.send(Ok(WatchResponse {
                    version,
                    entry: Some(wr::Entry::NvKv(wr::NvKeyValue {
                        key: key.clone(),
                        value: value.as_ref().cloned().unwrap_or_default(),
                        deleted: value.is_none(),
                    })),
                }))
                .await?;
            }
        }
    }
}
