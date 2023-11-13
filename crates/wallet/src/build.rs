use anyhow::Result;
use penumbra_custody::{AuthorizeRequest, CustodyClient};
use penumbra_keys::FullViewingKey;
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
            wallet_id: Some(fvk.wallet_id()),
            plan: plan.clone(),
            pre_authorizations: Vec::new(),
        })
        .await?
        .data
        .ok_or_else(|| anyhow::anyhow!("empty AuthorizeResponse message"))?
        .try_into()?;

    // Send a witness request to the view service to get witness data
    let witness_data = view.witness(fvk.wallet_id(), &plan).await?;

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
            .build_concurrent(fvk, &witness_data, &auth_data)
            .await
            .map_err(|_| tonic::Status::failed_precondition("Error building transaction"))?;

        Ok(tx)
    }
}
