//! Basic integration testing of `pclientd` versus a target testnet.
//!
//! Tests against the network in the `PENUMBRA_NODE_PD_URL` environment variable.
//!
//! Tests assume that the initial state of the test account is after genesis,
//! where no tokens have been delegated, and the address with index 0
//! was distributedp 1cube.

use std::{ops::Deref, process::Command as StdCommand};

use anyhow::Context;
use assert_cmd::cargo::CommandCargoExt;
use assert_cmd::Command as AssertCommand;
use base64::prelude::*;
use futures::{FutureExt, StreamExt, TryStreamExt};
use tempfile::{tempdir, TempDir};
use tokio::process::Command as TokioCommand;

use penumbra_sdk_asset::{asset, Value, STAKING_TOKEN_ASSET_ID};
use penumbra_sdk_keys::test_keys;
use penumbra_sdk_proto::{
    core::{component::fee::v1::Fee, component::ibc::v1::IbcRelay},
    custody::v1::{custody_service_client::CustodyServiceClient, AuthorizeRequest},
    penumbra::view::v1::view_service_client::ViewServiceClient,
    view::v1::{
        broadcast_transaction_response::Status as BroadcastStatus,
        witness_and_build_response::Status as WitnessAndBuildStatus, BroadcastTransactionRequest,
        TransactionPlannerRequest, WitnessAndBuildRequest,
    },
};
use penumbra_sdk_view::ViewClient;

// Generate a working pclientd config in the target directory.
fn generate_custody_config(home_dir: &TempDir) -> anyhow::Result<()> {
    let mut init_cmd = AssertCommand::cargo_bin("pclientd")?;
    init_cmd
        .args([
            "--home",
            home_dir.path().to_str().unwrap(),
            "init",
            "--bind-addr",
            "127.0.0.1:8081",
            "--grpc-url",
            std::env::var("PENUMBRA_NODE_PD_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8080".to_owned())
                .as_str(),
        ])
        .write_stdin(test_keys::SEED_PHRASE.to_string());
    init_cmd.assert().success();
    Ok(())
}

