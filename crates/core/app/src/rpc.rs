use {
    crate::{app::StateReadExt as _, PenumbraHost},
    anyhow::Context,
    cnidarium::{
        rpc::{
            proto::v1alpha1::query_service_server::QueryServiceServer as StorageQueryServiceServer,
            Server as StorageServer,
        },
        Storage,
    },
    ibc_proto::ibc::core::{
        channel::v1::query_server::QueryServer as ChannelQueryServer,
        client::v1::query_server::QueryServer as ClientQueryServer,
        connection::v1::query_server::QueryServer as ConnectionQueryServer,
    },
    penumbra_chain::component::{rpc::Server as ChainServer, StateReadExt as _},
    penumbra_compact_block::component::rpc::Server as CompactBlockServer,
    penumbra_dex::component::rpc::Server as DexServer,
    penumbra_governance::component::rpc::Server as GovernanceServer,
    penumbra_ibc::component::rpc::IbcQuery,
    penumbra_proto::core::app::v1alpha1::{
        query_service_server::QueryService, AppParametersRequest, AppParametersResponse,
        TransactionsByHeightRequest, TransactionsByHeightResponse,
    },
    penumbra_proto::core::component::dex::v1alpha1::simulation_service_server::SimulationServiceServer,
    penumbra_proto::core::{
        app::v1alpha1::query_service_server::QueryServiceServer as AppQueryServiceServer,
        component::{
            chain::v1alpha1::query_service_server::QueryServiceServer as ChainQueryServiceServer,
            compact_block::v1alpha1::query_service_server::QueryServiceServer as CompactBlockQueryServiceServer,
            dex::v1alpha1::query_service_server::QueryServiceServer as DexQueryServiceServer,
            governance::v1alpha1::query_service_server::QueryServiceServer as GovernanceQueryServiceServer,
            sct::v1alpha1::query_service_server::QueryServiceServer as SctQueryServiceServer,
            shielded_pool::v1alpha1::query_service_server::QueryServiceServer as ShieldedPoolQueryServiceServer,
            stake::v1alpha1::query_service_server::QueryServiceServer as StakeQueryServiceServer,
        },
    },
    penumbra_proto::util::tendermint_proxy::v1alpha1::tendermint_proxy_service_server::TendermintProxyServiceServer,
    penumbra_sct::component::rpc::Server as SctServer,
    penumbra_shielded_pool::component::rpc::Server as ShieldedPoolServer,
    penumbra_stake::component::rpc::Server as StakeServer,
    penumbra_tendermint_proxy::TendermintProxy,
    tonic::Status,
    tonic_web::enable as we,
    tracing::instrument,
};

/// Used to construct a [`tonic::transport::server::Routes`] router.
///
/// Use [`Routes::build()`] to try to construct a set of tonic routes.
pub struct Routes {
    pub storage: Storage,
    pub ibc: IbcQuery<PenumbraHost>,
    pub tendermint_proxy: TendermintProxy,
    pub enable_expensive_rpc: bool,
}

impl Routes {
    /// Returns a [`Routes`] that can be used to service requests.
    pub fn build(self) -> Result<tonic::transport::server::Routes, anyhow::Error> {
        let Self {
            storage,
            ibc,
            tendermint_proxy,
            enable_expensive_rpc,
        } = self;

        let routes = tonic::transport::server::Routes::new(we(StorageQueryServiceServer::new(
            StorageServer::new(storage.clone()),
        )))
        .add_service(AppQueryServiceServer::new(self::Server::new(
            storage.clone(),
        )))
        .add_service(we(ChainQueryServiceServer::new(ChainServer::new(
            storage.clone(),
        ))))
        .add_service(we(CompactBlockQueryServiceServer::new(
            CompactBlockServer::new(storage.clone()),
        )))
        .add_service(we(DexQueryServiceServer::new(DexServer::new(
            storage.clone(),
        ))))
        .add_service(we(GovernanceQueryServiceServer::new(
            GovernanceServer::new(storage.clone()),
        )))
        .add_service(we(SctQueryServiceServer::new(SctServer::new(
            storage.clone(),
        ))))
        .add_service(we(ShieldedPoolQueryServiceServer::new(
            ShieldedPoolServer::new(storage.clone()),
        )))
        .add_service(we(StakeQueryServiceServer::new(StakeServer::new(
            storage.clone(),
        ))))
        .add_service(we(ClientQueryServer::new(ibc.clone())))
        .add_service(we(ChannelQueryServer::new(ibc.clone())))
        .add_service(we(ConnectionQueryServer::new(ibc.clone())))
        .add_service(we(TendermintProxyServiceServer::new(tendermint_proxy)))
        .add_service(we(tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(penumbra_proto::FILE_DESCRIPTOR_SET)
            .build()
            .with_context(|| "could not configure grpc reflection service")?));

        let routes = if enable_expensive_rpc {
            routes.add_service(we(SimulationServiceServer::new(DexServer::new(
                storage.clone(),
            ))))
        } else {
            routes
        };

        Ok(routes)
    }
}

struct Server {
    storage: Storage,
}

impl Server {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

#[tonic::async_trait]
impl QueryService for Server {
    #[instrument(skip(self, request))]
    async fn transactions_by_height(
        &self,
        request: tonic::Request<TransactionsByHeightRequest>,
    ) -> Result<tonic::Response<TransactionsByHeightResponse>, Status> {
        let state = self.storage.latest_snapshot();
        state
            .check_chain_id(&request.get_ref().chain_id)
            .await
            .map_err(|e| tonic::Status::unknown(format!("chain_id not OK: {e}")))?;
        let request_inner = request.into_inner();
        let block_height = request_inner.block_height;

        let tx_response = state
            .transactions_by_height(block_height)
            .await
            .map_err(|e| tonic::Status::internal(format!("transaction response bad: {e}")))?;

        Ok(tonic::Response::new(tx_response))
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
