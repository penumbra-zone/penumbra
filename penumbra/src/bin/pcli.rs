use anyhow::Result;
use directories::ProjectDirs;
use rand_core::OsRng;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io, process};
use structopt::StructOpt;

use penumbra_wallet::{state, storage};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pcli",
    about = "The Penumbra command-line interface.", 
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {
    /// The address of the Tendermint node.
    #[structopt(short, long, default_value = "127.0.0.1:26657")]
    addr: std::net::SocketAddr,
    #[structopt(subcommand)]
    cmd: Command,
    /// The location of the wallet file.
    #[structopt(short, long)]
    wallet_location: Option<String>,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Creates a transaction.
    Tx { key: String, value: String },
    /// Queries the Penumbra state.
    Query { key: String },
    /// Generate keys.
    Generate,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    let project_dir =
        ProjectDirs::from("zone", "penumbra", "pcli").expect("can access penumbra project dir");
    // Currently we use just the data directory. Create it if it is missing.
    fs::create_dir_all(project_dir.data_dir()).expect("can create penumbra data directory");

    // We store wallet data in `penumbra_wallet.dat` in the state directory, unless
    // the user provides another location.
    let wallet_path: PathBuf;
    match opt.wallet_location {
        Some(path) => {
            wallet_path = Path::new(&path).to_path_buf();
        }
        None => {
            wallet_path = project_dir.data_dir().join("penumbra_wallet.dat");
        }
    }

    match opt.cmd {
        Command::Tx { key, value } => {
            let spend_key = load_existing_keys(&wallet_path);
            let _client = state::ClientState::new(spend_key);

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
            let spend_key = load_existing_keys(&wallet_path);
            let _client = state::ClientState::new(spend_key);

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
        Command::Generate => {
            let wallet = create_wallet(&wallet_path);
            let client = state::ClientState::new(wallet);
            println!("Your first address is {}", client.wallet.addresses[0]);
        }
    }

    Ok(())
}

/// Load existing keys from wallet file, printing an error if the file doesn't exist.
fn load_existing_keys(wallet_path: &Path) -> storage::Wallet {
    let wallet: storage::Wallet = match fs::read(wallet_path) {
        Ok(data) => bincode::deserialize(&data).expect("can deserialize wallet file"),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                eprintln!(
                    "error: key data not found, run `pcli generate` to generate Penumbra keys"
                );
                process::exit(1);
            }
            _ => {
                eprintln!("unknown error: {}", err);
                process::exit(2);
            }
        },
    };
    wallet
}

/// Create wallet file, ensuring existing wallets are not overwritten.
fn create_wallet(wallet_path: &Path) -> storage::Wallet {
    let wallet = storage::Wallet::generate(&mut OsRng);
    let mut file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(wallet_path)
    {
        Ok(file) => file,
        Err(err) => match err.kind() {
            io::ErrorKind::AlreadyExists => {
                eprintln!(
                    "error: wallet file already exists at {}",
                    wallet_path.display()
                );
                process::exit(3);
            }
            _ => {
                eprintln!("unknown error: {}", err);
                process::exit(2);
            }
        },
    };
    let seed_data = bincode::serialize(&wallet).expect("can serialize");
    file.write_all(&seed_data).expect("Unable to write file");
    println!(
        "Wallet generated, stored in {}. WARNING: This file contains your private keys. BACK UP THIS FILE!",
        wallet_path.display()
    );
    wallet
}
