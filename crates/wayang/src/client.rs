//! Contains an abstraction over connecting to the actual chain.
use anyhow::Context;
use penumbra_sdk_proto::box_grpc_svc::BoxGrpcService;
use penumbra_sdk_proto::custody::v1::custody_service_client::CustodyServiceClient;
use penumbra_sdk_proto::{box_grpc_svc, view::v1::view_service_client::ViewServiceClient};
use penumbra_sdk_view::ViewClient;

use crate::registry::Registry;

#[allow(dead_code)]
pub struct Client {
    custody_client: CustodyServiceClient<BoxGrpcService>,
    view_client: ViewServiceClient<BoxGrpcService>,
}

#[allow(dead_code)]
impl Client {
    /// Initialize a client.
    ///
    /// `view_url` should point to a view service instance, with custody support
    /// (e.g. `pclientd`).
    pub async fn init(view_url: &str) -> anyhow::Result<Self> {
        let endpoint = tonic::transport::Endpoint::new(view_url.to_string())?;
        let svc = box_grpc_svc::connect(endpoint).await.context(format!(
            "failed to connect to view service at {}",
            &view_url
        ))?;
        let view_client = ViewServiceClient::new(svc.clone());
        let custody_client = CustodyServiceClient::new(svc);
        Ok(Self {
            custody_client,
            view_client,
        })
    }

    pub async fn registry(&mut self) -> anyhow::Result<Registry> {
        let res = ViewClient::assets(&mut self.view_client).await?;
        Ok(Registry::from_metadata(&mut res.values()))
    }
}
