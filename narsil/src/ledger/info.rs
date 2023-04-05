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

// const ABCI_INFO_VERSION: &str = env!("VERGEN_GIT_SEMVER");
const ABCI_INFO_VERSION: &str = "wut";
const APP_VERSION: u64 = 1;

/// Implements service traits for Tonic gRPC services.
///
/// The fields of this struct are the configuration and data
/// necessary to the gRPC services.
#[derive(Clone, Debug)]
pub struct Info {
    /// Storage interface for retrieving chain state.
    storage: Storage,
}

impl Info {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub async fn info(
        &self,
        info: abci::request::Info,
    ) -> Result<abci::response::Info, anyhow::Error> {
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
                InfoRequest::Query(_query) => todo!(),
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
