use std::str::FromStr;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use penumbra_crypto::keys::SeedPhrase;
use rand_core::OsRng;
use sha2::{Digest, Sha256};

use crate::KeyStore;

#[derive(Debug, clap::Subcommand)]
pub enum WalletCmd {
    /// Import from an existing seed phrase.
    ImportFromPhrase {
        /// A 24 word phrase in quotes.
        seed_phrase: String,
    },
    /// Export the full viewing key for the wallet.
    ExportFvk,
    /// Generate a new seed phrase.
    Generate,
    /// Keep the spend seed, but reset all other client state.
    Reset,
    /// Delete the entire wallet permanently.
    Delete,
}

impl WalletCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn needs_sync(&self) -> bool {
        match self {
            WalletCmd::ImportFromPhrase { .. } => false,
            WalletCmd::ExportFvk => false,
            WalletCmd::Generate => false,
            WalletCmd::Reset => false,
            WalletCmd::Delete => false,
        }
    }

    fn archive_wallet(&self, wallet: &KeyStore) -> Result<()> {
        // Archive the newly generated state
        let archive_dir = ProjectDirs::from("zone", "penumbra", "penumbra-testnet-archive")
            .expect("can access penumbra-testnet-archive dir");

        // Create the directory <data dir>/penumbra-testnet-archive/<chain id>/<spend key hash prefix>/
        let spend_key_hash = Sha256::digest(&wallet.spend_key.to_bytes().0);
        let wallet_archive_dir = archive_dir
            .data_dir()
            .join(hex::encode(&spend_key_hash[0..8]));
        std::fs::create_dir_all(&wallet_archive_dir)
            .expect("can create penumbra wallet archive directory");

        // Save the wallet file in the archive directory
        let archive_path = wallet_archive_dir.join(crate::CUSTODY_FILE_NAME);
        println!("Saving backup wallet to {}", archive_path.display());
        wallet.save(archive_path)?;
        Ok(())
    }

    pub fn exec(&self, data_dir: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let data_dir = data_dir.as_ref();
        match self {
            WalletCmd::Generate => {
                let seed_phrase = SeedPhrase::generate(&mut OsRng);

                // xxx: Something better should be done here, this is in danger of being
                // shared by users accidentally in log output.
                println!(
                    "YOUR PRIVATE SEED PHRASE: {}\nDO NOT SHARE WITH ANYONE!",
                    seed_phrase
                );

                let wallet = KeyStore::from_seed_phrase(seed_phrase);
                wallet.save(data_dir.join(crate::CUSTODY_FILE_NAME))?;
                self.archive_wallet(&wallet)?;
            }
            WalletCmd::ImportFromPhrase { seed_phrase } => {
                let wallet = KeyStore::from_seed_phrase(SeedPhrase::from_str(seed_phrase)?);
                wallet.save(data_dir.join(crate::CUSTODY_FILE_NAME))?;
                self.archive_wallet(&wallet)?;
            }
            WalletCmd::ExportFvk => {
                let wallet = KeyStore::load(data_dir.join(crate::CUSTODY_FILE_NAME))?;
                println!("{}", wallet.spend_key.full_viewing_key());
            }
            WalletCmd::Delete => {
                let wallet_path = data_dir.join(crate::CUSTODY_FILE_NAME);
                if wallet_path.is_file() {
                    std::fs::remove_file(&wallet_path)?;
                    println!("Deleted wallet file at {}", wallet_path);
                } else if wallet_path.exists() {
                    return Err(anyhow!(
                            "Expected wallet file at {} but found something that is not a file; refusing to delete it",
                            wallet_path
                        ));
                } else {
                    return Err(anyhow!(
                        "No wallet exists at {}, so it cannot be deleted",
                        wallet_path
                    ));
                }
            }
            WalletCmd::Reset => {
                tracing::info!("resetting client state");
                let view_path = data_dir.join(crate::VIEW_FILE_NAME);
                if view_path.is_file() {
                    std::fs::remove_file(&view_path)?;
                    println!("Deleted view data at {}", view_path);
                } else if view_path.exists() {
                    return Err(anyhow!(
                            "Expected view data at {} but found something that is not a file; refusing to delete it",
                            view_path
                        ));
                } else {
                    return Err(anyhow!(
                        "No view data exists at {}, so it cannot be deleted",
                        view_path
                    ));
                }
            }
        };
        Ok(())
    }
}
