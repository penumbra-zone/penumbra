use std::{
    io::{IsTerminal as _, Read},
    str::FromStr,
};

use anyhow::Result;
use camino::Utf8PathBuf;
use penumbra_custody::threshold;
use penumbra_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use rand_core::OsRng;
use url::Url;

use crate::{
    config::{CustodyConfig, GovernanceCustodyConfig, PcliConfig},
    terminal::ActualTerminal,
};

#[derive(Debug, clap::Parser)]
pub struct InitCmd {
    #[clap(subcommand)]
    pub subcmd: InitTopSubCmd,
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
    /// For configs with spend authority, this will enable password encryption.
    ///
    /// This has no effect on a view only service.
    #[clap(long, action)]
    encrypted: bool,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum InitTopSubCmd {
    #[clap(flatten)]
    Spend(InitSubCmd),
    /// Initialize `pcli` in view-only mode, without spending keys.
    #[clap(display_order = 200)]
    ViewOnly {
        /// The full viewing key for the wallet to view.
        full_viewing_key: String,
    },
    /// Initialize a separate validator governance key for an existing `pcli` configuration (this
    /// option is only meaningful for validators).
    #[clap(subcommand, display_order = 300)]
    ValidatorGovernanceSubkey(InitSubCmd),
    /// If relevant, change the current config to an encrypted config, with a password.
    #[clap(display_order = 800)]
    ReEncrypt,
    /// Wipe all `pcli` configuration and data, INCLUDING KEYS.
    #[clap(display_order = 900)]
    UnsafeWipe {},
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum InitSubCmd {
    /// Initialize using a basic, file-based custody backend.
    #[clap(subcommand, display_order = 100)]
    SoftKms(SoftKmsInitCmd),
    /// Initialize using a manual threshold signing backend.
    #[clap(subcommand, display_order = 150)]
    Threshold(ThresholdInitCmd),
    // This is not accessible directly by the user, because it's impermissible to initialize the
    // governance subkey as view-only.
    #[clap(skip)]
    ViewOnly { full_viewing_key: String },
}

#[derive(Debug, Clone, clap::Subcommand)]
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
    fn spend_key(&self, init_type: InitType) -> Result<SpendKey> {
        Ok(match self {
            SoftKmsInitCmd::Generate => {
                let seed_phrase = SeedPhrase::generate(OsRng);

                // TODO: Something better should be done here, this is in danger of being
                // shared by users accidentally in log output.
                println!(
                    "YOUR PRIVATE {}SEED PHRASE:\n\n  {seed_phrase}\n\nSave this in a safe place!\nDO NOT SHARE WITH ANYONE!\n",
                    if let InitType::SpendKey = init_type {
                        ""
                    } else {
                        "GOVERNANCE "
                    },
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
                if std::io::stdin().is_terminal() {
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

#[derive(Debug, Clone, clap::Subcommand)]
pub enum ThresholdInitCmd {
    /// Use a centralized dealer to create config files for each signer.
    ///
    /// Unlike the other `pcli init` commands, this one ignores the global
    /// `--home` argument, since it generates one config for each signer.
    Deal {
        /// The minimum number of signers required to make a signature (>= 2).
        #[clap(short, long)]
        threshold: u16,
        /// A path to the home directory for each signer.
        ///
        /// Each directory will be configured to be used as the --home parameter
        /// for that signer's pcli instance.  This implicitly specifies the
        /// total number of signers (one for each --home).
        #[clap(long, value_delimiter = ' ', multiple_values = true)]
        home: Vec<Utf8PathBuf>,
    },
    /// Generate a config file without using a trusted dealer.
    Dkg {
        /// The minimum number of signers required to make a signature (>= 2).
        #[clap(short, long)]
        threshold: u16,
        /// The maximum number of signers that can make a signature
        #[clap(short, long)]
        num_participants: u16,
    },
}

fn exec_deal(
    init_type: InitType,
    threshold: u16,
    home: Vec<Utf8PathBuf>,
    grpc_url: Url,
) -> Result<()> {
    if threshold < 2 {
        anyhow::bail!("threshold must be >= 2");
    }
    let n = home.len() as u16;

    // Check before doing anything to make sure that files don't exist (spend key case) or that the
    // governance key is missing in all of them (governance key case) -- we do this check first so
    // that we don't write partial results if we would fail partway through (though we *also* check
    // partway through to reduce chances of a race where we'd overwrite data)
    for config_path in home.iter() {
        let config_path = config_path.join(crate::CONFIG_FILE_NAME);
        if let InitType::GovernanceKey = init_type {
            let config = PcliConfig::load(&config_path)?;
            if config.governance_custody.is_some() {
                anyhow::bail!(
                    "governance key already exists in config file at {:?}; refusing to overwrite it",
                    config_path
                );
            }
        } else if config_path.exists() {
            anyhow::bail!(
                "config file already exists at {:?}; refusing to overwrite it",
                config_path
            );
        }
    }

    println!("Generating {}-of-{} threshold config.", threshold, n);
    let configs = threshold::Config::deal(&mut OsRng, threshold, n)?;
    println!("Writing dealt config files...");
    for (i, (config, config_path)) in configs.into_iter().zip(home.iter()).enumerate() {
        let full_viewing_key = config.fvk().clone();

        let config = if let InitType::SpendKey = init_type {
            PcliConfig {
                custody: CustodyConfig::Threshold(config),
                full_viewing_key,
                grpc_url: grpc_url.clone(),
                view_url: None,
                disable_warning: false,
                governance_custody: None,
            }
        } else {
            let mut pcli_config = PcliConfig::load(config_path.join(crate::CONFIG_FILE_NAME))?;
            if pcli_config.governance_custody.is_some() {
                anyhow::bail!(
                    "governance key already exists in config file at {:?}; refusing to overwrite it",
                    config_path
                );
            }
            pcli_config.governance_custody = Some(GovernanceCustodyConfig::Threshold(config));
            pcli_config
        };

        println!("  Writing signer {} config to {}", i, config_path);
        std::fs::create_dir_all(config_path)?;
        config.save(config_path.join(crate::CONFIG_FILE_NAME))?;
    }
    Ok(())
}

/// Which kind of initialization are we doing?
#[derive(Clone, Copy)]
enum InitType {
    /// Initialize from scratch with a spend key.
    SpendKey,
    /// Add a governance key to an existing configuration.
    GovernanceKey,
}

impl InitCmd {
    pub async fn exec(&self, home_dir: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let (init_type, subcmd) = match self.subcmd.clone() {
            InitTopSubCmd::Spend(subcmd) => (InitType::SpendKey, subcmd),
            InitTopSubCmd::ValidatorGovernanceSubkey(subcmd) => (InitType::GovernanceKey, subcmd),
            InitTopSubCmd::ViewOnly { full_viewing_key } => (
                InitType::SpendKey,
                InitSubCmd::ViewOnly { full_viewing_key },
            ),
            InitTopSubCmd::UnsafeWipe {} => {
                println!("Deleting all data in {}...", home_dir.as_ref());
                std::fs::remove_dir_all(home_dir.as_ref())?;
                return Ok(());
            }
            InitTopSubCmd::ReEncrypt => {
                let path = home_dir.as_ref().join(crate::CONFIG_FILE_NAME);
                let mut config = PcliConfig::load(path.clone())?;
                config.custody = match config.custody.clone() {
                    x @ CustodyConfig::ViewOnly => x,
                    x @ CustodyConfig::Encrypted(_) => x,
                    CustodyConfig::SoftKms(spend_key) => {
                        let password = ActualTerminal.get_confirmed_password().await?;
                        CustodyConfig::Encrypted(penumbra_custody::encrypted::Config::create(
                            &password,
                            penumbra_custody::encrypted::InnerConfig::SoftKms(spend_key.into()),
                        )?)
                    }
                    CustodyConfig::Threshold(c) => {
                        let password = ActualTerminal.get_confirmed_password().await?;
                        CustodyConfig::Encrypted(penumbra_custody::encrypted::Config::create(
                            &password,
                            penumbra_custody::encrypted::InnerConfig::Threshold(c),
                        )?)
                    }
                };
                config.save(path)?;
                return Ok(());
            }
        };

        if let InitSubCmd::Threshold(ThresholdInitCmd::Deal { threshold, home }) = &subcmd {
            exec_deal(
                init_type,
                threshold.clone(),
                home.clone(),
                self.grpc_url.clone(),
            )?;
            return Ok(());
        }
        let home_dir = home_dir.as_ref();

        match &init_type {
            InitType::SpendKey => {
                // Check that the data_dir is empty before running init:
                if home_dir.exists() && home_dir.read_dir()?.next().is_some() {
                    anyhow::bail!(
                        "home directory {:?} is not empty; refusing to initialize",
                        home_dir
                    );
                }
            }
            InitType::GovernanceKey => {
                // Check that there is no existing governance key before running init:
                let config_path = home_dir.join(crate::CONFIG_FILE_NAME);
                let config = PcliConfig::load(config_path)?;
                if config.governance_custody.is_some() {
                    anyhow::bail!(
                        "governance key already exists in config file at {:?}; refusing to overwrite it",
                        home_dir
                    );
                }
            }
        }

        let (full_viewing_key, custody) = match (&init_type, &subcmd) {
            (_, InitSubCmd::SoftKms(cmd)) => {
                let spend_key = cmd.spend_key(init_type)?;
                (
                    spend_key.full_viewing_key().clone(),
                    if self.encrypted {
                        let password = ActualTerminal.get_confirmed_password().await?;
                        CustodyConfig::Encrypted(penumbra_custody::encrypted::Config::create(
                            &password,
                            penumbra_custody::encrypted::InnerConfig::SoftKms(spend_key.into()),
                        )?)
                    } else {
                        CustodyConfig::SoftKms(spend_key.into())
                    },
                )
            }
            (
                _,
                InitSubCmd::Threshold(ThresholdInitCmd::Dkg {
                    threshold,
                    num_participants,
                }),
            ) => {
                let config = threshold::dkg(*threshold, *num_participants, &ActualTerminal).await?;
                let fvk = config.fvk().clone();
                let custody_config = if self.encrypted {
                    let password = ActualTerminal.get_confirmed_password().await?;
                    CustodyConfig::Encrypted(penumbra_custody::encrypted::Config::create(
                        &password,
                        penumbra_custody::encrypted::InnerConfig::Threshold(config),
                    )?)
                } else {
                    CustodyConfig::Threshold(config)
                };
                (fvk, custody_config)
            }
            (_, InitSubCmd::Threshold(ThresholdInitCmd::Deal { .. })) => {
                unreachable!("this should already have been handled above")
            }
            (InitType::SpendKey, InitSubCmd::ViewOnly { full_viewing_key }) => {
                let full_viewing_key = full_viewing_key.parse()?;
                (full_viewing_key, CustodyConfig::ViewOnly)
            }
            (InitType::GovernanceKey, InitSubCmd::ViewOnly { .. }) => {
                unreachable!("governance keys can't be initialized in view-only mode")
            }
        };

        let config = if let InitType::SpendKey = init_type {
            PcliConfig {
                custody,
                full_viewing_key,
                grpc_url: self.grpc_url.clone(),
                view_url: None,
                disable_warning: false,
                governance_custody: None,
            }
        } else {
            let config_path = home_dir.join(crate::CONFIG_FILE_NAME);
            let mut config = PcliConfig::load(config_path)?;
            let governance_custody = match custody {
                CustodyConfig::SoftKms(config) => GovernanceCustodyConfig::SoftKms(config),
                CustodyConfig::Threshold(config) => GovernanceCustodyConfig::Threshold(config),
                _ => unreachable!("governance keys can't be initialized in view-only mode"),
            };
            config.governance_custody = Some(governance_custody);
            config
        };

        let config_path = home_dir.join(crate::CONFIG_FILE_NAME);
        println!("Writing generated config to {}", config_path);
        config.save(config_path)?;

        if let InitType::GovernanceKey = init_type {
            println!("\nIf you defined a validator on-chain before initializing this separate governance subkey, you need to update its definition to use your new public governance key:\n");
            println!("  governance_key = \"{}\"", config.governance_key());
            println!("\nUntil you do this, your validator will not be able to vote on governance proposals, so it's best to do it at your earliest convenience.")
        }

        Ok(())
    }
}
