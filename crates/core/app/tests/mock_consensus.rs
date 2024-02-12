//! App integration tests using mock consensus.
//
//  Note: these should eventually replace the existing test cases. mock consensus tests are placed
//  here while the engine is still in development. See #3588.

mod common;

use anyhow::anyhow;
use cnidarium::TempStorage;
use penumbra_app::server::consensus::Consensus;

const MESSAGE: &str = "hello penumbra abci!";

#[tokio::test]
async fn an_app_with_mock_consensus_can_be_instantiated_2_electric_boogaloo() -> anyhow::Result<()>
{
    // Install a tracing subscriber to log events during the duration of this test.
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("trace")
        .pretty()
        .with_test_writer()
        .finish();
    let guard = tracing::subscriber::set_default(subscriber);

    // Reserve some temporary storage that will be cleaned up after the test is finished.
    let storage = TempStorage::new().await?;
    let consensus = Consensus::new(storage.as_ref().clone());

    use penumbra_mock_consensus::TestNode;
    let _engine = TestNode::builder()
        .single_validator()
        .app_state(() /*genesis::AppState::default()*/)
        .init_chain(consensus)
        .await;

    drop(guard);

    Ok(())
}
