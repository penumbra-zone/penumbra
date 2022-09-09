use anyhow::Result;
use penumbra_crypto::FullViewingKey;
use penumbra_custody::{AuthorizeRequest, CustodyClient};
use penumbra_proto::view::WitnessRequest;
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

    // Get the witness data from the view service only for non-zero amounts of value,
    // since dummy spends will have a zero amount.
    let note_commitments = plan
        .spend_plans()
        .filter(|plan| plan.note.amount() != 0)
        .map(|spend| spend.note.commit().into())
        .collect();
    let witness_data = view
        .witness(WitnessRequest {
            account_id: Some(fvk.hash().into()),
            note_commitments,
        })
        .await?;

    // ... and then build the transaction:
    plan.build(&mut rng, fvk, auth_data, witness_data)
}
