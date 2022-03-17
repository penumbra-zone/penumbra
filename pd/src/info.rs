use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures::FutureExt;
use tendermint::abci::{self, response::Echo, InfoRequest, InfoResponse};
use tower_abci::BoxError;
use tracing::Instrument;

use crate::{db::schema, state, RequestExt, Storage};

const ABCI_INFO_VERSION: &str = env!("VERGEN_GIT_SEMVER");

#[derive(Clone, Debug)]
pub struct Info {
    state: state::Reader,
    storage: Storage,
}

impl Info {
    pub fn new(state: state::Reader, storage: Storage) -> Self {
        Self { state, storage }
    }

    async fn info(&self, info: abci::request::Info) -> Result<abci::response::Info, anyhow::Error> {
        tracing::info!(?info);

        let (last_block_height, last_block_app_hash) = match self.state.latest_block_info().await? {
            Some(schema::BlocksRow {
                height, app_hash, ..
            }) => (height.try_into().unwrap(), app_hash.into()),
            None => (0u32.into(), vec![0; 32].into()),
        };

        Ok(abci::response::Info {
            data: "penumbra".to_string(),
            version: ABCI_INFO_VERSION.to_string(),
            app_version: 1,
            last_block_height,
            last_block_app_hash,
        })
    }

    async fn query(
        &self,
        query: abci::request::Query,
    ) -> Result<abci::response::Query, anyhow::Error> {
        tracing::warn!(?query, "unhandled query");
        // TODO: implement (#22)
        Ok(Default::default())
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
