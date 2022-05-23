use penumbra_crypto::FullViewingKey;
use penumbra_custody::{AuthorizeRequest, CustodyProtocolClient};
use penumbra_proto::view::{view_protocol_client::ViewProtocolClient};
use penumbra_transaction::{plan::TransactionPlan, Transaction};
use rand_core::{CryptoRng, RngCore};
use tonic::Request;

pub async fn build_transaction<V, C, R>(
    fvk: &FullViewingKey,
    plan: TransactionPlan,
    mut view_client: ViewProtocolClient<V>,
    mut custody_client: CustodyProtocolClient<C>,
    rng: R,
) -> anyhow::Result<Transaction>
where
    R: RngCore + CryptoRng,
    // TODO: find a way to pull out these trait bounds
    // e.g., can we make our own trait that wraps this up and
    // also hides the proto conversions?
    // for now: fine but ugly
    V: tonic::client::GrpcService<tonic::body::BoxBody>,
    V::ResponseBody: tonic::codegen::Body + Send + 'static,
    V::Error: Into<tonic::codegen::StdError>,
    <V::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + Send,
    C: tonic::client::GrpcService<tonic::body::BoxBody>,
    C::ResponseBody: tonic::codegen::Body + Send + 'static,
    C::Error: Into<tonic::codegen::StdError>,
    <C::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::codegen::StdError> + Send,
{
    // First, request authorization:
    let auth_data = custody_client
        .authorize(Request::new(
            AuthorizeRequest {
                plan: plan.into(),
                fvk_hash: fvk.hash(),
            }
            .into(),
        ))
        .await?;

    // Next, get the witness data:
    // TODO: unify AuthPathsResponse with WitnessData
    let witness_data = view_client
        .auth_paths(Request::new(
            AuthPathsRequest {
                notes: plan
                    .spend_plans()
                    .map(|spend| spend.note.commit())
                    .collect(),
                fvk_hash: fvk.hash(),
            }
            .into(),
        ))
        .await?;
    
    // Finally, build the transaction:
    plan.build(&mut rng, fvk, auth_data, witness_data)
}
