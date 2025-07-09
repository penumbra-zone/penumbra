#![cfg(feature = "network-integration")]
//! Integration integration testing of `pmonitor` against a local devnet.
//! Sets up various scenarios of genesis allocations, and ensures the tool reports
//! violations as errors.
//!
//! As a convenience to developers, there's a commented-out `sleep` call in the
//! `audit_passes_on_compliant_wallets` test. If enabled, the setup testbed can be interacted with
//! manually, which helps when trying to diagnose behavior of the tool.
use anyhow::Context;
use assert_cmd::Command as AssertCommand;
use pcli::config::PcliConfig;
mod common;
use crate::common::pcli_helpers::{pcli_init_softkms, pcli_migrate_balance, pcli_view_address};
use crate::common::PmonitorTestRunner;

#[tokio::test]
/// Tests the simplest happy path for pmonitor: all wallets have genesis balances,
/// they never transferred any funds out, nor migrated balances, so all
/// current balances equal the genesis balances. In this case `pmonitor`
/// should exit 0.
async fn audit_passes_on_compliant_wallets() -> anyhow::Result<()> {
    tracing_subscriber::fmt::try_init().ok();
    let p = PmonitorTestRunner::new();
    p.create_pcli_wallets()?;
    let _network = p.start_devnet().await?;
    p.initialize_pmonitor()?;

    // Debugging: uncomment the sleep line below if you want to interact with the pmonitor testbed
    // that was set up already. Use e.g.:
    //
    //   cargo run --bin pmonitor -- --home /tmp/pmonitor-integration-test/pmonitor audit
    //
    // to view the output locally.
    //
    // std::thread::sleep(std::time::Duration::from_secs(3600));

    p.pmonitor_audit()?;
    Ok(())
}

#[tokio::test]
/// Tests another happy path for pmonitor: all wallets have genesis balances,
/// one of the wallets ran `pcli migrate balance` once. This means that all
/// wallets still have their genesis balance, save one, which has the genesis
/// balance minus gas fees. In this case, `pmonitor` should exit 0,
/// because it understood the balance migration and updated the FVK.
async fn audit_passes_on_wallets_that_migrated_once() -> anyhow::Result<()> {
    let p = PmonitorTestRunner::new();
    p.create_pcli_wallets()?;
    let _network = p.start_devnet().await?;
    // Run audit once, to confirm compliance on clean slate.
    p.initialize_pmonitor()?;
    p.pmonitor_audit()?;

    // Create an empty wallet, with no genesis funds, to which we'll migrate a balance.
    let alice_pcli_home = p.wallets_dir()?.join("wallet-alice");
    pcli_init_softkms(&alice_pcli_home)?;
    let alice_pcli_config = PcliConfig::load(
        alice_pcli_home
            .join("config.toml")
            .to_str()
            .expect("failed to convert alice wallet to str"),
    )?;

    // Take the second wallet, and migrate its balance to Alice.
    let migrated_wallet = p.wallets_dir()?.join("wallet-1");
    pcli_migrate_balance(&migrated_wallet, &alice_pcli_config.full_viewing_key)?;

    // Now re-run the audit tool: it should report OK again, because all we did was migrate.
    p.pmonitor_audit()?;
    Ok(())
}

#[tokio::test]
/// Tests another happy path for pmonitor: all wallets have genesis balances,
/// one of the wallets ran `pcli migrate balance` once, then that receiving
/// wallet ran `pcli migrate balance` itself, so the genesis funds are now
/// two (2) FVKs away from the original account. In this case,
/// `pmonitor` should exit 0, because it understood all balance migrations
/// and updated the FVK in its config file accordingly.
async fn audit_passes_on_wallets_that_migrated_twice() -> anyhow::Result<()> {
    let p = PmonitorTestRunner::new();
    p.create_pcli_wallets()?;
    let _network = p.start_devnet().await?;
    // Run audit once, to confirm compliance on clean slate.
    p.initialize_pmonitor()?;
    p.pmonitor_audit()
        .context("failed unexpectedly during initial audit run")?;

    // Create an empty wallet, with no genesis funds, to which we'll migrate a balance.
    let alice_pcli_home = p.wallets_dir()?.join("wallet-alice");
    pcli_init_softkms(&alice_pcli_home)?;
    let alice_pcli_config = PcliConfig::load(
        alice_pcli_home
            .join("config.toml")
            .to_str()
            .expect("failed to convert alice wallet to str"),
    )?;

    // Take the second wallet, and migrate its balance to Alice.
    let migrated_wallet = p.wallets_dir()?.join("wallet-1");
    pcli_migrate_balance(&migrated_wallet, &alice_pcli_config.full_viewing_key)?;

    // Now re-run the audit tool: it should report OK again, because all we did was migrate.
    p.pmonitor_audit()
        .context("failed unexpectedly during second audit run")?;

    // Create another empty wallet, with no genesis funds, to which we'll migrate a balance.
    let bob_pcli_home = p.wallets_dir()?.join("wallet-bob");
    pcli_init_softkms(&bob_pcli_home)?;
    let bob_pcli_config = PcliConfig::load(
        bob_pcli_home
            .join("config.toml")
            .to_str()
            .expect("failed to convert bob wallet to str"),
    )?;

    // Re-migrate the balance from Alice to Bob.
    pcli_migrate_balance(&alice_pcli_home, &bob_pcli_config.full_viewing_key)?;

    // Now re-run the audit tool: it should report OK again, confirming that it
    // successfully tracks multiple migratrions.
    p.pmonitor_audit()
        .context("failed unexpectedly during final audit run in test")?;

    Ok(())
}

