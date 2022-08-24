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

    // Get the current FMD parameters from the view service...
    let fmd_params = view.fmd_parameters().await?;
    let precision_bits = fmd_params.precision_bits;

    // ... and then build the transaction:
    plan.build(
        &mut rng,
        fvk,
        auth_data,
        witness_data,
        precision_bits.into(),
    )
}
