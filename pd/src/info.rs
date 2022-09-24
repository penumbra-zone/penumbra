use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
use penumbra_storage::{get_with_proof, AppHash, State, Storage};
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
            .map(|rh| AppHash::from(rh).0)
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
            "state/key" => {
                let height: u64 = query.height.into();
                let key = hex::decode(&query.data).unwrap_or_else(|_| query.data.to_vec());
                let store = jmt::JellyfishMerkleTree::new(&self.storage);

                let (value, proof) = get_with_proof(&store, key, height).await?;

                Ok(abci::response::Query {
                    code: 0,
                    key: query.data,
                    log: "".to_string(),
                    value: value.into(),
                    proof: Some(proof),
                    height: height.try_into().unwrap(),
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
                InfoRequest::SetOption(_) => todo!(),
            }
        }
        .instrument(span)
        .boxed()
    }
}
