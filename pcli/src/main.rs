use anyhow::{anyhow, Result};
use comfy_table::{presets, Table};
use directories::ProjectDirs;
use rand_core::OsRng;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use penumbra_proto::wallet::{
    wallet_client::WalletClient, AssetListRequest, AssetLookupRequest, TransactionByNoteRequest,
};
use penumbra_wallet::{ClientState, Wallet};

pub mod opt;
pub mod warning;
use opt::*;

mod sync;
pub use sync::sync;

mod state;
pub use state::ClientStateFile;

#[tokio::main]
async fn main() -> Result<()> {
    // Display a warning message to the user so they don't get upset when all their tokens are lost.
    if std::env::var("PCLI_UNLEASH_DANGER").is_err() {
        warning::display();
    }

    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    let project_dir =
        ProjectDirs::from("zone", "penumbra", "pcli").expect("can access penumbra project dir");
    // Currently we use just the data directory. Create it if it is missing.
    std::fs::create_dir_all(project_dir.data_dir()).expect("can create penumbra data directory");

    // We store wallet data in `penumbra_wallet.dat` in the state directory, unless
    // the user provides another location.
    let wallet_path: PathBuf;
    match opt.wallet_location {
        Some(path) => {
            wallet_path = Path::new(&path).to_path_buf();
        }
        None => {
            wallet_path = project_dir.data_dir().join("penumbra_wallet.json");
        }
    }

    match opt.cmd {
        Command::Sync => {
            let mut state = ClientStateFile::load(wallet_path)?;
            sync(
                &mut state,
                format!("http://{}:{}", opt.node, opt.wallet_port),
            )
            .await?;
        }
        Command::Tx(TxCmd::Send {
            amount,
            denomination,
            address,
            fee,
        }) => {
            let mut state = ClientStateFile::load(wallet_path)?;
            sync(
                &mut state,
                format!("http://{}:{}", opt.node, opt.wallet_port),
            )
            .await?;
            let tx = state.new_transaction(&mut OsRng, amount, denomination, address, fee)?;
            let serialized_tx: Vec<u8> = tx.into();

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
            let state = ClientState::new(Wallet::generate(&mut OsRng));
            println!("Saving wallet to {}", wallet_path.display());
            ClientStateFile::save(state, wallet_path)?;
        }
        Command::Wallet(WalletCmd::Delete) => {
            if wallet_path.is_file() {
                std::fs::remove_file(&wallet_path)?;
                println!("Deleted wallet file at {}", wallet_path.display());
            } else if wallet_path.exists() {
                return Err(anyhow!(
                    "Expected wallet file at {} but found something that is not a file; refusing to delete it",
                    wallet_path.display()
                ));
            } else {
                return Err(anyhow!(
                    "No wallet exists at {}, so it cannot be deleted",
                    wallet_path.display()
                ));
            }
        }
        Command::Addr(AddrCmd::List) => {
            let state = ClientStateFile::load(wallet_path)?;

            let mut table = Table::new();
            table.load_preset(presets::NOTHING);
            table.set_header(vec!["Index", "Label", "Address"]);
            for (index, label, address) in state.wallet().addresses() {
                table.add_row(vec![index.to_string(), label, address.to_string()]);
            }
            println!("{}", table);
        }
        Command::Addr(AddrCmd::New { label }) => {
            let mut state = ClientStateFile::load(wallet_path)?;
            let (index, address, _dtk) = state.wallet_mut().new_address(label.clone());
            state.commit()?;

            let mut table = Table::new();
            table.load_preset(presets::NOTHING);
            table.set_header(vec!["Index", "Label", "Address"]);
            table.add_row(vec![index.to_string(), label, address.to_string()]);
            println!("{}", table);
        }
        Command::FetchByNoteCommitment { note_commitment } => {
            let mut client =
                WalletClient::connect(format!("http://{}:{}", opt.node, opt.wallet_port)).await?;

            let cm = hex::decode(note_commitment).expect("note commitment is hex encoded");
            let request = tonic::Request::new(TransactionByNoteRequest { cm: cm.clone() });
            tracing::info!("requesting tx by note commitment: {:?}", cm);
            let response = client.transaction_by_note(request).await?;
            tracing::info!("got response: {:?}", response);
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
        Command::Balance => {
            let state = ClientStateFile::load(wallet_path)?;
            let notes_by_asset = state.notes_by_asset_denomination();

            let mut table = Table::new();
            table.load_preset(presets::NOTHING);
            table.set_header(vec!["Asset denomination", "Balance"]);
            for (denom, notes) in notes_by_asset {
                let note_amounts: Vec<u64> = notes.iter().map(|note| note.amount()).collect();
                let balance: u64 = note_amounts.iter().sum();
                table.add_row(vec![denom, balance.to_string()]);
            }
            println!("{}", table);
        }
        _ => todo!(),
    }

    Ok(())
}
