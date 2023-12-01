use std::{io::Read, str::FromStr};

use anyhow::Result;
use camino::Utf8PathBuf;
use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use rand_core::OsRng;
use url::Url;

use crate::config::{CustodyConfig, PcliConfig};

#[derive(Debug, clap::Parser)]
pub struct InitCmd {
    #[clap(subcommand)]
    pub subcmd: InitSubCmd,
    /// The GRPC URL that will be used in the generated config.
    #[clap(
            long,
            default_value = "https://grpc.testnet.penumbra.zone",
            // Note: reading from the environment here means that running
            // pcli init inside of the test harness (where we override that)
            // will correctly set the URL, even though we don't subsequently
            // read it from the environment.
            env = "PENUMBRA_NODE_PD_URL",
            parse(try_from_str = Url::parse),
        )]
    grpc_url: Url,
}

#[derive(Debug, clap::Subcommand)]
pub enum InitSubCmd {
    /// Initialize `pcli` with a basic, file-based custody backend.
    #[clap(subcommand, display_order = 100)]
    SoftKms(SoftKmsInitCmd),
    /// Initialize `pcli` with a manual threshold signing backend.
    #[clap(subcommand, display_order = 150)]
    Threshold(ThresholdInitCmd),
    /// Initialize `pcli` in view-only mode, without spending keys.
    #[clap(display_order = 200)]
    ViewOnly {
        /// The full viewing key for the wallet to view.
        full_viewing_key: String,
    },
    /// Wipe all `pcli` configuration and data, INCLUDING KEYS.
    #[clap(display_order = 900)]
    UnsafeWipe {},
}

#[derive(Debug, clap::Subcommand)]
pub enum SoftKmsInitCmd {
    /// Generate a new seed phrase and import its corresponding key.
    #[clap(display_order = 100)]
    Generate,
    /// Import a spend key from an existing seed phrase.
    #[clap(display_order = 200)]
    ImportPhrase {
        /// If set, will use legacy BIP39 derivation.
        ///
        /// Use this ONLY if:
        /// - you generated your wallet prior to Testnet 62.
        /// - you need to replicate legacy derivation for some reason.
        #[clap(long, action)]
        legacy_raw_bip39_derivation: bool,
    },
}

impl SoftKmsInitCmd {
    fn spend_key(&self) -> Result<SpendKey> {
        Ok(match self {
            SoftKmsInitCmd::Generate => {
                let seed_phrase = SeedPhrase::generate(OsRng);

                // xxx: Something better should be done here, this is in danger of being
                // shared by users accidentally in log output.
                println!(
                    "YOUR PRIVATE SEED PHRASE:\n{seed_phrase}\nSave this in a safe place!\nDO NOT SHARE WITH ANYONE!"
                );

                let path = Bip44Path::new(0);
                SpendKey::from_seed_phrase_bip44(seed_phrase, &path)
            }
            SoftKmsInitCmd::ImportPhrase {
                legacy_raw_bip39_derivation,
            } => {
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

                let seed_phrase = SeedPhrase::from_str(&seed_phrase)?;

                if *legacy_raw_bip39_derivation {
                    SpendKey::from_seed_phrase_bip39(seed_phrase, 0)
                } else {
                    let path = Bip44Path::new(0);
                    SpendKey::from_seed_phrase_bip44(seed_phrase, &path)
                }
            }
        })
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum ThresholdInitCmd {
    /// Use a centralized dealer to create config files for each signer.
    Deal {
        /// The minimum number of signers required to make a signature (>= 2).
        #[clap(short, long)]
        threshold: u16,
        /// One file name for each signer, where the dealt config will be saved.
        #[clap(short, long, value_delimiter = ' ', multiple_values = true)]
        out: Vec<Utf8PathBuf>,
    },
}

impl ThresholdInitCmd {
    fn exec(&self, grpc_url: Url) -> Result<()> {
        use penumbra_custody::threshold;

        match self {
            ThresholdInitCmd::Deal { threshold: t, out } => {
                if *t < 2 {
                    anyhow::bail!("threshold must be >= 2");
                }
                let configs = threshold::Config::deal(&mut OsRng, *t, out.len() as u16)?;
                println!("Writing dealt config files.");
                for (config, path) in configs.into_iter().zip(out.iter()) {
                    let full_viewing_key = config.fvk().clone();
                    let config = PcliConfig {
                        custody: CustodyConfig::Threshold(config),
                        full_viewing_key,
                        grpc_url: grpc_url.clone(),
                        view_url: None,
                        disable_warning: false,
                    };
                    config.save(path.join(crate::CONFIG_FILE_NAME))?;
                }
                Ok(())
            }
        }
    }
}

impl InitCmd {
    pub fn exec(&self, home_dir: impl AsRef<camino::Utf8Path>) -> Result<()> {
        if let InitSubCmd::Threshold(cmd) = &self.subcmd {
            cmd.exec(self.grpc_url.clone())?;
            return Ok(());
        }
        let home_dir = home_dir.as_ref();

        match &self.subcmd {
            InitSubCmd::UnsafeWipe {} => {
                println!("Deleting all data in {}...", home_dir);
                std::fs::remove_dir_all(home_dir)?;
                return Ok(());
            }
            _ => {
                // Check that the data_dir is empty before running init:
                if home_dir.exists() && home_dir.read_dir()?.next().is_some() {
                    anyhow::bail!(
                        "home directory {:?} is not empty; refusing to initialize",
                        home_dir
                    );
                }
            }
        }

        let (full_viewing_key, custody) = match &self.subcmd {
            InitSubCmd::UnsafeWipe {} => unreachable!("this case is handled above"),
            InitSubCmd::SoftKms(cmd) => {
                let spend_key = cmd.spend_key()?;
                (
                    spend_key.full_viewing_key().clone(),
                    CustodyConfig::SoftKms(spend_key.into()),
                )
            }
            InitSubCmd::Threshold(_) => panic!("this should already have been handled above"),
            InitSubCmd::ViewOnly { full_viewing_key } => {
                let full_viewing_key = full_viewing_key.parse()?;
                (full_viewing_key, CustodyConfig::ViewOnly)
            }
        };

        let config = PcliConfig {
            custody,
            full_viewing_key,
            grpc_url: self.grpc_url.clone(),
            view_url: None,
            disable_warning: false,
        };

        // Create the config directory, if

        let config_path = home_dir.join(crate::CONFIG_FILE_NAME);
        println!("Writing generated configs to {}", config_path);
        config.save(config_path)?;

        Ok(())
    }
}
