//! Logic for enabling `pd` to interact with chain state.
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
use penumbra_chain::AppHashRead;
use penumbra_storage::Storage;
use tendermint::abci::{self, response::Echo, InfoRequest, InfoResponse};
use tower_abci::BoxError;
use tracing::Instrument;

use penumbra_tower_trace::RequestExt;

mod oblivious;
mod specific;

const ABCI_INFO_VERSION: &str = env!("VERGEN_GIT_SEMVER");
const APP_VERSION: u64 = 1;

/// Implements service traits for Tonic gRPC services.
///
/// The fields of this struct are the configuration and data
/// necessary to the gRPC services.
#[derive(Clone, Debug)]
pub struct Info {
    /// Storage interface for retrieving chain state.
    storage: Storage,
    // height_rx: watch::Receiver<block::Height>,
}

impl Info {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    async fn info(&self, info: abci::request::Info) -> Result<abci::response::Info, anyhow::Error> {
        let state = self.storage.latest_snapshot();
        tracing::info!(?info, version = ?state.version());

        let last_block_height = match state.version() {
            // When the state is uninitialized, state.version() will return -1 (u64::MAX),
            // which could confuse Tendermint, so special-case this value to 0.
            u64::MAX => 0,
            v => v,
        }
        .try_into()
        .unwrap();

        let last_block_app_hash = state.app_hash().await?.0.to_vec().try_into()?;

        Ok(abci::response::Info {
            data: "penumbra".to_string(),
            version: ABCI_INFO_VERSION.to_string(),
            app_version: APP_VERSION,
            last_block_height,
            last_block_app_hash,
        })
    }

    async fn query(
        &self,
        query: abci::request::Query,
    ) -> Result<abci::response::Query, anyhow::Error> {
        tracing::info!(?query);

        match query.path.as_str() {
            "state/key" => {
                let (snapshot, height) = match u64::from(query.height) {
                    // ABCI docs say:
                    //
                    // The default `0` returns data for the latest committed block. Note that
                    // this is the height of the block containing the application's Merkle root
                    // hash, which represents the state as it was after committing the block at
                    // `height - 1`.
                    //
                    // Try to do this behavior by querying at height = latest-1.
                    0 => {
                        let height = self.storage.latest_snapshot().version() - 1;
                        let snapshot = self
                            .storage
                            .snapshot(height)
                            .ok_or_else(|| anyhow::anyhow!("no snapshot of height {height}"))?;

                        (snapshot, height.try_into().unwrap())
                    }
                    height => {
                        let snapshot = self
                            .storage
                            .snapshot(height)
                            .ok_or_else(|| anyhow::anyhow!("no snapshot of height {height}"))?;
                        (snapshot, query.height)
                    }
                };

                let key = hex::decode(&query.data).unwrap_or_else(|_| query.data.to_vec());

                let (value, proof_ops) = snapshot.get_with_proof_to_apphash_tm(key).await?;

                Ok(abci::response::Query {
                    code: 0.into(),
                    key: query.data,
                    log: "".to_string(),
                    value: value.into(),
                    proof: Some(proof_ops),
                    height,
                    codespace: "".to_string(),
                    info: "".to_string(),
                    index: 0,
                })
            }
            _ => {
                // TODO: handle unrecognized path
                Ok(Default::default())
            }
        }
        // TODO: implement (#22)
    }
}

impl tower_service::Service<InfoRequest> for Info {
    type Response = InfoResponse;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<InfoResponse, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: InfoRequest) -> Self::Future {
        let span = req.create_span();
        let self2 = self.clone();

        async move {
            match req {
                InfoRequest::Info(info) => self2
                    .info(info)
                    .await
                    .map(InfoResponse::Info)
                    .map_err(Into::into),
                InfoRequest::Query(query) => match self2.query(query).await {
                    Ok(rsp) => Ok(InfoResponse::Query(rsp)),
                    Err(e) => Ok(InfoResponse::Query(abci::response::Query {
                        code: 1.into(),
                        log: e.to_string(),
                        ..Default::default()
                    })),
                },
                InfoRequest::Echo(echo) => Ok(InfoResponse::Echo(Echo {
                    message: echo.message,
                })),
                InfoRequest::SetOption(_) => todo!(),
            }
        }
        .instrument(span)
        .boxed()
    }
}
