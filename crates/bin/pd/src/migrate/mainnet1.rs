//! Migration for shipping consensus-breaking IBC changes, fixing
//! how withdrawals from Penumbra to Noble are handled, and ensures that IBC
//! error messages from counterparty chains are processed.
use cnidarium::Snapshot;
use cnidarium::{StateDelta, Storage};
use ibc_types::core::channel::{Packet, PortId};
use ibc_types::transfer::acknowledgement::TokenTransferAcknowledgement;
use jmt::RootHash;
use penumbra_sdk_app::app::StateReadExt as _;
use penumbra_sdk_app::app_version::migrate_app_version;
use penumbra_sdk_governance::StateWriteExt;
use penumbra_sdk_ibc::{component::ChannelStateWriteExt as _, IbcRelay};
use penumbra_sdk_sct::component::clock::EpochManager;
use penumbra_sdk_sct::component::clock::EpochRead;
use penumbra_sdk_transaction::{Action, Transaction};
use std::path::PathBuf;
use tracing::instrument;

use crate::network::generate::NetworkConfig;

/// The block where proposal #2 passed, enabling outbound ICS20 transfers.
const ICS20_TRANSFER_START_HEIGHT: u64 = 411616;

/// Find all of the lost transfers inside of a transaction.
///
/// In other words, look for relayed packet acknowledgements relating to ICS20 transfers containing an error.
/// These packets were not correctly handled, being deleted when the ack had an error,
/// as if the ack were successful.
fn tx_lost_transfers(transaction: Transaction) -> impl Iterator<Item = Packet> {
    transaction
        .transaction_body()
        .actions
        .into_iter()
        .filter_map(move |action| match action {
            Action::IbcRelay(IbcRelay::Acknowledgement(m)) => {
                // Make sure we're only looking at ICS20 related packets
                if m.packet.port_on_b != PortId::transfer() {
                    return None;
                }
                // This shouldn't fail to parse, because the transaction wouldn't have been
                // included otherwise, but if for some reason it doesn't, ignore it.
                let transfer: TokenTransferAcknowledgement =
                    match serde_json::from_slice(m.acknowledgement.as_slice()) {
                        Err(_) => return None,
                        Ok(x) => x,
                    };
                // If the ack was successful, then that packet was correctly handled, so don't
                // consider it.
                match transfer {
                    TokenTransferAcknowledgement::Success(_) => None,
                    TokenTransferAcknowledgement::Error(_) => Some(m.packet),
                }
            }
            _ => None,
        })
}

/// Retrieve all the packets resulting in a locked transfer because of error acks.
///
/// This does so by looking at all transactions, looking for the relayed acknowledgements.
async fn lost_transfers(state: &StateDelta<Snapshot>) -> anyhow::Result<Vec<Packet>> {
    let mut out = Vec::new();
    let end_height = state.get_block_height().await?;
    // We only need to start from the height where transfers were enabled via governance.
    for height in ICS20_TRANSFER_START_HEIGHT..=end_height {
        let transactions = state.transactions_by_height(height).await?.transactions;
        for tx in transactions.into_iter() {
            for lost in tx_lost_transfers(tx.try_into()?) {
                out.push(lost);
            }
        }
    }
    Ok(out)
}

/// Replace all the packets that were erroneously removed from the state.
async fn replace_lost_packets(delta: &mut StateDelta<Snapshot>) -> anyhow::Result<()> {
    let lost_packets = lost_transfers(delta).await?;
    for packet in lost_packets {
        // This will undo what happens in https://github.com/penumbra-zone/penumbra/blob/882a061bd69ce14b01711041bbc0c0ce209e2823/crates/core/component/ibc/src/component/msg_handler/acknowledgement.rs#L99.
        delta.put_packet_commitment(&packet);
    }
    Ok(())
}

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

        // Note, when this bit of code was added, the upgrade happened months ago,
        // and the version safeguard mechanism was not in place. However,
        // adding this will prevent someone running version 0.80.X with the
        // safeguard patch from accidentally running the migraton again, since they
        // will already have version 8 written into the state. But, if someone is syncing
        // up from genesis, then version 0.79 will not have written anything into the safeguard,
        // and this method will not complain. So, this addition provides a safeguard
        // for existing nodes, while also not impeding syncing up a node from scratch.
        migrate_app_version(&mut delta, 8).await?;

        // Reinsert all of the erroneously removed packets
        replace_lost_packets(&mut delta).await?;

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
    let app_state = penumbra_sdk_app::genesis::Content {
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
