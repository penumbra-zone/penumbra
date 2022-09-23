use anyhow::Result;
use penumbra_crypto::FullViewingKey;
use penumbra_custody::{AuthorizeRequest, CustodyClient};
use penumbra_proto::view::v1alpha1::WitnessRequest;
use penumbra_tct::Proof;
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
        .filter(|plan| plan.note.amount() != 0u64.into())
        .map(|spend| spend.note.commit().into())
        .chain(
            plan.swap_claim_plans()
                .map(|swap_claim| swap_claim.swap_nft_note.commit().into()),
        )
        .collect();
    let mut witness_data = view
        .witness(WitnessRequest {
            account_id: Some(fvk.hash().into()),
            note_commitments,
        })
        .await?;

    // Now we need to augment the witness data with dummy proofs such that
    // note commitments corresponding to dummy spends also have proofs.
    for nc in plan
        .spend_plans()
        .filter(|plan| plan.note.amount() == 0u64.into())
        .map(|plan| plan.note.commit())
    {
        witness_data.add_proof(nc, Proof::dummy(&mut rng, nc));
    }

    // ... and then build the transaction:
    plan.build(&mut rng, fvk, auth_data, witness_data)
}
