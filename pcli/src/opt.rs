use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pcli",
    about = "The Penumbra command-line interface.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
pub struct Opt {
    /// The address of the Tendermint node.
    #[structopt(short, long, default_value = "127.0.0.1")]
    pub node: String,
    #[structopt(short, long, default_value = "26657")]
    pub abci_port: u16,
    #[structopt(long, default_value = "26666")]
    pub wallet_port: u16,
    #[structopt(subcommand)]
    pub cmd: Command,
    /// The location of the wallet file [default: platform appdata directory]
    #[structopt(short, long)]
    pub wallet_location: Option<String>,
}

// Note: can't use `Vec<u8>` directly, as structopt would instead look for
// conversion function from `&str` to `u8`.
type Bytes = Vec<u8>;

fn parse_bytestring(s: &str) -> Result<Vec<u8>, String> {
    let decoded = hex::decode(s).expect("Invalid bytestring");

    Ok(decoded)
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Creates a transaction.
    Tx(TxCmd),
    /// Queries the Penumbra state.
    #[structopt()]
    Query { key: String },
    /// Manages the wallet state.
    Wallet(WalletCmd),
    /// Manages addresses.
    Addr(AddrCmd),
    /// Synchronizes the chain state to the client.
    ///
    /// `pcli` syncs automatically prior to any action requiring chain state,
    /// but this command can be used to "pre-sync" before interactive use.
    Sync,
    /// Fetch transaction by note commitment - TEMP (developer only, remove when sync implemented)
    FetchByNoteCommitment { note_commitment: String },
    /// Asset Registry Lookup based on asset ID
    AssetLookup {
        #[structopt(parse(try_from_str = parse_bytestring))]
        asset_id: Bytes,
    },
    /// List every asset in the Asset Registry
    AssetList {},
    /// Displays current balance by asset.
    Balance,
}

#[derive(Debug, StructOpt)]
pub enum WalletCmd {
    /// Import an existing spend seed.
    Import,
    /// Generate a new spend seed.
    Generate,
    /// Delete the wallet permanently.
    Delete,
}

#[derive(Debug, StructOpt)]
pub enum AddrCmd {
    /// List addresses.
    List,
    /// Show the address with the given index.
    Show {
        /// The index of the address to show.
        #[structopt(short, long)]
        index: u32,
    },
    /// Create a new address.
    New {
        /// A freeform label for the address, stored only locally.
        label: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum TxCmd {
    /// Send transaction to the node.
    Send {
        /// Amount to send.
        amount: u64,
        /// Denomination.
        denomination: String,
        /// Destination address.
        address: String,
        /// Fee.
        fee: u64,
    },
}
