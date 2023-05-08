//! Basic integration testing of `pclientd` versus a target testnet.
//!
//! Tests against the network in the `PENUMBRA_NODE_PD_URL` environment variable.
//!
//! Tests assume that the initial state of the test account is after genesis,
//! where no tokens have been delegated, and the address with index 0
//! was distributedp 1cube.

use assert_cmd::cargo::CommandCargoExt;
use futures::StreamExt;
use pclientd::PclientdConfig;
use penumbra_chain::test_keys;
use penumbra_custody::soft_kms;
use penumbra_proto::{
    core::ibc::v1alpha1::IbcAction,
    custody::v1alpha1::{
        custody_protocol_service_client::CustodyProtocolServiceClient, AuthorizeRequest,
    },
    penumbra::view::v1alpha1::view_protocol_service_client::ViewProtocolServiceClient,
    view::v1alpha1::{
        BroadcastTransactionRequest, TransactionPlannerRequest, WitnessAndBuildRequest,
    },
};
use penumbra_view::ViewClient;
use std::process::Command as StdCommand;
use tempfile::tempdir;
use tokio::process::Command as TokioCommand;

#[ignore]
#[tokio::test]
async fn transaction_send_flow() -> anyhow::Result<()> {
    use ibc_proto::protobuf::Protobuf;

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
    let channel = tonic::transport::Channel::from_static("http://127.0.0.1:8081")
        .connect()
        .await?;
    let mut view_client = ViewProtocolServiceClient::new(channel.clone());
    let mut custody_client = CustodyProtocolServiceClient::new(channel.clone());

    // 4. Use the view protocol to wait for it to sync.
    let mut status_stream = (&mut view_client as &mut dyn ViewClient)
        .status_stream(test_keys::FULL_VIEWING_KEY.account_group_id())
        .await?;
    while let Some(item) = status_stream.as_mut().next().await.transpose()? {
        tracing::debug!(?item);
    }

    // 5. Try building a transaction using the simplified flow.
    // Here we don't want to use the Penumbra Rust libraries much, because
    // we're executing as if we were a Go program that had to construct all these
    // protos manually, with no access to Penumbra crypto.
    use penumbra_proto::view::v1alpha1::transaction_planner_request as tpr;

    // Specifically, pretend we're relaying IBC messages, so pull one in:

    // base64 encoded MsgCreateClient that was used to create the currently in-use Stargaze
    // light client on the cosmos hub:
    // https://cosmos.bigdipper.live/transactions/13C1ECC54F088473E2925AD497DDCC092101ADE420BC64BADE67D34A75769CE9
    let msg_create_client_stargaze_raw = base64::decode(
        include_str!("../../crates/ibc/src/component/test/create_client.msg").replace('\n', ""),
    )
    .unwrap();
    use ibc_proto::protobuf::Protobuf;
    use ibc_types::core::ics02_client::msgs::create_client::MsgCreateClient;
    let msg_create_stargaze_client =
        MsgCreateClient::decode(msg_create_client_stargaze_raw.as_slice()).unwrap();
    let create_client_action: IbcAction = msg_create_stargaze_client.into();

    // 5.1. Generate a transaction plan sending funds to an address.
    let plan = view_client
        .transaction_planner(TransactionPlannerRequest {
            outputs: vec![tpr::Output {
                address: Some(test_keys::ADDRESS_1.clone().into()),
                value: Some(
                    penumbra_crypto::Value {
                        amount: 1_000_000u64.into(),
                        asset_id: penumbra_crypto::STAKING_TOKEN_ASSET_ID.clone(),
                    }
                    .into(),
                ),
            }],
            ibc_actions: vec![create_client_action],
            ..Default::default()
        })
        .await?
        .into_inner()
        .plan
        .ok_or_else(|| anyhow::anyhow!("TransactionPlannerResponse missing plan"))?;

    // 5.2. Get authorization data for the transaction from pclientd (signing).
    let auth_data = custody_client
        .authorize(AuthorizeRequest {
            plan: Some(plan.clone()),
            pre_authorizations: Vec::new(),
            ..Default::default()
        })
        .await?
        .into_inner()
        .data
        .ok_or_else(|| anyhow::anyhow!("AuthorizeResponse missing data"))?;

    // 5.3. Have pclientd build and sign the planned transaction.
    let tx = view_client
        .witness_and_build(WitnessAndBuildRequest {
            transaction_plan: Some(plan),
            authorization_data: Some(auth_data),
        })
        .await?
        .into_inner()
        .transaction
        .ok_or_else(|| anyhow::anyhow!("WitnessAndBuildResponse missing transaction"))?;

    // 5.4. Have pclientd broadcast and await confirmation of the built transaction.
    let tx_id = view_client
        .broadcast_transaction(BroadcastTransactionRequest {
            transaction: Some(tx),
            await_detection: true,
        })
        .await?
        .into_inner()
        .id
        .ok_or_else(|| anyhow::anyhow!("BroadcastTransactionRequest missing id"))?;

    tracing::debug!(?tx_id);

    // Last, check that we didn't have any errors:
    if let Some(status) = pclientd.try_wait()? {
        // An error occurred during startup, probably.
        return Err(anyhow::anyhow!("pclientd errored: {status:?}"));
    }
    pclientd.kill().await?;

    Ok(())
}
