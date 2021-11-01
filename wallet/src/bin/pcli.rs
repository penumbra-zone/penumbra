use anyhow::Result;
use structopt::StructOpt;

use penumbra_crypto::{memo::MemoPlaintext, merkle, note, Note, Nullifier};
use std::collections::{BTreeMap, HashSet};

const MAX_MERKLE_CHECKPOINTS_CLIENT: usize = 10;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pcli",
    about = "The Penumbra command-line interface.", 
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {
    /// The address of the Tendermint node.
    #[structopt(short, long, default_value = "127.0.0.1:26658")]
    addr: std::net::SocketAddr,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Creates a transaction.
    Tx { key: String, value: String },
    /// Queries the Penumbra state.
    Query { key: String },
}

struct ClientState {
    // The first block height we've scanned from.
    first_block_height: i64,
    // The last block height we've scanned to.
    last_block_height: i64,
    // Note commitment tree.
    note_commitment_tree: merkle::BridgeTree<note::Commitment, { merkle::DEPTH as u8 }>,
    // Our nullifiers and the notes they correspond to.
    nullifier_map: BTreeMap<Nullifier, Note>,
    // Notes that we have received.
    received_set: HashSet<(Note, MemoPlaintext)>,
    // Notes that we have spent.
    spent_set: HashSet<Note>,
    // Transaction IDs we have visibility into.
    transactions: Vec<Vec<u8>>,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            first_block_height: 0,
            last_block_height: 0,
            note_commitment_tree: merkle::BridgeTree::new(MAX_MERKLE_CHECKPOINTS_CLIENT),
            nullifier_map: BTreeMap::new(),
            received_set: HashSet::new(),
            spent_set: HashSet::new(),
            transactions: Vec::new(),
        }
    }
}

impl ClientState {
    // TODO: For each output in scanned transactions, try to decrypt the note ciphertext.
    // If the note decrypts, we:
    // * add the (note plaintext, memo) to the received_set.
    // * compute and add the nullifier to the nullifier map.
    // * add the note commitment to the note commitment tree.
    // * witness the note commitment value.
    //
    // For each spend, if the revealed nf is in our nullifier_map, then
    // we add nullifier_map[nf] to spent_set.
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    // XXX probably good to move towards using the tendermint-rs RPC functionality

    match opt.cmd {
        Command::Tx { key, value } => {
            let rsp = reqwest::get(format!(
                r#"http://{}/broadcast_tx_async?tx="{}={}""#,
                opt.addr, key, value
            ))
            .await?
            .text()
            .await?;

            tracing::info!("{}", rsp);
        }
        Command::Query { key } => {
            let rsp: serde_json::Value = reqwest::get(format!(
                r#"http://{}/abci_query?data=0x{}"#,
                opt.addr,
                hex::encode(key),
            ))
            .await?
            .json()
            .await?;

            tracing::info!(?rsp);
        }
    }

    Ok(())
}
