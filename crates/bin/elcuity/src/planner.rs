use anyhow::anyhow;
use futures::TryStreamExt as _;
use penumbra_sdk_custody::AuthorizeRequest;
use penumbra_sdk_keys::keys::AddressIndex;
use penumbra_sdk_proto::view::v1::broadcast_transaction_response::Status as BroadcastStatus;
use penumbra_sdk_transaction::{Transaction, TransactionPlan};
use penumbra_sdk_view::Planner;
use rand_core::OsRng;
use std::future::Future;

use crate::clients::Clients;

pub async fn submit(clients: &Clients, plan: TransactionPlan) -> anyhow::Result<Transaction> {
    let mut view = clients.view();
    let mut custody = clients.custody();

    let auth_data = custody
        .authorize(AuthorizeRequest {
            plan: plan.clone(),
            pre_authorizations: Default::default(),
        })
        .await?
        .data
        .expect("auth data should be present")
        .try_into()?;
    let tx = view.witness_and_build(plan, auth_data).await?;
    let mut rsp = view.broadcast_transaction(tx.clone(), true).await?;
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

pub async fn build_and_submit<F, Fut>(
    clients: &Clients,
    source: AddressIndex,
    add_to_plan: F,
) -> anyhow::Result<Transaction>
where
    F: FnOnce(Planner<OsRng>) -> Fut,
    Fut: Future<Output = anyhow::Result<Planner<OsRng>>>,
{
    let mut view = clients.view();

    let mut planner = Planner::new(OsRng);
    planner.set_gas_prices(view.gas_prices().await?);
    let mut planner = add_to_plan(planner).await?;
    let plan = planner.plan(view.as_mut(), source).await?;

    submit(clients, plan).await
}
