use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
use penumbra_chain::View as _;
use penumbra_proto::{client::specific::KeyValueResponse, Message};
use penumbra_storage::{State, Storage};
use tendermint::{
    abci::{self, response::Echo, InfoRequest, InfoResponse},
    block,
};
use tokio::sync::watch;
use tower_abci::BoxError;
use tracing::Instrument;

use crate::RequestExt;

mod oblivious;
mod specific;

const ABCI_INFO_VERSION: &str = env!("VERGEN_GIT_SEMVER");

#[derive(Clone, Debug)]
pub struct Info {
    storage: Storage,
    height_rx: watch::Receiver<block::Height>,
}

impl Info {
    pub fn new(storage: Storage, height_rx: watch::Receiver<block::Height>) -> Self {
        Self { storage, height_rx }
    }

    async fn state_tonic(&self) -> Result<State, tonic::Status> {
        self.storage.state_tonic().await
    }

    async fn info(&self, info: abci::request::Info) -> Result<abci::response::Info, anyhow::Error> {
        tracing::info!(?info);

        let last_block_height = self.storage.latest_version().await?.unwrap_or(0);
        let last_block_app_hash = jmt::JellyfishMerkleTree::new(&self.storage)
            .get_root_hash_option(last_block_height)
            .await?
            .map(|rh| rh.0)
            .unwrap_or([0u8; 32])
            .to_vec()
            .into();

        Ok(abci::response::Info {
            data: "penumbra".to_string(),
            version: ABCI_INFO_VERSION.to_string(),
            app_version: 1,
            last_block_height: last_block_height.try_into().unwrap(),
            last_block_app_hash,
        })
    }

    async fn query(
        &self,
        query: abci::request::Query,
    ) -> Result<abci::response::Query, anyhow::Error> {
        tracing::info!(?query);

        match query.path.as_str() {
            "/jmt/key" => {
                let state = self.state_tonic().await?;
                let (value, proof) = state
                    .read()
                    .await
                    .get_with_proof(query.data.to_vec())
                    .await
                    .map_err(|e| tonic::Status::internal(e.to_string()))?;

                let commitment_proof = ics23::CommitmentProof {
                    proof: Some(ics23::commitment_proof::Proof::Exist(proof)),
                };

                let kvr = KeyValueResponse {
                    value,
                    proof: Some(commitment_proof),
                };

                let height = state.get_block_height().await?;

                Ok(abci::response::Query {
                    code: 0,
                    key: query.data,
                    log: "".to_string(),
                    value: kvr.encode_to_vec().into(),
                    // NOTE: the ABCI query proof is not the same as ICS-23 proofs.
                    proof: None,
                    height: height.try_into()?,
                    codespace: "".to_string(),
                    info: "".to_string(),
                    index: 0, // TODO
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

impl tower::Service<InfoRequest> for Info {
    type Response = InfoResponse;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<InfoResponse, BoxError>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // alternatively: poll for free db connections?
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
                        code: 1,
                        log: e.to_string(),
                        ..Default::default()
                    })),
                },
                InfoRequest::Echo(echo) => Ok(InfoResponse::Echo(Echo {
                    message: echo.message,
                })),
            }
        }
        .instrument(span)
        .boxed()
    }
}
