//! Contains an abstraction over connecting to the actual chain.
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::Context;
use futures::TryStreamExt as _;
use penumbra_sdk_custody::AuthorizeRequest;
use penumbra_sdk_custody::CustodyClient;
use penumbra_sdk_dex::lp::position::Position;
use penumbra_sdk_keys::keys::AddressIndex;
use penumbra_sdk_proto::box_grpc_svc::BoxGrpcService;
use penumbra_sdk_proto::core::component::dex::v1 as pb_dex;
use penumbra_sdk_proto::core::component::dex::v1::LiquidityPositionsByIdRequest;
use penumbra_sdk_proto::custody::v1::custody_service_client::CustodyServiceClient;
use penumbra_sdk_proto::view::v1::broadcast_transaction_response::Status as BroadcastStatus;
use penumbra_sdk_proto::{box_grpc_svc, view::v1::view_service_client::ViewServiceClient};
use penumbra_sdk_transaction::{Transaction, TransactionPlan};
use penumbra_sdk_view::Planner;
use penumbra_sdk_view::ViewClient;
use penumbra_sdk_view::ViewServer;
use rand_core::OsRng;
use tonic::transport::Channel;

use crate::dex::Registry;

type DexQueryServiceClient = pb_dex::query_service_client::QueryServiceClient<Channel>;

#[allow(dead_code)]
pub struct Client {
    node_channel: Channel,
    custody_client: CustodyServiceClient<BoxGrpcService>,
    view_client: ViewServiceClient<BoxGrpcService>,
}

#[allow(dead_code)]
impl Client {
    /// Initialize a client.
    ///
    /// `node_url` should point to a node.
    ///
    /// `view_url` should point to a view service instance, with custody support
    /// (e.g. `pclientd`).
    pub async fn init(node_url: &str, view_url: &str) -> anyhow::Result<Self> {
        let node_channel = ViewServer::get_pd_channel(FromStr::from_str(&node_url)?)
            .await
            .context(format!("failed to connect to node at {}", &node_url))?;
        let endpoint = tonic::transport::Endpoint::new(view_url.to_string())?;
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

    pub async fn registry(&mut self) -> anyhow::Result<Registry> {
        let res = ViewClient::assets(&mut self.view_client).await?;
        Ok(Registry::from_metadata(&mut res.values()))
    }

    async fn submit(&mut self, plan: TransactionPlan) -> anyhow::Result<Transaction> {
        let auth_data = CustodyClient::authorize(
            &mut self.custody_client,
            AuthorizeRequest {
                plan: plan.clone(),
                pre_authorizations: Default::default(),
            },
        )
        .await?
        .data
        .expect("auth data should be present")
        .try_into()?;
        let tx = ViewClient::witness_and_build(&mut self.view_client, plan, auth_data).await?;
        let mut rsp =
            ViewClient::broadcast_transaction(&mut self.view_client, tx.clone(), true).await?;
        let tx_id = format!("{}", tx.id());

        while let Some(rsp) = rsp.try_next().await? {
            match rsp.status.ok_or(anyhow!("missing status"))? {
                BroadcastStatus::BroadcastSuccess(_) => {
                    tracing::info!(tx_id, "transaction broadcast");
                }
                BroadcastStatus::Confirmed(c) => {
                    tracing::info!(tx_id, height = c.detection_height, "transaction confirmed");
                    break;
                }
            }
        }

        Ok(tx)
    }

    pub async fn positions(&mut self) -> anyhow::Result<Vec<Position>> {
        let positions =
            ViewClient::owned_position_ids(&mut self.view_client, None, None, None).await?;
        let positions_proto: Vec<_> = positions.into_iter().map(|x| x.into()).collect();
        let count = positions_proto.len();
        let mut resp = DexQueryServiceClient::new(self.node_channel.clone())
            .liquidity_positions_by_id(LiquidityPositionsByIdRequest {
                position_id: positions_proto,
            })
            .await?
            .into_inner();
        let mut out = Vec::with_capacity(count);
        while let Some(x) = resp.try_next().await? {
            out.push(
                x.data
                    .ok_or_else(|| anyhow!("expected position in LiquidityPositionsByIdResponse"))?
                    .try_into()?,
            );
        }

        Ok(out)
    }

    /// This will return Some if the closure returns Some.
    pub async fn build_and_submit<F>(
        &mut self,
        source: AddressIndex,
        add_to_plan: F,
    ) -> anyhow::Result<Option<Transaction>>
    where
        F: FnOnce(Planner<OsRng>) -> anyhow::Result<Option<Planner<OsRng>>>,
    {
        let mut planner = Planner::new(OsRng);
        planner.set_gas_prices(ViewClient::gas_prices(&mut self.view_client).await?);
        let Some(mut planner) = add_to_plan(planner)? else {
            return Ok(None);
        };
        let plan = planner.plan(&mut self.view_client, source).await?;

        Ok(Some(self.submit(plan).await?))
    }
}