#[tokio::test]
/// Tests an unhappy path for `pmonitor`: a single wallet has sent all its funds
/// to non-genesis account, via `pcli tx send` rather than `pcli migrate balance`.
/// In this case, `pmonitor` should exit non-zero.
async fn audit_fails_on_misbehaving_wallet_that_sent_funds() -> anyhow::Result<()> {
    let p = PmonitorTestRunner::new();
    p.create_pcli_wallets()?;
    let _network = p.start_devnet().await?;
    // Run audit once, to confirm compliance on clean slate.
    p.initialize_pmonitor()?;
    p.pmonitor_audit()?;

    // Create an empty wallet, with no genesis funds, to which we'll
    // manually send balance.
    let alice_pcli_home = p.wallets_dir()?.join("wallet-alice");
    pcli_init_softkms(&alice_pcli_home)?;

    let alice_address = pcli_view_address(&alice_pcli_home)?;

    // Take the second wallet, and send most of its funds of staking tokens to Alice.
    let misbehaving_wallet = p.wallets_dir()?.join("wallet-1");

    let send_cmd = AssertCommand::cargo_bin("pcli")?
        .args([
            "--home",
            misbehaving_wallet.to_str().unwrap(),
            "tx",
            "send",
            "--to",
            &alice_address.to_string(),
            "900penumbra",
        ])
        .output()
        .expect("failed to execute sending tx to alice wallet");
    assert!(send_cmd.status.success(), "failed to send tx to alice");

    // Now re-run the audit tool: it should report failure, via a non-zero exit code,
    // because of the missing funds.
    let result = p.pmonitor_audit();
    assert!(
        result.is_err(),
        "expected pmonitor to fail due to missing funds"
    );
    Ok(())
}

#[tokio::test]
/// Tests a happy path for `pmonitor`: a single wallet has sent all its funds
/// to non-genesis account, via `pcli tx send` rather than `pcli migrate balance`,
/// but the receiving wallet then sent those funds back.
/// In this case, `pmonitor` should exit zero.
async fn audit_passes_on_misbehaving_wallet_that_sent_funds_but_got_them_back() -> anyhow::Result<()>
{
    tracing_subscriber::fmt::try_init().ok();
    let p = PmonitorTestRunner::new();
    p.create_pcli_wallets()?;
    let _network = p.start_devnet().await?;
    // Run audit once, to confirm compliance on clean slate.
    p.initialize_pmonitor()?;
    p.pmonitor_audit()?;

    // Create an empty wallet, with no genesis funds, to which we'll
    // manually send balance.
    let alice_pcli_home = p.wallets_dir()?.join("wallet-alice");
    pcli_init_softkms(&alice_pcli_home)?;

    let alice_address = pcli_view_address(&alice_pcli_home)?;

    // Take the second wallet, and send most of its funds of staking tokens to Alice.
    let misbehaving_wallet = p.wallets_dir()?.join("wallet-1");

    let send_cmd = AssertCommand::cargo_bin("pcli")?
        .args([
            "--home",
            misbehaving_wallet.to_str().unwrap(),
            "tx",
            "send",
            "--to",
            &alice_address.to_string(),
            "900penumbra",
        ])
        .output()
        .expect("failed to execute sending tx to alice wallet");
    assert!(send_cmd.status.success(), "failed to send tx to alice");

    // The audit tool detects this state as a failure, since funds are missing.
    let result = p.pmonitor_audit();
    assert!(
        result.is_err(),
        "expected pmonitor to fail due to missing funds"
    );

    // Send the funds from alice back to the misbehaving wallet.
    let misbehaving_address = pcli_view_address(&misbehaving_wallet)?;
    let refund_cmd = AssertCommand::cargo_bin("pcli")?
        .args([
            "--home",
            alice_pcli_home.to_str().unwrap(),
            "tx",
            "send",
            "--to",
            &misbehaving_address.to_string(),
            // We intentionally specify a bit less than we received, to account for gas.
            "899.99penumbra",
        ])
        .output()
        .expect("failed to execute refund tx from alice wallet");
    assert!(
        refund_cmd.status.success(),
        "failed to send refund tx from alice"
    );

    // The audit tool detects this state as compliant again, because the funds were returned.
    p.pmonitor_audit()?;

    Ok(())
}
