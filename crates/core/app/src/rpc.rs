mod query;

// TODO: Once we migrate to Tonic 0.10.0, we'll be able to use the `Routes` structure to have each
// component define a method that returns a `Routes` with all of its query services bundled inside.
//
// This means we won't have to import all this shit and recite every single service -- we can e.g.,
// have the app crate assemble all of its components' query services into a single `Routes` and
// then just add that to the gRPC server.
use {
    self::query::AppQueryServer,
    crate::PenumbraHost,
    anyhow::Context,
    cnidarium::{
        proto::v1::query_service_server::QueryServiceServer as StorageQueryServiceServer,
        rpc::Server as StorageServer,
    },
    ibc_proto::{
        cosmos::bank::v1beta1::query_server::QueryServer as TransferQueryServer,
        ibc::{
            applications::transfer::v1::query_server::QueryServer as BankQueryServer,
            core::{
                channel::v1::query_server::QueryServer as ChannelQueryServer,
                client::v1::query_server::QueryServer as ClientQueryServer,
                connection::v1::query_server::QueryServer as ConnectionQueryServer,
            },
        },
    },
    penumbra_sdk_auction::component::rpc::Server as AuctionServer,
    penumbra_sdk_community_pool::component::rpc::Server as CommunityPoolServer,
    penumbra_sdk_compact_block::component::rpc::Server as CompactBlockServer,
    penumbra_sdk_dex::component::rpc::Server as DexServer,
    penumbra_sdk_fee::component::rpc::Server as FeeServer,
    penumbra_sdk_funding::component::rpc::Server as FundingServer,
    penumbra_sdk_governance::component::rpc::Server as GovernanceServer,
    penumbra_sdk_proto::{
        core::{
            app::v1::query_service_server::QueryServiceServer as AppQueryServiceServer,
            component::{
                auction::v1::query_service_server::QueryServiceServer as AuctionQueryServiceServer,
                community_pool::v1::query_service_server::QueryServiceServer as CommunityPoolQueryServiceServer,
                compact_block::v1::query_service_server::QueryServiceServer as CompactBlockQueryServiceServer,
                dex::v1::{
                    query_service_server::QueryServiceServer as DexQueryServiceServer,
                    simulation_service_server::SimulationServiceServer,
                },
                fee::v1::query_service_server::QueryServiceServer as FeeQueryServiceServer,
                funding::v1::funding_service_server::FundingServiceServer as FundingQueryServiceServer,
                governance::v1::query_service_server::QueryServiceServer as GovernanceQueryServiceServer,
                sct::v1::query_service_server::QueryServiceServer as SctQueryServiceServer,
                shielded_pool::v1::query_service_server::QueryServiceServer as ShieldedPoolQueryServiceServer,
                stake::v1::query_service_server::QueryServiceServer as StakeQueryServiceServer,
            },
        },
        util::tendermint_proxy::v1::tendermint_proxy_service_server::{
            TendermintProxyService, TendermintProxyServiceServer,
        },
    },
    penumbra_sdk_sct::component::rpc::Server as SctServer,
    penumbra_sdk_shielded_pool::component::rpc::Server as ShieldedPoolServer,
    penumbra_sdk_stake::component::rpc::Server as StakeServer,
    tonic::service::Routes,
    tonic_web::enable as we,
};

pub fn routes(
    storage: &cnidarium::Storage,
    tm_proxy: impl TendermintProxyService,
    _enable_expensive_rpc: bool,
) -> anyhow::Result<tonic::service::Routes> {
    let ibc = penumbra_sdk_ibc::component::rpc::IbcQuery::<PenumbraHost>::new(storage.clone());

    let mut builder = Routes::builder();
    builder
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
        .add_service(we(AuctionQueryServiceServer::new(AuctionServer::new(
            storage.clone(),
        ))))
        .add_service(we(AppQueryServiceServer::new(AppQueryServer::new(
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
        .add_service(we(TransferQueryServer::new(ShieldedPoolServer::new(
            storage.clone(),
        ))))
        .add_service(we(BankQueryServer::new(ShieldedPoolServer::new(
            storage.clone(),
        ))))
        .add_service(we(CommunityPoolQueryServiceServer::new(
            CommunityPoolServer::new(storage.clone()),
        )))
        .add_service(we(StakeQueryServiceServer::new(StakeServer::new(
            storage.clone(),
        ))))
        .add_service(we(ClientQueryServer::new(ibc.clone())))
        .add_service(we(ChannelQueryServer::new(ibc.clone())))
        .add_service(we(ConnectionQueryServer::new(ibc.clone())))
        .add_service(we(TendermintProxyServiceServer::new(tm_proxy)))
        .add_service(we(SimulationServiceServer::new(DexServer::new(
            storage.clone(),
        ))))
        .add_service(we(FundingQueryServiceServer::new(FundingServer::new(
            storage.clone(),
        ))))
        .add_service(we(tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(penumbra_sdk_proto::FILE_DESCRIPTOR_SET)
            .build_v1()
            .with_context(|| "could not configure grpc reflection service")?));
    Ok(builder.routes().prepare())
}
