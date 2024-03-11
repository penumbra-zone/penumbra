use anyhow::Context;
use ibc_proto::ibc::core::channel::v1::query_server::QueryServer as ChannelQueryServer;
use ibc_proto::ibc::core::client::v1::query_server::QueryServer as ClientQueryServer;
use ibc_proto::ibc::core::connection::v1::query_server::QueryServer as ConnectionQueryServer;
use penumbra_app::PenumbraHost;
use penumbra_proto::core::component::dex::v1::simulation_service_server::SimulationServiceServer;
use penumbra_proto::util::tendermint_proxy::v1::tendermint_proxy_service_server::TendermintProxyServiceServer;
use penumbra_tendermint_proxy::TendermintProxy;
use penumbra_tower_trace::remote_addr;
use tonic::transport::server::Server;

pub fn f(
    storage: &cnidarium::Storage,
    cometbft_addr: url::Url,
    enable_expensive_rpc: bool,
) -> anyhow::Result<tonic::transport::server::Router> {
    // TODO: Once we migrate to Tonic 0.10.0, we'll be able to use the
    // `Routes` structure to have each component define a method that
    // returns a `Routes` with all of its query services bundled inside.
    //
    // This means we won't have to import all this shit and recite every
    // single service -- we can e.g., have the app crate assemble all of
    // its components' query services into a single `Routes` and then
    // just add that to the gRPC server.

    use cnidarium::rpc::proto::v1::query_service_server::QueryServiceServer as StorageQueryServiceServer;
    use penumbra_proto::core::{
        app::v1::query_service_server::QueryServiceServer as AppQueryServiceServer,
        component::{
            compact_block::v1::query_service_server::QueryServiceServer as CompactBlockQueryServiceServer,
            dex::v1::query_service_server::QueryServiceServer as DexQueryServiceServer,
            fee::v1::query_service_server::QueryServiceServer as FeeQueryServiceServer,
            governance::v1::query_service_server::QueryServiceServer as GovernanceQueryServiceServer,
            sct::v1::query_service_server::QueryServiceServer as SctQueryServiceServer,
            shielded_pool::v1::query_service_server::QueryServiceServer as ShieldedPoolQueryServiceServer,
            stake::v1::query_service_server::QueryServiceServer as StakeQueryServiceServer,
        },
    };
    use tonic_web::enable as we;

    use cnidarium::rpc::Server as StorageServer;
    use penumbra_app::rpc::Server as AppServer;
    use penumbra_compact_block::component::rpc::Server as CompactBlockServer;
    use penumbra_dex::component::rpc::Server as DexServer;
    use penumbra_fee::component::rpc::Server as FeeServer;
    use penumbra_governance::component::rpc::Server as GovernanceServer;
    use penumbra_sct::component::rpc::Server as SctServer;
    use penumbra_shielded_pool::component::rpc::Server as ShieldedPoolServer;
    use penumbra_stake::component::rpc::Server as StakeServer;

    let tm_proxy = TendermintProxy::new(cometbft_addr);
    let ibc = penumbra_ibc::component::rpc::IbcQuery::<PenumbraHost>::new(storage.clone());
    let mut grpc_server = Server::builder()
        .trace_fn(|req| match remote_addr(req) {
            Some(remote_addr) => {
                tracing::error_span!("grpc", ?remote_addr)
            }
            None => tracing::error_span!("grpc"),
        })
        // Allow HTTP/1, which will be used by grpc-web connections.
        // This is particularly important when running locally, as gRPC
        // typically uses HTTP/2, which requires HTTPS. Accepting HTTP/2
        // allows local applications such as web browsers to talk to pd.
        .accept_http1(true)
        // As part of #2932, we are disabling all timeouts until we circle back to our
        // performance story.
        // Sets a timeout for all gRPC requests, but note that in the case of streaming
        // requests, the timeout is only applied to the initial request. This means that
        // this does not prevent long lived streams, for example to allow clients to obtain
        // new blocks.
        // .timeout(std::time::Duration::from_secs(7))
        // Wrap each of the gRPC services in a tonic-web proxy:
        .add_service(we(StorageQueryServiceServer::new(StorageServer::new(
            storage.clone(),
        ))))
        .add_service(we(AppQueryServiceServer::new(AppServer::new(
            storage.clone(),
        ))))
        .add_service(we(CompactBlockQueryServiceServer::new(
            CompactBlockServer::new(storage.clone()),
        )))
        .add_service(we(DexQueryServiceServer::new(DexServer::new(
            storage.clone(),
        ))))
        .add_service(we(FeeQueryServiceServer::new(FeeServer::new(
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
        .add_service(we(TendermintProxyServiceServer::new(tm_proxy.clone())))
        .add_service(we(tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(penumbra_proto::FILE_DESCRIPTOR_SET)
            .build()
            .with_context(|| "could not configure grpc reflection service")?));

    if enable_expensive_rpc {
        grpc_server = grpc_server.add_service(we(SimulationServiceServer::new(DexServer::new(
            storage.clone(),
        ))));
    }

    Ok(grpc_server)
}
