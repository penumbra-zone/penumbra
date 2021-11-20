use anyhow::Result;
use directories::ProjectDirs;
use rand_core::OsRng;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io, process};
use structopt::StructOpt;

use penumbra_proto::wallet::{
    wallet_client::WalletClient, AssetListRequest, AssetLookupRequest, CompactBlockRangeRequest,
    TransactionByNoteRequest,
};
use penumbra_wallet::{ClientState, Wallet};

pub mod opt;
pub mod warning;

use opt::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Display a warning message to the user so they don't get upset when all their tokens are lost.
    warning::display();

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
        Command::Tx(TxCmd::Send {
            amount: _,
            denomination: _,
            address: _,
            fee,
        }) => {
            let spend_key = load_wallet(&wallet_path);
            let mut local_storage = ClientState::new(spend_key);

            let dummy_tx = local_storage.new_transaction(&mut OsRng, fee)?;
            let serialized_tx: Vec<u8> = dummy_tx.into();

            let rsp = reqwest::get(format!(
                r#"http://{}:{}/broadcast_tx_async?tx=0x{}"#,
                opt.node,
                opt.abci_port,
                hex::encode(serialized_tx)
            ))
            .await?
            .text()
            .await?;

            tracing::info!("{}", rsp);
        }
        Command::Query { key } => {
            let spend_key = load_wallet(&wallet_path);
            let _local_storage = ClientState::new(spend_key);

            // TODO: get working as part of issue 22
            let rsp: serde_json::Value = reqwest::get(format!(
                r#"http://{}:{}/abci_query?data=0x{}"#,
                opt.node,
                opt.abci_port,
                hex::encode(key),
            ))
            .await?
            .json()
            .await?;

            tracing::info!(?rsp);
        }
        Command::Wallet(WalletCmd::Generate) => {
            if wallet_path.exists() {
                return Err(anyhow::anyhow!(
                    "Wallet path {} already exists, refusing to overwrite it",
                    wallet_path.display()
                ));
            }
            let wallet = Wallet::generate(&mut OsRng);
            save_wallet(&wallet, &wallet_path)?;
            println!("Wallet saved to {}", wallet_path.display());
        }
        Command::Wallet(WalletCmd::Delete) => {
            if wallet_path.is_file() {
                fs::remove_file(&wallet_path)?;
                println!("Deleted wallet file at {}", wallet_path.display());
            } else if wallet_path.exists() {
                println!(
                    "Expected wallet file at {} but found something that is not a file; refusing to delete it",
                    wallet_path.display()
                );
            } else {
                println!(
                    "No wallet exists at {}, so it cannot be deleted",
                    wallet_path.display()
                );
            }
        }
        Command::Addr(AddrCmd::List) => {
            let wallet = load_wallet(&wallet_path);

            use comfy_table::{presets, Table};
            let mut table = Table::new();
            table.load_preset(presets::NOTHING);
            table.set_header(vec!["Index", "Label", "Address"]);
            for (index, label, address) in wallet.addresses() {
                table.add_row(vec![index.to_string(), label, address.to_string()]);
            }
            println!("{}", table);
        }
        Command::Addr(AddrCmd::New { label }) => {
            let mut wallet = load_wallet(&wallet_path);
            let (index, address, _dtk) = wallet.new_address(label.clone());
            save_wallet(&wallet, &wallet_path)?;

            use comfy_table::{presets, Table};
            let mut table = Table::new();
            table.load_preset(presets::NOTHING);
            table.set_header(vec!["Index", "Label", "Address"]);
            table.add_row(vec![index.to_string(), label, address.to_string()]);
            println!("{}", table);
        }
        Command::FetchByNoteCommitment { note_commitment } => {
            let spend_key = load_wallet(&wallet_path);
            let _local_storage = ClientState::new(spend_key);
            let mut client =
                WalletClient::connect(format!("http://{}:{}", opt.node, opt.wallet_port)).await?;

            let cm = hex::decode(note_commitment).expect("note commitment is hex encoded");
            let request = tonic::Request::new(TransactionByNoteRequest { cm: cm.clone() });
            tracing::info!("requesting tx by note commitment: {:?}", cm);
            let response = client.transaction_by_note(request).await?;
            tracing::info!("got response: {:?}", response);
        }
        Command::BlockRequest {
            start_height,
            end_height,
        } => {
            let spend_key = load_wallet(&wallet_path);
            let _local_storage = ClientState::new(spend_key);
            let mut client =
                WalletClient::connect(format!("http://{}:{}", opt.node, opt.wallet_port)).await?;
            let request = tonic::Request::new(CompactBlockRangeRequest {
                start_height,
                end_height,
            });
            tracing::info!(
                "requesting state fragments from: {:?} to {:?}",
                start_height,
                end_height
            );
            let mut stream = client.compact_block_range(request).await?.into_inner();

            while let Some(block) = stream.message().await? {
                tracing::info!("got fragment: {:?}", block);
            }
        }
        Command::AssetLookup { asset_id } => {
            let mut client =
                WalletClient::connect(format!("http://{}:{}", opt.node, opt.wallet_port)).await?;
            tracing::info!("requesting asset denom for asset id: {:?}", &asset_id,);
            let request = tonic::Request::new(AssetLookupRequest { asset_id });
            let asset = client.asset_lookup(request).await?.into_inner();

            tracing::info!("got asset: {:?}", asset);
        }
        Command::AssetList {} => {
            let mut client =
                WalletClient::connect(format!("http://{}:{}", opt.node, opt.wallet_port)).await?;
            tracing::info!("requesting asset list");
            let request = tonic::Request::new(AssetListRequest {});

            let mut stream = client.asset_list(request).await?.into_inner();

            while let Some(asset) = stream.message().await? {
                tracing::info!("got asset: {:?}", asset);
            }
        }
        Command::Sync => {
            let spend_key = load_wallet(&wallet_path);
            //sync(&spend_key)?;
        }
        _ => todo!(),
    }

    Ok(())
}

/// Load existing keys from wallet file, printing an error if the file doesn't exist.
fn load_wallet(wallet_path: &Path) -> Wallet {
    let wallet: Wallet = match fs::read(wallet_path) {
        Ok(data) => bincode::deserialize(&data).expect("can deserialize wallet file"),
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => {
                eprintln!(
                    "error: key data not found, run `pcli wallet generate` to generate Penumbra keys"
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

fn save_wallet(wallet: &Wallet, wallet_path: &Path) -> Result<(), anyhow::Error> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(wallet_path)?;

    let seed_data = bincode::serialize(&wallet)?;
    file.write_all(&seed_data)?;

    Ok(())
}
