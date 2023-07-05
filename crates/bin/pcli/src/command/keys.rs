use std::io::Read;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use penumbra_keys::keys::SeedPhrase;
use rand_core::OsRng;
use sha2::{Digest, Sha256};

use crate::KeyStore;

#[derive(Debug, clap::Subcommand)]
pub enum KeysCmd {
    /// Import an existing key.
    #[clap(subcommand)]
    Import(ImportCmd),
    /// Export keys from the wallet.
    #[clap(subcommand)]
    Export(ExportCmd),
    /// Generate a new seed phrase and import its corresponding key.
    Generate,
    /// Delete the entire wallet permanently.
    Delete,
}

#[derive(Debug, clap::Subcommand)]
pub enum ImportCmd {
    /// Import wallet from an existing 24-word seed phrase. Will prompt for input interactively.
    /// Also accepts input from stdin, for use with pipes.
    Phrase,
}

#[derive(Debug, clap::Subcommand)]
pub enum ExportCmd {
    /// Export the full viewing key for the wallet.
    FullViewingKey,
    /// Export the account group ID.
    AccountGroupId,
}

impl KeysCmd {
    /// Determine if this command requires a network sync before it executes.
    pub fn offline(&self) -> bool {
        true
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
            KeysCmd::Generate => {
                let seed_phrase = SeedPhrase::generate(OsRng);

                // xxx: Something better should be done here, this is in danger of being
                // shared by users accidentally in log output.
                println!("YOUR PRIVATE SEED PHRASE: {seed_phrase}\nDO NOT SHARE WITH ANYONE!");

                let wallet = KeyStore::from_seed_phrase(seed_phrase);
                wallet.save(data_dir.join(crate::CUSTODY_FILE_NAME))?;
                self.archive_wallet(&wallet)?;
            }
            KeysCmd::Import(ImportCmd::Phrase) => {
                let mut seed_phrase = String::new();
                // The `rpassword` crate doesn't support reading from stdin, so we check
                // for an interactive session. We must support non-interactive use cases,
                // for integration with other tooling.
                if atty::is(atty::Stream::Stdin) {
                    seed_phrase = rpassword::prompt_password("Enter seed phrase: ")?;
                } else {
                    while let Ok(n_bytes) = std::io::stdin().lock().read_to_string(&mut seed_phrase)
                    {
                        if n_bytes == 0 {
                            break;
                        }
                        seed_phrase = seed_phrase.trim().to_string();
                    }
                }
                let wallet = KeyStore::from_seed_phrase(SeedPhrase::from_str(&seed_phrase)?);
                wallet.save(data_dir.join(crate::CUSTODY_FILE_NAME))?;
                self.archive_wallet(&wallet)?;
            }
            KeysCmd::Export(ExportCmd::FullViewingKey) => {
                let wallet = KeyStore::load(data_dir.join(crate::CUSTODY_FILE_NAME))?;
                println!("{}", wallet.spend_key.full_viewing_key());
            }
            KeysCmd::Export(ExportCmd::AccountGroupId) => {
                let wallet = KeyStore::load(data_dir.join(crate::CUSTODY_FILE_NAME))?;
                let account_group_id = wallet.spend_key.full_viewing_key().account_group_id();
                println!("{}", serde_json::to_string_pretty(&account_group_id)?);
            }
            KeysCmd::Delete => {
                let wallet_path = data_dir.join(crate::CUSTODY_FILE_NAME);
                if wallet_path.is_file() {
                    std::fs::remove_file(&wallet_path)?;
                    println!("Deleted wallet file at {wallet_path}");
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
        };

        Ok(())
    }
}
