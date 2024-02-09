//! App integration tests using mock consensus.
//
//  Note: these should eventually replace the existing test cases. mock consensus tests are placed
//  here while the engine is still in development. See #3588.

mod common;

use anyhow::anyhow;
use cnidarium::TempStorage;

const MESSAGE: &str = "hello penumbra abci!";

#[tokio::test]
async fn an_app_with_mock_consensus_can_be_instantiated() -> anyhow::Result<()> {
    // Install a tracing subscriber to log events during the duration of this test.
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("trace")
        .pretty()
        .with_test_writer()
        .finish();
    let guard = tracing::subscriber::set_default(subscriber);

    // Reserve some temporary storage that will be cleaned up after the test is finished.
    let storage = TempStorage::new().await?;

    // Instantiate the ABCI application, and an in-memory connection.
    let (abci_connection, mut response_stream, mut request_sink) = {
        let storage = storage.as_ref().clone();
        let server = penumbra_app::server::new(storage);
        const MAX_BUF_SIZE: usize = 1024 * 8;
        server.connect_local(MAX_BUF_SIZE)
    };

    // Send an "echo" message to the application...
    tracing::info!("sending echo request to abci app");
    {
        use futures::SinkExt;
        use tendermint::v0_37::abci::request::{Echo, Request};
        let echo = Request::Echo(Echo {
            message: MESSAGE.to_owned(),
        });
        let echo = echo.into();
        request_sink.send(echo).await.ok(/*TODO: error must be `Debug`*/).unwrap()
    }
    tracing::info!("sent echo request to abci app");

    // ...and then read the "echo" response back.
    tracing::info!("receiving echo response from abci app");
    {
        use futures::StreamExt;
        use tendermint::v0_37::abci::response::{Echo, Response};
        let response = response_stream.next().await.ok_or(anyhow!("stream ended unexpectedly"))?
            .ok(/*TODO: recover this error later*/).unwrap();
        let response = Response::try_from(response)?;
        match response {
            Response::Echo(Echo { message }) if message == MESSAGE => {}
            other => anyhow::bail!("unexpected response: {other:?}"),
        };
    };
    tracing::info!("received echo response from abci app");

    // Check that the connection is still alive, and clean up our temporary storage.
    assert!(
        !abci_connection.is_finished(),
        "abci connection is stil alive"
    );
    drop(storage);
    drop(guard);

    Ok(())
}