#[ignore]
#[tokio::test]
async fn transaction_send_flow() -> anyhow::Result<()> {
    tracing_subscriber::fmt::try_init().ok();
    // Create a tempdir for the pclientd instance to run in.
    let data_dir = tempdir().unwrap();

    // 1. Construct a config for the `pclientd` instance:
    generate_custody_config(&data_dir)?;

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
        anyhow::bail!("pclientd exited early: {status:?}");
    }

    // 3. Build a client for the daemon we just started.
    let channel = tonic::transport::Channel::from_static("http://127.0.0.1:8081")
        .connect()
        .await?;
    let mut view_client = ViewServiceClient::new(channel.clone());
    let mut custody_client = CustodyServiceClient::new(channel.clone());

    // 4. Use the view protocol to wait for it to sync.
    let mut status_stream = (&mut view_client as &mut dyn ViewClient)
        .status_stream()
        .await?;
    while let Some(item) = status_stream.as_mut().next().await.transpose()? {
        tracing::debug!(?item);
    }

    // 5. Try building a transaction using the simplified flow.
    // Here we don't want to use the Penumbra Rust libraries much, because
    // we're executing as if we were a Go program that had to construct all these
    // protos manually, with no access to Penumbra crypto.
    use penumbra_sdk_proto::view::v1::transaction_planner_request as tpr;

    // Specifically, pretend we're relaying IBC messages, so pull one in:

    // base64 encoded MsgCreateClient that was used to create the currently in-use Stargaze
    // light client on the cosmos hub:
    // https://cosmos.bigdipper.live/transactions/13C1ECC54F088473E2925AD497DDCC092101ADE420BC64BADE67D34A75769CE9
    let msg_create_client_stargaze_raw = BASE64_STANDARD
        .decode(
            include_str!("../../../core/component/ibc/src/component/test/create_client.msg")
                .replace('\n', ""),
        )
        .unwrap();
    use ibc_types::core::client::msgs::MsgCreateClient;
    use ibc_types::DomainType;
    let msg_create_stargaze_client =
        MsgCreateClient::decode(msg_create_client_stargaze_raw.as_slice()).unwrap();
    let create_client_action: IbcRelay = msg_create_stargaze_client.into();

    // 5.1. Generate a transaction plan sending funds to an address.
    let plan = view_client
        .transaction_planner(TransactionPlannerRequest {
            outputs: vec![tpr::Output {
                address: Some(test_keys::ADDRESS_1.deref().clone().into()),
                value: Some(
                    Value {
                        amount: 1_000_000u64.into(),
                        asset_id: *STAKING_TOKEN_ASSET_ID,
                    }
                    .into(),
                ),
            }],
            ibc_relay_actions: vec![create_client_action],
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
        })
        .await?
        .into_inner()
        .data
        .ok_or_else(|| anyhow::anyhow!("AuthorizeResponse missing data"))?;

    // 5.3. Have pclientd build and sign the planned transaction.
    let mut tx_rsp = view_client
        .witness_and_build(WitnessAndBuildRequest {
            transaction_plan: Some(plan),
            authorization_data: Some(auth_data),
        })
        .await?
        .into_inner();
    let tx = (async move {
        while let Some(tx_rsp) = tx_rsp.try_next().await? {
            match tx_rsp.status {
                Some(status) => match status {
                    WitnessAndBuildStatus::BuildProgress(_) => {}
                    WitnessAndBuildStatus::Complete(c) => {
                        return c.transaction.ok_or_else(|| {
                            anyhow::anyhow!("WitnessAndBuildResponse missing transaction")
                        });
                    }
                },
                None => {
                    // No status is unexpected behavior
                    return Err(anyhow::anyhow!(
                        "empty BroadcastTransactionResponse message"
                    ));
                }
            }
        }
        Err(anyhow::anyhow!("no witness and build response"))
    }
    .boxed())
    .await
    .context("error building transaction")?;

    // 5.4. Have pclientd broadcast and await confirmation of the built transaction.
    let mut broadcast_rsp = view_client
        .broadcast_transaction(BroadcastTransactionRequest {
            transaction: Some(tx),
            await_detection: true,
        })
        .await?
        .into_inner();
    let tx_id = (async move {
        while let Some(broadcast_rsp) = broadcast_rsp.try_next().await? {
            match broadcast_rsp.status {
                Some(status) => match status {
                    BroadcastStatus::BroadcastSuccess(_) => {}
                    BroadcastStatus::Confirmed(c) => {
                        println!("transaction confirmed");
                        return c.id.ok_or_else(|| {
                            anyhow::anyhow!("WitnessAndBuildResponse missing transaction")
                        });
                    }
                },
                None => {
                    // No status is unexpected behavior
                    return Err(anyhow::anyhow!(
                        "empty BroadcastTransactionResponse message"
                    ));
                }
            }
        }
        Err(anyhow::anyhow!("no witness and build response"))
    }
    .boxed())
    .await
    .context("error building transaction")?;

    tracing::debug!(?tx_id);

    // Last, check that we didn't have any errors:
    if let Some(status) = pclientd.try_wait()? {
        // An error occurred during startup, probably.
        anyhow::bail!("pclientd errored: {status:?}");
    }
    pclientd.kill().await?;

    Ok(())
}

