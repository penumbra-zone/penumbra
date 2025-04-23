#![cfg(feature = "integration-testnet")]
//! Performs read-only operations against the public Penumbra testnet.
//! Useful to validate that HTTPS support is working.

use std::process::Command as StdCommand;

use anyhow::Context;
use assert_cmd::cargo::CommandCargoExt;
use assert_cmd::Command as AssertCommand;
use futures::StreamExt;
use tempfile::{tempdir, TempDir};
use tokio::process::Command as TokioCommand;

use penumbra_sdk_keys::test_keys;
use penumbra_sdk_proto::view::v1::view_service_client::ViewServiceClient;
use penumbra_sdk_view::ViewClient;

const NODE_URL: &str = "https://testnet.plinfra.net";

const PCLIENTD_BIND_ADDR: &str = "127.0.0.1:9081";

/// Build config for pclientd, supporting only read operations.
/// The custody portion is intentionally unset, to avoid side-effects
/// on the public testnet.
fn generate_readonly_config(home_dir: &TempDir) -> anyhow::Result<()> {
    let mut init_cmd = AssertCommand::cargo_bin("pclientd")?;
    init_cmd
        .args([
            "--home",
            home_dir.path().to_str().unwrap(),
            "init",
            "--bind-addr",
            "127.0.0.1:8081",
            "--grpc-url",
            NODE_URL,
            "--view",
        ])
        .write_stdin(test_keys::FULL_VIEWING_KEY.clone().to_string());
    init_cmd.assert().success();
    Ok(())
}

#[ignore]
#[tokio::test]
/// Start a pclientd process for the testnet wallet, and sync to current height
/// on the testnet. We don't perform any write actions, so there will be no on-chain
/// side-effects. Instead, we simply confirm that pclientd can talk to a remote
/// endpoint and understand the blocks it receives.
async fn pclientd_sync_against_testnet() -> anyhow::Result<()> {
    tracing_subscriber::fmt::try_init().ok();
    // Create a tempdir for the pclientd instance to run in.
    let data_dir = tempdir().unwrap();

    // 1. Construct a config for the `pclientd` instance:
    generate_readonly_config(&data_dir)?;

    // 2. Run a `pclientd` instance in the background as a subprocess.
    let home_dir = data_dir.path().to_owned();
    // Use a std Command so we can use the cargo-specific extensions from assert_cmd
    let mut pclientd_cmd = StdCommand::cargo_bin("pclientd")?;
    pclientd_cmd.args(["--home", home_dir.as_path().to_str().unwrap(), "start"]);

    // Convert to an async-aware Tokio command so we can spawn it in the background.
    let mut pclientd_cmd = TokioCommand::from(pclientd_cmd);
    // Important: without this, we could accidentally leave the pclientd instance running.
    pclientd_cmd.kill_on_drop(true);

    let mut pclientd = pclientd_cmd.spawn()?;

    // Wait for the newly spawned daemon to come up.
    tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
    if let Some(status) = pclientd.try_wait()? {
        // An error occurred during startup, probably.
        anyhow::bail!("pclientd exited early: {status:?}");
    }

    // 3. Build a client for the daemon we just started.
    let pclientd_url = format!("http://{}", PCLIENTD_BIND_ADDR);
    let channel = tonic::transport::Channel::from_shared(pclientd_url)?
        .connect()
        .await
        .context("failed to open channel to test instance of pclientd")?;
    let mut view_client = ViewServiceClient::new(channel.clone());
    // let mut custody_client = CustodyServiceClient::new(channel.clone());

    // 4. Use the view protocol to wait for it to sync.
    let mut status_stream = (&mut view_client as &mut dyn ViewClient)
        .status_stream()
        .await?;
    while let Some(item) = status_stream.as_mut().next().await.transpose()? {
        tracing::debug!(?item);
    }
    // We exit here, dropping the pclientd process handle.
    // We've verified that we can sync the wallet.
    Ok(())
}
