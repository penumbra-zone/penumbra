use anyhow::Result;
use penumbra_crypto::keys::{SeedPhrase, SpendKey, SpendSeed};
use penumbra_custody::SoftHSM;
use penumbra_proto::{
    client::oblivious::oblivious_query_client::ObliviousQueryClient,
    custody::{
        custody_protocol_client::CustodyProtocolClient,
        custody_protocol_server::CustodyProtocolServer,
    },
    view::{view_protocol_client::ViewProtocolClient, view_protocol_server::ViewProtocolServer},
};
use penumbra_transaction::plan::TransactionPlan;
use penumbra_view::{Storage, ViewService};
use penumbra_wallet_next::build_transaction;
use rand_core::OsRng;

#[tokio::main]
async fn main() -> Result<()> {
    // stub code to check that generics are well-formed in wallet-next

    let sk = SpendKey::new(SpendSeed::from_seed_phrase(SeedPhrase::generate(OsRng), 0));
    let fvk = sk.full_viewing_key().clone();

    let oq_client =
        ObliviousQueryClient::connect(format!("http://{}:{}", "testnet.penumbra.zone", "8080"))
            .await?;

    let storage = Storage::load("tmp.sqlite".to_string()).await?;
    let view_service = ViewService::new(storage, oq_client).await?;
    let custody_service = SoftHSM::new(vec![sk]);

    // local, in-memory servers
    let vc1 = ViewProtocolClient::new(ViewProtocolServer::new(view_service));
    let cc1 = CustodyProtocolClient::new(CustodyProtocolServer::new(custody_service));

    // remote servers
    let vc2 = ViewProtocolClient::connect("http://example.com:8080").await?;
    let cc2 = CustodyProtocolClient::connect("http://example.com:8080").await?;

    let plan = TransactionPlan::default();

    // both of these sholud compile, proving that the generics capture what we want

    // local servers
    let _tx1 = build_transaction(&fvk, plan.clone(), vc1, cc1, OsRng).await?;
    // remote servers
    let _tx2 = build_transaction(&fvk, plan.clone(), vc2, cc2, OsRng).await?;

    Ok(())
}
