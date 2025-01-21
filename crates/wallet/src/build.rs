use anyhow::Result;

use penumbra_sdk_custody::{AuthorizeRequest, CustodyClient};
use penumbra_sdk_keys::FullViewingKey;
use penumbra_sdk_transaction::{AuthorizationData, Transaction, TransactionPlan};
use penumbra_sdk_view::ViewClient;

pub async fn build_transaction<V, C>(
    fvk: &FullViewingKey,
    view: &mut V,
    custody: &mut C,
    plan: TransactionPlan,
) -> Result<Transaction>
where
    V: ViewClient,
    C: CustodyClient,
{
    // Get the authorization data from the custody service...
    let auth_data: AuthorizationData = custody
        .authorize(AuthorizeRequest {
            plan: plan.clone(),
            pre_authorizations: Vec::new(),
        })
        .await?
        .data
        .ok_or_else(|| anyhow::anyhow!("empty AuthorizeResponse message"))?
        .try_into()?;

    // Send a witness request to the view service to get witness data
    let witness_data = view.witness(&plan).await?;

    // ... and then build the transaction:
    #[cfg(not(feature = "parallel"))]
    {
        let tx = plan.build(fvk, &witness_data, &auth_data)?;
        return Ok(tx);
    }

    #[cfg(feature = "parallel")]
    {
        let tx = plan
            .build_concurrent(fvk, &witness_data, &auth_data)
            .await
            .map_err(|_| tonic::Status::failed_precondition("Error building transaction"))?;

        Ok(tx)
    }
}
