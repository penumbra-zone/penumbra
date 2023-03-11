//! Basic integration testing of `pcli` versus a target testnet.
//!
//! Tests against the network in the `PENUMBRA_NODE_HOSTNAME` environment variable.
//!
//! Tests assume that the initial state of the test account is after genesis,
//! where no tokens have been delegated, and the address with index 0
//! was distributedp 1cube.

use assert_cmd::cargo::CommandCargoExt;
use futures::StreamExt;
use pclientd::PclientdConfig;
use penumbra_chain::test_keys;
use penumbra_custody::soft_kms;
use penumbra_proto::penumbra::view::v1alpha1::view_protocol_service_client::ViewProtocolServiceClient;
use penumbra_view::ViewClient;
use std::process::Command as StdCommand;
use tempfile::tempdir;
use tokio::process::Command as TokioCommand;

#[ignore]
#[tokio::test]
async fn transaction_send_flow() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    // Create a tempdir for the pclientd instance to run in.
    let data_dir = tempdir().unwrap();

    // 1. Construct a config for the `pclientd` instance:
    let config = PclientdConfig {
        fvk: test_keys::FULL_VIEWING_KEY.clone(),
        kms_config: Some(soft_kms::Config {
            spend_key: test_keys::SPEND_KEY.clone(),
            auth_policy: Vec::new(),
        }),
    };

    let mut config_file_path = data_dir.path().to_owned();
    config_file_path.push("config.toml");
    config.save(&config_file_path)?;

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
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    if let Some(status) = pclientd.try_wait()? {
        // An error occurred during startup, probably.
        return Err(anyhow::anyhow!("pclientd exited early: {status:?}"));
    }

    // 3. Build a client for the daemon we just started.
    let mut client = ViewProtocolServiceClient::connect("http://127.0.0.1:8081").await?;

    // 4. Use the view protocol to wait for it to sync.
    let mut status_stream = (&mut client as &mut dyn ViewClient)
        .status_stream(test_keys::FULL_VIEWING_KEY.account_id())
        .await?;
    while let Some(item) = status_stream.as_mut().next().await.transpose()? {
        tracing::debug!(?item);
    }

    // 5.

    // Last, check that we didn't have any errors:
    if let Some(status) = pclientd.try_wait()? {
        // An error occurred during startup, probably.
        return Err(anyhow::anyhow!("pclientd errored: {status:?}"));
    }
    pclientd.kill().await?;

    Ok(())
}