#[ignore]
#[tokio::test]
async fn swap_claim_flow() -> anyhow::Result<()> {
    tracing_subscriber::fmt::try_init().ok();
    // Create a tempdir for the pclientd instance to run in.
    let data_dir = tempdir().unwrap();

    // 1. Construct a config for the `pclientd` instance:
    generate_custody_config(&data_dir)?;

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
        anyhow::bail!("pclientd exited early: {status:?}");
    }

    // 3. Build a client for the daemon we just started.
    let channel = tonic::transport::Channel::from_static("http://127.0.0.1:8081")
        .connect()
        .await?;
    let mut view_client = ViewServiceClient::new(channel.clone());
    let mut custody_client = CustodyServiceClient::new(channel.clone());

    // 4. Use the view protocol to wait for it to sync.
    let mut status_stream = (&mut view_client as &mut dyn ViewClient)
        .status_stream()
        .await?;
    while let Some(item) = status_stream.as_mut().next().await.transpose()? {
        tracing::debug!(?item);
    }

    // 5. Try building a transaction using the simplified flow.
    // Here we don't want to use the Penumbra Rust libraries much, because
    // we're executing as if we were a Go program that had to construct all these
    // protos manually, with no access to Penumbra crypto.
    use penumbra_sdk_proto::core::num::v1 as num;
    use penumbra_sdk_proto::view::v1::transaction_planner_request as tpr;

    // 5.1. Generate a transaction plan performing a swap. Since there are no liquidity positions
    // on this test network, we'll expect to get all our inputs back.
    let gm = asset::Cache::with_known_assets()
        .get_unit("gm")
        .unwrap()
        .id();

    let plan = view_client
        .transaction_planner(TransactionPlannerRequest {
            swaps: vec![tpr::Swap {
                value: Some(
                    Value {
                        amount: 1_000u64.into(),
                        asset_id: *STAKING_TOKEN_ASSET_ID,
                    }
                    .into(),
                ),
                target_asset: Some(gm.into()),
                fee: Some(Fee {
                    amount: Some(num::Amount { lo: 0, hi: 0 }),
                    asset_id: None,
                }),
                claim_address: Some(test_keys::ADDRESS_1.deref().clone().into()),
            }],
            ..Default::default()
        })
        .await?
        .into_inner()
        .plan
        .ok_or_else(|| anyhow::anyhow!("TransactionPlannerResponse missing plan"))?;

    // Hold on to the swap plaintext to be able to claim.
    let swap_plaintext =
        TryInto::<penumbra_sdk_transaction::TransactionPlan>::try_into(plan.clone())?
            .swap_plans()
            .next()
            .expect("swap plan must be present")
            .swap_plaintext
            .clone();

    // 5.2. Get authorization data for the transaction from pclientd (signing).
    let auth_data = custody_client
        .authorize(AuthorizeRequest {
            plan: Some(plan.clone()),
            pre_authorizations: Vec::new(),
        })
        .await?
        .into_inner()
        .data
        .ok_or_else(|| anyhow::anyhow!("AuthorizeResponse missing data"))?;

    // 5.3. Have pclientd build and sign the planned transaction.
    let mut tx_rsp = view_client
        .witness_and_build(WitnessAndBuildRequest {
            transaction_plan: Some(plan),
            authorization_data: Some(auth_data),
        })
        .await?
        .into_inner();
    let tx = (async move {
        while let Some(tx_rsp) = tx_rsp.try_next().await? {
            match tx_rsp.status {
                Some(status) => match status {
                    WitnessAndBuildStatus::BuildProgress(_) => {}
                    WitnessAndBuildStatus::Complete(c) => {
                        return c.transaction.ok_or_else(|| {
                            anyhow::anyhow!("WitnessAndBuildResponse missing transaction")
                        });
                    }
                },
                None => {
                    // No status is unexpected behavior
                    return Err(anyhow::anyhow!(
                        "empty BroadcastTransactionResponse message"
                    ));
                }
            }
        }
        Err(anyhow::anyhow!("no witness and build response"))
    }
    .boxed())
    .await
    .context("error building transaction")?;

    // 5.4. Have pclientd broadcast and await confirmation of the built transaction.
    let mut broadcast_rsp = view_client
        .broadcast_transaction(BroadcastTransactionRequest {
            transaction: Some(tx),
            await_detection: true,
        })
        .await?
        .into_inner();
    let tx_id = (async move {
        while let Some(broadcast_rsp) = broadcast_rsp.try_next().await? {
            match broadcast_rsp.status {
                Some(status) => match status {
                    BroadcastStatus::BroadcastSuccess(bs) => {
                        println!("broadcast success, tx id: {:#?}", bs);
                    }
                    BroadcastStatus::Confirmed(c) => {
                        println!("transaction confirmed");
                        return c.id.ok_or_else(|| {
                            anyhow::anyhow!("WitnessAndBuildResponse missing transaction")
                        });
                    }
                },
                None => {
                    // No status is unexpected behavior
                    return Err(anyhow::anyhow!(
                        "empty BroadcastTransactionResponse message"
                    ));
                }
            }
        }
        Err(anyhow::anyhow!("no witness and build response"))
    }
    .boxed())
    .await
    .context("error building transaction")?;

    tracing::debug!(?tx_id);

    // Check that we didn't have any errors:
    if let Some(status) = pclientd.try_wait()? {
        // An error occurred during startup, probably.
        anyhow::bail!("pclientd errored: {status:?}");
    }

    // 6. Use the view protocol to wait for it to sync.
    let mut status_stream = (&mut view_client as &mut dyn ViewClient)
        .status_stream()
        .await?;
    while let Some(item) = status_stream.as_mut().next().await.transpose()? {
        tracing::debug!(?item);
    }

    // Ensure we can fetch the SwapRecord with the claimable swap.
    let _swap_record = (&mut view_client as &mut dyn ViewClient)
        .swap_by_commitment(swap_plaintext.swap_commitment())
        .await?;

    // 7. Prepare the swap claim
    let plan = view_client
        .transaction_planner(TransactionPlannerRequest {
            swap_claims: vec![tpr::SwapClaim {
                swap_commitment: Some(swap_plaintext.swap_commitment().into()),
            }],
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
        })
        .await?
        .into_inner()
        .data
        .ok_or_else(|| anyhow::anyhow!("AuthorizeResponse missing data"))?;

    // 5.3. Have pclientd build and sign the planned transaction.
    let mut tx_rsp = view_client
        .witness_and_build(WitnessAndBuildRequest {
            transaction_plan: Some(plan),
            authorization_data: Some(auth_data),
        })
        .await?
        .into_inner();
    let tx = (async move {
        while let Some(tx_rsp) = tx_rsp.try_next().await? {
            match tx_rsp.status {
                Some(status) => match status {
                    WitnessAndBuildStatus::BuildProgress(_) => {}
                    WitnessAndBuildStatus::Complete(c) => {
                        return c.transaction.ok_or_else(|| {
                            anyhow::anyhow!("WitnessAndBuildResponse missing transaction")
                        });
                    }
                },
                None => {
                    // No status is unexpected behavior
                    return Err(anyhow::anyhow!(
                        "empty BroadcastTransactionResponse message"
                    ));
                }
            }
        }
        Err(anyhow::anyhow!("no witness and build response"))
    }
    .boxed())
    .await
    .context("error building transaction")?;

    // 5.4. Have pclientd broadcast and await confirmation of the built transaction.
    let mut broadcast_rsp = view_client
        .broadcast_transaction(BroadcastTransactionRequest {
            transaction: Some(tx),
            await_detection: true,
        })
        .await?
        .into_inner();
    let tx_id = (async move {
        while let Some(broadcast_rsp) = broadcast_rsp.try_next().await? {
            match broadcast_rsp.status {
                Some(status) => match status {
                    BroadcastStatus::BroadcastSuccess(_) => {}
                    BroadcastStatus::Confirmed(c) => {
                        println!("transaction confirmed");
                        return c.id.ok_or_else(|| {
                            anyhow::anyhow!("WitnessAndBuildResponse missing transaction")
                        });
                    }
                },
                None => {
                    // No status is unexpected behavior
                    return Err(anyhow::anyhow!(
                        "empty BroadcastTransactionResponse message"
                    ));
                }
            }
        }
        Err(anyhow::anyhow!("no witness and build response"))
    }
    .boxed())
    .await
    .context("error building transaction")?;

    tracing::debug!(?tx_id);

    // Check that we didn't have any errors:
    if let Some(status) = pclientd.try_wait()? {
        // An error occurred during startup, probably.
        anyhow::bail!("pclientd errored: {status:?}");
    }
    pclientd.kill().await?;

    Ok(())
}
