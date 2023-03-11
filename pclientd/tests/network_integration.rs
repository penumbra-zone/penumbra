//! Basic integration testing of `pcli` versus a target testnet.
//!
//! Tests against the network in the `PENUMBRA_NODE_HOSTNAME` environment variable.
//!
//! Tests assume that the initial state of the test account is after genesis,
//! where no tokens have been delegated, and the address with index 0
//! was distributedp 1cube.

use assert_cmd::Command;
use tempfile::tempdir;

use futures::StreamExt;
use pclientd::PclientdConfig;
use penumbra_chain::test_keys;
use penumbra_custody::soft_kms;
use penumbra_proto::penumbra::view::v1alpha1::view_protocol_service_client::ViewProtocolServiceClient;
use penumbra_view::ViewClient;

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
    let handle = std::thread::spawn(move || {
        let mut pclientd_cmd = Command::cargo_bin("pclientd").unwrap();
        pclientd_cmd
            .args(["--home", home_dir.as_path().to_str().unwrap(), "start"])
            .assert()
            .success();

        ()
    });
    // Wait for the newly spawned daemon to come up.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    if handle.is_finished() {
        // An error occurred during startup, probably.
        handle.join().unwrap();
        return Err(anyhow::anyhow!("pclientd exited early"));
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
    if handle.is_finished() {
        handle.join().unwrap();
    }

    Ok(())
}
