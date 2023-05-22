use anyhow::Result;
use penumbra_crypto::FullViewingKey;
use penumbra_custody::{AuthorizeRequest, CustodyClient};
use penumbra_transaction::{plan::TransactionPlan, AuthorizationData, Transaction};
use penumbra_view::ViewClient;
use rand_core::{CryptoRng, RngCore};

pub async fn build_transaction<V, C, R>(
    fvk: &FullViewingKey,
    view: &mut V,
    custody: &mut C,
    mut rng: R,
    plan: TransactionPlan,
) -> Result<Transaction>
where
    V: ViewClient,
    C: CustodyClient,
    R: RngCore + CryptoRng,
{
    // Get the authorization data from the custody service...
    let auth_data: AuthorizationData = custody
        .authorize(AuthorizeRequest {
            account_group_id: Some(fvk.account_group_id()),
            plan: plan.clone(),
            pre_authorizations: Vec::new(),
        })
        .await?
        .data
        .ok_or_else(|| anyhow::anyhow!("empty AuthorizeResponse message"))?
        .try_into()?;

    // Send a witness request to the view service to get witness data
    let witness_data = view.witness(fvk.account_group_id(), &plan).await?;

    // ... and then build the transaction:
    #[cfg(not(feature = "parallel"))]
    {
        let unauth_tx = plan.build(fvk, witness_data)?;
        let tx = unauth_tx.authorize(&mut rng, &auth_data)?;
        return Ok(tx);
    }

    #[cfg(feature = "parallel")]
    {
        let tx = plan
            .build_concurrent(&mut rng, fvk, witness_data)
            .await
            .map_err(|_| tonic::Status::failed_precondition("Error building transaction"))?
            .authorize(&mut rng, &auth_data)
            .map_err(|_| tonic::Status::failed_precondition("Error authorizing transaction"))?;

        Ok(tx)
    }
}
