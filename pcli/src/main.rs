use anyhow::{anyhow, Result};
use comfy_table::{presets, Table};
use directories::ProjectDirs;
use penumbra_crypto::keys::SpendSeed;
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use penumbra_crypto::CURRENT_CHAIN_ID;
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

    let archive_dir = ProjectDirs::from("zone", "penumbra", "testnet-archive")
        .expect("can access penumbra testnet-archive dir");
    let project_dir =
        ProjectDirs::from("zone", "penumbra", "pcli").expect("can access penumbra project dir");
    // Currently we use just the data directory. Create it if it is missing.
    std::fs::create_dir_all(project_dir.data_dir()).expect("can create penumbra data directory");
    std::fs::create_dir_all(archive_dir.data_dir())
        .expect("can create penumbra testnet-archive directory");

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
                format!("http://{}:{}", opt.node, opt.lightwallet_port),
            )
            .await?;
        }
        Command::Tx(TxCmd::Send {
            amount,
            denomination,
            address,
            fee,
            source_address_id,
            memo,
        }) => {
            let mut state = ClientStateFile::load(wallet_path)?;
            sync(
                &mut state,
                format!("http://{}:{}", opt.node, opt.lightwallet_port),
            )
            .await?;
            let tx = state.new_transaction(
                &mut OsRng,
                amount,
                denomination,
                address,
                fee,
                source_address_id,
                memo,
            )?;
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
        Command::Wallet(WalletCmd::Generate) => {
            if wallet_path.exists() {
                return Err(anyhow::anyhow!(
                    "Wallet path {} already exists, refusing to overwrite it",
                    wallet_path.display()
                ));
            }
            let state = ClientState::new(Wallet::generate(&mut OsRng));
            println!("Saving wallet to {}", wallet_path.display());
            ClientStateFile::save(state.clone(), wallet_path)?;
            // Also save the archived version for testnet backup purposes

            // The archived wallet is stored in a path determined by the testnet ID and hash of the key material.
            let archive_path: PathBuf;
            let mut hasher = Sha256::new();
            hasher.update(&state.wallet().spend_key().seed().0);
            let result = hasher.finalize();

            let wallet_archive_dir = archive_dir
                .data_dir()
                .join(CURRENT_CHAIN_ID)
                .join(hex::encode(&result[0..8]));
            std::fs::create_dir_all(&wallet_archive_dir)
                .expect("can create penumbra wallet archive directory");
            archive_path = wallet_archive_dir.join("penumbra_wallet.json");
            println!("Saving backup wallet to {}", archive_path.display());
            ClientStateFile::save(state, archive_path)?;
        }
        Command::Wallet(WalletCmd::Import { spend_seed }) => {
            if wallet_path.exists() {
                return Err(anyhow::anyhow!(
                    "Wallet path {} already exists, refusing to overwrite it",
                    wallet_path.display()
                ));
            }
            let spend_seed = SpendSeed::try_from(hex::decode(&spend_seed)?.as_slice())?;
            let state = ClientState::new(Wallet::import(spend_seed));
            println!("Saving wallet to {}", wallet_path.display());
            ClientStateFile::save(state, wallet_path)?;
        }
        Command::Wallet(WalletCmd::Export) => {
            let state = ClientStateFile::load(wallet_path)?;
            let seed = state.wallet().spend_key().seed().clone();
            println!("{}", hex::encode(&seed.0));
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
        Command::Wallet(WalletCmd::Reset) => {
            tracing::info!("resetting client state");
            let mut state = ClientStateFile::load(wallet_path)?;
            let wallet = state.wallet().clone();
            let new_state = ClientState::new(wallet);
            *state = new_state;
            state.commit()?;
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
        Command::Addr(AddrCmd::Show { index, addr_only }) => {
            let state = ClientStateFile::load(wallet_path)?;
            let (label, address) = state.wallet().address_by_index(index as usize)?;

            if addr_only {
                println!("{}", address.to_string());
            } else {
                let mut table = Table::new();
                table.load_preset(presets::NOTHING);
                table.set_header(vec!["Index", "Label", "Address"]);
                table.add_row(vec![index.to_string(), label, address.to_string()]);
                println!("{}", table);
            }
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
        Command::Balance { by_address } => {
            let mut state = ClientStateFile::load(wallet_path)?;
            sync(
                &mut state,
                format!("http://{}:{}", opt.node, opt.lightwallet_port),
            )
            .await?;

            if by_address {
                let mut table = Table::new();
                table.load_preset(presets::NOTHING);
                table.set_header(vec!["Address", "Asset", "Balance"]);
                for (address_id, by_denom) in state.unspent_notes_by_address_and_denom().into_iter()
                {
                    let (mut label, _) = state.wallet().address_by_index(address_id as usize)?;
                    for (denom, notes) in by_denom.into_iter() {
                        let balance: u64 = notes.iter().map(|note| note.amount()).sum();
                        table.add_row(vec![label.clone(), denom, balance.to_string()]);
                        // Only display the label on the first row
                        label = String::default();
                    }
                }
                println!("{}", table);
            } else {
                let mut table = Table::new();
                table.load_preset(presets::NOTHING);
                table.set_header(vec!["Asset", "Balance"]);
                for (denom, by_address) in state.unspent_notes_by_denom_and_address().into_iter() {
                    let balance: u64 = by_address
                        .values()
                        .flatten()
                        .map(|note| note.amount())
                        .sum();
                    table.add_row(vec![denom, balance.to_string()]);
                }
                println!("{}", table);
            }
        }
    }

    Ok(())
}
