use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
//use penumbra_storage::{get_with_proof, AppHash, State, Storage};
use penumbra_chain::AppHashRead;
use penumbra_storage::Storage;
use tendermint::abci::{self, response::Echo, InfoRequest, InfoResponse};
use tower_abci::BoxError;
use tracing::Instrument;

use crate::RequestExt;

mod oblivious;
mod specific;

const ABCI_INFO_VERSION: &str = env!("VERGEN_GIT_SEMVER");
const APP_VERSION: u64 = 1;

#[derive(Clone, Debug)]
pub struct Info {
    storage: Storage,
    // height_rx: watch::Receiver<block::Height>,
}

impl Info {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    async fn info(&self, info: abci::request::Info) -> Result<abci::response::Info, anyhow::Error> {
        let state = self.storage.state();
        tracing::info!(?info, version = ?state.version());

        let last_block_height = match state.version() {
            // When the state is uninitialized, state.version() will return -1 (u64::MAX),
            // which could confuse Tendermint, so special-case this value to 0.
            u64::MAX => 0,
            v => v,
        }
        .try_into()
        .unwrap();

        let last_block_app_hash = state.app_hash().await?.0.to_vec().into();

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
                // TODO: decide how to handle height field
                // - add versioned get_with_proof to Storage ?
                // - add a State cache in Storage ?
                let _height: u64 = query.height.into();
                let key = hex::decode(&query.data).unwrap_or_else(|_| query.data.to_vec());

                let state = self.storage.state();
                let _height = state.version();

                // TODO: align types (check storage/src/app_hash.rs::get_with_proof)
                // where should that logic go?
                let (_value, _proof) = state.get_with_proof(key).await?;
                // let (value, proof) = get_with_proof(&store, key, height).await?;

                /*
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
                 */
                // TODO: restore
                Ok(Default::default())
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
