use anyhow::Result;
use penumbra_crypto::FullViewingKey;
use penumbra_custody::{AuthorizeRequest, CustodyClient};
use penumbra_proto::view::WitnessRequest;
use penumbra_transaction::{plan::TransactionPlan, Transaction};
use penumbra_view::ViewClient;
use rand_core::{CryptoRng, RngCore};

pub async fn build_transaction<V, C, R>(
    fvk: &FullViewingKey,
    plan: TransactionPlan,
    mut view: V,
    mut custody: C,
    mut rng: R,
) -> Result<Transaction>
where
    V: ViewClient,
    C: CustodyClient,
    R: RngCore + CryptoRng,
{
    // Get the authorization data from the custody service...
    let auth_data = custody
        .authorize(AuthorizeRequest {
            fvk_hash: fvk.hash(),
            plan: plan.clone(),
        })
        .await?;

    // Get the witness data from the view service...
    let witness_data = view
        .witness(WitnessRequest {
            fvk_hash: Some(fvk.hash().into()),
            note_commitments: plan
                .spend_plans()
                .map(|spend| spend.note.commit().into())
                .collect(),
        })
        .await?;

    // ... and then build the transaction:
    plan.build(&mut rng, fvk, auth_data, witness_data)
}
