use std::future::Future;

use penumbra_sdk_view::Planner;

use crate::clients::Clients;

pub async fn build_and_submit<F, Fut>(
    clients: &Clients,
    source: AddressIndex,
    add_to_plan: F,
) -> anyhow::Result<Transaction>
where
    F: FnOnce(&mut Planner<OsRng>) -> Fut,
    Fut: Future<Output = anyhow::Result<()>>,
{
    let mut view = clients.view();
    let mut custody = clients.custody();

    let mut planner = Planner::new(OsRng);
    planner.set_gas_prices(view.gas_prices().await?);
    add_to_plan(&mut planner).await?;
    let plan = planner.plan(view.as_mut(), source).await?;
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

    Ok(tx)
}
