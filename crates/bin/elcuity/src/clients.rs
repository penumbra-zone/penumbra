use std::str::FromStr;

use anyhow::Context;
use penumbra_sdk_custody::CustodyClient;
use penumbra_sdk_proto::box_grpc_svc::BoxGrpcService;
use penumbra_sdk_proto::core::component::dex::v1 as pb_dex;
use penumbra_sdk_proto::core::component::sct::v1 as pb_sct;
use penumbra_sdk_proto::custody::v1::custody_service_client::CustodyServiceClient;
use penumbra_sdk_proto::{box_grpc_svc, view::v1::view_service_client::ViewServiceClient};
use penumbra_sdk_view::{ViewClient, ViewServer};
use tonic::transport::Channel;

pub type SctQueryServiceClient = pb_sct::query_service_client::QueryServiceClient<Channel>;

pub type DexQueryServiceClient = pb_dex::query_service_client::QueryServiceClient<Channel>;

pub type DexSimulationServiceClient =
    pb_dex::simulation_service_client::SimulationServiceClient<Channel>;

pub type DynViewClient = dyn ViewClient + Send + 'static;

pub type DynCustodyClient = dyn CustodyClient + Send + 'static;

/// Provides utility for connecting to external services.
#[derive(Clone)]
pub struct Clients {
    node_channel: Channel,
    custody_client: CustodyServiceClient<BoxGrpcService>,
    view_client: ViewServiceClient<BoxGrpcService>,
}

impl Clients {
    /// Initialize the clients.
    ///
    /// This needs a way to contact a node serving an RPC,
    /// and a view service, which should also have custody enabled.
    pub async fn init(node_url: String, view_url: String) -> anyhow::Result<Self> {
        let node_channel = ViewServer::get_pd_channel(FromStr::from_str(&node_url)?)
            .await
            .context(format!("failed to connect to node at {}", &node_url))?;
        let endpoint = tonic::transport::Endpoint::new(view_url.clone())?;
        let svc = box_grpc_svc::connect(endpoint).await.context(format!(
            "failed to connect to view service at {}",
            &view_url
        ))?;
        let view_client = ViewServiceClient::new(svc.clone());
        let custody_client = CustodyServiceClient::new(svc);
        Ok(Self {
            node_channel,
            custody_client,
            view_client,
        })
    }

    // Choosing Box<dyn> over impl because dynamic dispatch is more
    // than fine for performance, and minding compile times.
    // I doubt that it makes much of a difference either way though.
    /// Get a view client.
    pub fn view(&self) -> Box<DynViewClient> {
        Box::new(self.view_client.clone())
    }

    pub fn custody(&self) -> Box<DynCustodyClient> {
        Box::new(self.custody_client.clone())
    }

    /// Get an SCT Query Service client.
    pub fn sct_query_service(&self) -> SctQueryServiceClient {
        SctQueryServiceClient::new(self.node_channel.clone())
    }

    /// Get a Dex Simulation Service client.
    pub fn dex_simulation_service(&self) -> DexSimulationServiceClient {
        DexSimulationServiceClient::new(self.node_channel.clone())
    }

    /// Get a Dex Query Service client
    pub fn dex_query_service(&self) -> DexQueryServiceClient {
        DexQueryServiceClient::new(self.node_channel.clone())
    }
}
