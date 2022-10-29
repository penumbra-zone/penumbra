use anyhow::Result;
use penumbra_crypto::FullViewingKey;
use penumbra_custody::{AuthorizeRequest, CustodyClient};
use penumbra_transaction::{plan::TransactionPlan, Transaction};
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
    let auth_data = custody
        .authorize(AuthorizeRequest {
            account_id: fvk.hash(),
            plan: plan.clone(),
        })
        .await?;

    // Send a witness request to the view service to get witness data
    let witness_data = view.witness(fvk.hash(), &mut rng, &plan).await?;

    // ... and then build the transaction:
    plan.build(&mut rng, fvk, auth_data, witness_data)
}
