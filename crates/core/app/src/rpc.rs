use std::pin::Pin;

use anyhow::Context;
use async_stream::try_stream;
use futures::{StreamExt as _, TryStreamExt as _};
use penumbra_chain::component::StateReadExt as _;
use penumbra_proto::{
    core::app::v1alpha1::{
        query_service_server::QueryService, AppParametersRequest, AppParametersResponse,
        TransactionsByHeightRequest, TransactionsByHeightResponse,
    },
    StateReadProto as _,
};
use penumbra_storage::{StateRead as _, Storage};
use sha2::Digest;
use tokio::task::JoinSet;
use tonic::Status;
use tracing::instrument;

use crate::app::{state_key, StateReadExt as _};

// TODO: Hide this and only expose a Router?
pub struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl QueryService for Server {
    type TransactionsByHeightStream = Pin<
        Box<dyn futures::Stream<Item = Result<TransactionsByHeightResponse, tonic::Status>> + Send>,
    >;

    #[instrument(skip(self, request))]
    async fn transactions_by_height(
        &self,
        request: tonic::Request<TransactionsByHeightRequest>,
    ) -> Result<tonic::Response<Self::TransactionsByHeightStream>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let request_inner = request.into_inner();
        let block_heights = request_inner.block_heights;

        let s = try_stream! {
            for h in block_heights.iter() {
                let mut transactions = state.transactions_by_height(*h);

                while let Some(r) = transactions.next().await {
                    let r = r.context("error getting prefix value from storage")?;
                    let tx_bytes = r.2;
                    let tx_id = r.1;
                    let transaction = tx_bytes.try_into().context("bad tx bytes in storage")?;
                        yield TransactionsByHeightResponse {
                        block_height: r.0,
                        transaction: Some(transaction),
                        tx_id: tx_id.to_vec()
                    };
                }
            }
        };

        Ok(tonic::Response::new(
            s.map_err(|e: anyhow::Error| {
                tonic::Status::unavailable(format!("error getting transactions: {e}"))
            })
            .boxed(),
            // TODO: how do we instrument a Stream
            //.instrument(Span::current())
        ))
    }

    #[instrument(skip(self, request))]
    async fn app_parameters(
        &self,
        request: tonic::Request<AppParametersRequest>,
    ) -> Result<tonic::Response<AppParametersResponse>, Status> {
        let state = self.storage.latest_snapshot();
        // We map the error here to avoid including `tonic` as a dependency
        // in the `chain` crate, to support its compilation to wasm.
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| {
                tonic::Status::unknown(format!(
                    "failed to validate chain id during app parameters lookup: {e}"
                ))
            })?;

        let app_parameters = state.get_app_params().await.map_err(|e| {
            tonic::Status::unavailable(format!("error getting app parameters: {e}"))
        })?;

        Ok(tonic::Response::new(AppParametersResponse {
            app_parameters: Some(app_parameters.into()),
        }))
    }
}
