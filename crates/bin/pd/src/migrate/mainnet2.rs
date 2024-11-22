//! Migration for shipping consensus-breaking IBC changes, fixing
//! how withdrawals from Penumbra to Noble are handled, and ensures that IBC
//! error messages from counterparty chains are processed.
use cnidarium::{StateDelta, Storage};
use jmt::RootHash;
use penumbra_app::app::StateReadExt as _;
use penumbra_app::app_version::migrate_app_version;
use penumbra_governance::StateWriteExt;
use penumbra_sct::component::clock::EpochManager;
use penumbra_sct::component::clock::EpochRead;
use std::path::PathBuf;
use tracing::instrument;

use crate::network::generate::NetworkConfig;

/// Run the full migration, emitting a new genesis event, representing historical state.
///
/// This will have the effect of reinserting packets which had acknowledgements containing
/// errors, and erroneously removed from state, as if the acknowledgements had contained successes.
#[instrument]
pub async fn migrate(
    storage: Storage,
    pd_home: PathBuf,
    genesis_start: Option<tendermint::time::Time>,
) -> anyhow::Result<()> {
    // Setup:
    let initial_state = storage.latest_snapshot();
    let chain_id = initial_state.get_chain_id().await?;
    let root_hash = initial_state
        .root_hash()
        .await
        .expect("chain state has a root hash");
    // We obtain the pre-upgrade hash solely to log it as a result.
    let pre_upgrade_root_hash: RootHash = root_hash.into();
    let pre_upgrade_height = initial_state
        .get_block_height()
        .await
        .expect("chain state has a block height");
    let post_upgrade_height = pre_upgrade_height.wrapping_add(1);

    let mut delta = StateDelta::new(initial_state);
    let (migration_duration, post_upgrade_root_hash) = {
        let start_time = std::time::SystemTime::now();

        migrate_app_version(&mut delta, 9).await?;

        // Reset the application height and halt flag.
        delta.ready_to_start();
        delta.put_block_height(0u64);

        // Finally, commit the changes to the chain state.
        let post_upgrade_root_hash = storage.commit_in_place(delta).await?;
        tracing::info!(?post_upgrade_root_hash, "post-migration root hash");

        (
            start_time.elapsed().expect("start is set"),
            post_upgrade_root_hash,
        )
    };
    storage.release().await;

    // The migration is complete, now we need to generate a genesis file. To do this, we need
    // to lookup a validator view from the chain, and specify the post-upgrade app hash and
    // initial height.
    let app_state = penumbra_app::genesis::Content {
        chain_id,
        ..Default::default()
    };
    let mut genesis = NetworkConfig::make_genesis(app_state.clone()).expect("can make genesis");
    genesis.app_hash = post_upgrade_root_hash
        .0
        .to_vec()
        .try_into()
        .expect("infallible conversion");

    genesis.initial_height = post_upgrade_height as i64;
    genesis.genesis_time = genesis_start.unwrap_or_else(|| {
        let now = tendermint::time::Time::now();
        tracing::info!(%now, "no genesis time provided, detecting a testing setup");
        now
    });
    let checkpoint = post_upgrade_root_hash.0.to_vec();
    let genesis = NetworkConfig::make_checkpoint(genesis, Some(checkpoint));
    let genesis_json = serde_json::to_string(&genesis).expect("can serialize genesis");
    tracing::info!("genesis: {}", genesis_json);
    let genesis_path = pd_home.join("genesis.json");
    std::fs::write(genesis_path, genesis_json).expect("can write genesis");

    let validator_state_path = pd_home.join("priv_validator_state.json");
    let fresh_validator_state = crate::network::generate::NetworkValidator::initial_state();
    std::fs::write(validator_state_path, fresh_validator_state).expect("can write validator state");

    tracing::info!(
        pre_upgrade_height,
        post_upgrade_height,
        ?pre_upgrade_root_hash,
        ?post_upgrade_root_hash,
        duration = migration_duration.as_secs(),
        "successful migration!"
    );

    Ok(())
}
