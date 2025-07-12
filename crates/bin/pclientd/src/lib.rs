// Requires nightly.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use std::io::IsTerminal;
use std::io::Read;
use std::net::SocketAddr;
use std::path::Path;

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Parser;
use directories::ProjectDirs;
use penumbra_sdk_custody::policy::{AuthPolicy, PreAuthorizationPolicy};
use penumbra_sdk_custody::soft_kms::{self, SoftKms};
use penumbra_sdk_keys::keys::{Bip44Path, SeedPhrase, SpendKey};
use penumbra_sdk_keys::FullViewingKey;
use penumbra_sdk_proto::{
    core::app::v1::{
        query_service_client::QueryServiceClient as AppQueryServiceClient, AppParametersRequest,
    },
    custody::v1::custody_service_server::CustodyServiceServer,
    view::v1::view_service_server::ViewServiceServer,
};
use penumbra_sdk_view::{Storage, ViewServer};
use reqwest;
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tempfile::NamedTempFile;

use std::fs;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use tonic::transport::Server;
use url::Url;

mod proxy;
pub use proxy::{
    AppQueryProxy, ChainQueryProxy, CompactBlockQueryProxy, DexQueryProxy, DexSimulationProxy,
    GovernanceQueryProxy, SctQueryProxy, ShieldedPoolQueryProxy, StakeQueryProxy,
    TendermintProxyProxy,
};

use crate::proxy::FeeQueryProxy;

#[serde_as]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PclientdConfig {
    /// FVK for both view and custody modes
    #[serde_as(as = "DisplayFromStr")]
    pub full_viewing_key: FullViewingKey,
    /// The URL of the gRPC endpoint used to talk to pd.
    pub grpc_url: Url,
    /// The address to bind to serve gRPC.
    pub bind_addr: SocketAddr,
    /// Optional KMS config for custody mode
    pub kms_config: Option<soft_kms::Config>,
}

impl PclientdConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let contents = toml::to_string_pretty(&self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

pub fn default_home() -> Utf8PathBuf {
    let path = ProjectDirs::from("zone", "penumbra", "pclientd")
        .expect("Failed to get platform data dir")
        .data_dir()
        .to_path_buf();
    Utf8PathBuf::from_path_buf(path).expect("Platform default data dir was not UTF-8")
}

#[derive(Debug, Parser)]
#[clap(name = "pclientd", about = "The Penumbra view daemon.", version)]
pub struct Opt {
    /// Command to run.
    #[clap(subcommand)]
    pub cmd: Command,
    /// The path used to store pclientd state and config files.
    #[clap(long, default_value_t = default_home(), env = "PENUMBRA_PCLIENTD_HOME")]
    pub home: Utf8PathBuf,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Generate configs for `pclientd` in view or custody mode.
    ///
    /// In custody mode, pclientd will have spend authority over the configured account,
    /// enabling it to perform transactions on behalf of the wallet.
    ///
    /// In view mode, pclientd will be able to read all transactions related to the
    /// configured account, but cannot create new transactions.
    Init {
        /// If provided, initialize in view mode, by providing a full viewing key.
        ///
        /// Otherwise, a prompt will accept a seed phrase.
        #[clap(long, display_order = 100)]
        view: bool,
        /// Sets the URL of the gRPC endpoint used to talk to pd.
        #[clap(
            long,
            display_order = 900,
            parse(try_from_str = Url::parse)
        )]
        grpc_url: Url,
        /// Sets the address to bind to serve gRPC.
        #[clap(long, display_order = 900, default_value = "127.0.0.1:8081")]
        bind_addr: SocketAddr,
    },
    /// Start running `pclientd`.
    Start {},
    /// Delete `pclientd` storage to reset local state.
    Reset {},
    /// Load assets from a registry into the pclientd instance.
    ///
    /// This enables smarter handling of metadata.
    LoadRegistry {
        /// If present, where to fetch the assets from.
        ///
        /// If this is not present, this will use the Prax wallet registry, with the chain
        /// ID pclientd has previously been initialize with to source the correct registry.
        ///
        /// If this is present, it will be assumed to be an HTTP URL. If the URL ends in ".json",
        /// it's assumed to be a specific registry file, which will be fetched. If the URL
        /// does not end in ".json", it will be concatenated with the chain id pclientd has,
        /// in the assumption that this points to a folder of registry files.
        ///
        /// If the URL starts with "file://" instead of "https://" or "http://", then
        /// the local filesystem will be used, with all the same rules.
        #[clap(long)]
        source: Option<String>,
    },
}

impl Opt {
    fn config_path(&self) -> Utf8PathBuf {
        let mut path = self.home.clone();
        path.push("config.toml");
        path
    }

    fn sqlite_path(&self) -> Utf8PathBuf {
        let mut path = self.home.clone();
        path.push("pclientd-db.sqlite");
        path
    }

    fn check_home_nonempty(&self) -> Result<()> {
        if self.home.exists() {
            if !self.home.is_dir() {
                return Err(anyhow::anyhow!(
                    "The home directory {:?} is not a directory.",
                    self.home
                ));
            }
            let mut entries = fs::read_dir(&self.home)?.peekable();
            if entries.peek().is_some() {
                return Err(anyhow::anyhow!(
                    "The home directory {:?} is not empty, refusing to overwrite it",
                    self.home
                ));
            }
        } else {
            fs::create_dir_all(&self.home)?;
        }
        Ok(())
    }

    async fn init_sqlite(&self, fvk: &FullViewingKey, grpc_url: &Url) -> Result<Storage> {
        // Initialize client and storage
        let mut client = AppQueryServiceClient::connect(grpc_url.to_string()).await?;

        let params = client
            .app_parameters(tonic::Request::new(AppParametersRequest {}))
            .await?
            .into_inner()
            .try_into()?;

        Storage::initialize(Some(self.sqlite_path()), fvk.clone(), params).await
    }

    async fn load_or_init_sqlite(&self, fvk: &FullViewingKey, grpc_url: &Url) -> Result<Storage> {
        if self.sqlite_path().exists() {
            Ok(Storage::load(self.sqlite_path()).await?)
        } else {
            self.init_sqlite(fvk, grpc_url).await
        }
    }

    // Reusable function for prompting for sensitive info on the CLI.
    fn prompt_for_password(&self, msg: &str) -> Result<String> {
        let mut password = String::new();
        // The `rpassword` crate doesn't support reading from stdin, so we check
        // for an interactive session. We must support non-interactive use cases,
        // for integration with other tooling.
        if std::io::stdin().is_terminal() {
            password = prompt_password(msg)?;
        } else {
            while let Ok(n_bytes) = std::io::stdin().lock().read_to_string(&mut password) {
                if n_bytes == 0 {
                    break;
                }
                password = password.trim().to_string();
            }
        }
        Ok(password)
    }

    pub async fn exec(self) -> Result<()> {
        let opt = self;
        match &opt.cmd {
            Command::Reset {} => {
                if opt.sqlite_path().exists() {
                    fs::remove_file(opt.sqlite_path())?;
                    println!("Deleted local storage at: {:?}", opt.sqlite_path());
                } else {
                    println!("No local storage at: {:?} (have you started pclientd, so it would have data to store?)", opt.sqlite_path());
                }

                Ok(())
            }
            Command::Init {
                view,
                grpc_url,
                bind_addr,
            } => {
                // Check that the home directory is empty.
                opt.check_home_nonempty()?;

                // Initialize key vars, which will differ based on view or custody mode.
                let key_material: String;
                let spend_key: Option<SpendKey>;
                let full_viewing_key: FullViewingKey;

                // If view-only mode is requested, prompt for a FullViewingKey.
                if *view {
                    key_material = opt
                        .prompt_for_password("Enter full viewing key: ")?
                        .to_owned();
                    full_viewing_key = key_material.parse()?;
                    spend_key = None;
                // Otherwise, we're in full custody mode.
                } else {
                    key_material = opt
                        .prompt_for_password(
                            "Enter your seed phrase to enable pclientd custody mode: ",
                        )?
                        .to_owned();
                    let sk = SpendKey::from_seed_phrase_bip44(
                        SeedPhrase::from_str(key_material.as_str())?,
                        &Bip44Path::new(0),
                    );
                    full_viewing_key = sk.full_viewing_key().clone();
                    spend_key = Some(sk);
                }

                println!(
                    "Initializing configuration at: {:?}",
                    fs::canonicalize(&opt.home)?
                );

                // Create config file with example authorization policy.
                let kms_config: Option<soft_kms::Config> = spend_key.map(|spend_key| {
                    // It's important that we throw away the signing key here, so that
                    // by default the config is "cannot spend funds" without manual editing.
                    let pak = ed25519_consensus::SigningKey::new(rand_core::OsRng);
                    let pvk = pak.verification_key();

                    let auth_policy = vec![
                        AuthPolicy::DestinationAllowList {
                            allowed_destination_addresses: vec![
                                spend_key
                                    .incoming_viewing_key()
                                    .payment_address(Default::default())
                                    .0,
                            ],
                        },
                        AuthPolicy::OnlyIbcRelay,
                        AuthPolicy::PreAuthorization(PreAuthorizationPolicy::Ed25519 {
                            required_signatures: 1,
                            allowed_signers: vec![pvk],
                        }),
                    ];
                    soft_kms::Config {
                        spend_key,
                        auth_policy,
                    }
                });

                let client_config = PclientdConfig {
                    kms_config,
                    full_viewing_key,
                    grpc_url: grpc_url.clone(),
                    bind_addr: *bind_addr,
                };

                let encoded = toml::to_string_pretty(&client_config)
                    .expect("able to convert client config to toml string");

                // Write config to directory

                let config_file_path = &mut opt.home.clone();
                config_file_path.push("config.toml");
                let mut config_file = File::create(&config_file_path)?;

                config_file.write_all(encoded.as_bytes())?;

                Ok(())
            }
            Command::Start {} => {
                let config = PclientdConfig::load(opt.config_path()).context(
                    "Failed to load pclientd config file. Have you run `pclientd init` with a FVK?",
                )?;

                tracing::info!(?opt.home, ?config.bind_addr, %config.grpc_url, "starting pclientd");
                let storage = opt
                    .load_or_init_sqlite(&config.full_viewing_key, &config.grpc_url)
                    .await?;

                let proxy_channel = ViewServer::get_pd_channel(config.grpc_url.clone()).await?;

                let app_query_proxy = AppQueryProxy(proxy_channel.clone());
                let governance_query_proxy = GovernanceQueryProxy(proxy_channel.clone());
                let dex_query_proxy = DexQueryProxy(proxy_channel.clone());
                let dex_simulation_proxy = DexSimulationProxy(proxy_channel.clone());
                let sct_query_proxy = SctQueryProxy(proxy_channel.clone());
                let fee_query_proxy = FeeQueryProxy(proxy_channel.clone());
                let shielded_pool_query_proxy = ShieldedPoolQueryProxy(proxy_channel.clone());
                let chain_query_proxy = ChainQueryProxy(proxy_channel.clone());
                let stake_query_proxy = StakeQueryProxy(proxy_channel.clone());
                let compact_block_query_proxy = CompactBlockQueryProxy(proxy_channel.clone());
                let tendermint_proxy_proxy = TendermintProxyProxy(proxy_channel.clone());

                let view_service =
                    ViewServiceServer::new(ViewServer::new(storage, config.grpc_url).await?);
                let custody_service = config.kms_config.as_ref().map(|kms_config| {
                    CustodyServiceServer::new(SoftKms::new(kms_config.spend_key.clone().into()))
                });

                let server = Server::builder()
                    .accept_http1(true)
                    .add_service(tonic_web::enable(view_service))
                    .add_optional_service(custody_service.map(tonic_web::enable))
                    .add_service(tonic_web::enable(app_query_proxy))
                    .add_service(tonic_web::enable(governance_query_proxy))
                    .add_service(tonic_web::enable(dex_query_proxy))
                    .add_service(tonic_web::enable(dex_simulation_proxy))
                    .add_service(tonic_web::enable(sct_query_proxy))
                    .add_service(tonic_web::enable(fee_query_proxy))
                    .add_service(tonic_web::enable(shielded_pool_query_proxy))
                    .add_service(tonic_web::enable(chain_query_proxy))
                    .add_service(tonic_web::enable(stake_query_proxy))
                    .add_service(tonic_web::enable(compact_block_query_proxy))
                    .add_service(tonic_web::enable(tendermint_proxy_proxy))
                    // TODO: should we add the IBC services here as well? they will appear
                    // in reflection but not be available.
                    .add_service(tonic_web::enable(
                        tonic_reflection::server::Builder::configure()
                            .register_encoded_file_descriptor_set(
                                penumbra_sdk_proto::FILE_DESCRIPTOR_SET,
                            )
                            .build_v1()
                            .with_context(|| "could not configure grpc reflection service")?,
                    ))
                    .serve(config.bind_addr);

                tokio::spawn(server).await??;

                Ok(())
            }
            Command::LoadRegistry { source } => {
                let config = PclientdConfig::load(opt.config_path()).context(
                    "Failed to load pclientd config file. Have you run `pclientd init` with a FVK?",
                )?;

                // Load existing storage
                let storage = opt
                    .load_or_init_sqlite(&config.full_viewing_key, &config.grpc_url)
                    .await?;

                // Use provided source or default to Prax wallet registry
                let source_url = source.clone().unwrap_or_else(|| {
                    "https://raw.githubusercontent.com/prax-wallet/registry/refs/heads/main/registry/chains/".to_string()
                });

                // Determine the final registry URL
                let registry_url = determine_registry_url(&source_url, &storage).await?;

                tracing::info!(?registry_url, "Loading assets from registry");

                // Download the registry file to a temporary file
                let temp_file = download_registry_to_temp_file(&registry_url).await?;

                // Load asset metadata into the storage
                let temp_path = camino::Utf8Path::from_path(temp_file.path())
                    .ok_or_else(|| anyhow::anyhow!("Temporary file path is not valid UTF-8"))?;
                storage.load_asset_metadata(temp_path).await?;

                println!("Successfully loaded assets from registry: {}", registry_url);
                Ok(())
            }
        }
    }
}

/// Determines the final registry URL based on the provided source and storage chain ID.
async fn determine_registry_url(source: &str, storage: &Storage) -> Result<String> {
    if source.ends_with(".json") {
        // Direct registry file URL
        Ok(source.to_string())
    } else {
        // Directory URL - need to append chain ID
        let app_params = storage.app_params().await?;
        let chain_id = app_params.chain_id;
        let mut url = source.to_string();
        if !url.ends_with('/') {
            url.push('/');
        }
        url.push_str(&format!("{}.json", chain_id));
        Ok(url)
    }
}

/// Downloads a registry file from the given URL to a temporary file.
async fn download_registry_to_temp_file(url: &str) -> Result<NamedTempFile> {
    if url.starts_with("file://") {
        // Local file - copy to temp file
        let local_path = &url[7..]; // Remove "file://" prefix
        let content = std::fs::read_to_string(local_path)
            .with_context(|| format!("Failed to read local registry file: {}", local_path))?;

        let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;
        temp_file
            .write_all(content.as_bytes())
            .context("Failed to write to temporary file")?;
        temp_file
            .flush()
            .context("Failed to flush temporary file")?;

        Ok(temp_file)
    } else {
        // HTTP/HTTPS URL - download with reqwest
        let response = reqwest::get(url)
            .await
            .with_context(|| format!("Failed to download registry from: {}", url))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download registry: HTTP {}",
                response.status()
            ));
        }

        let content = response
            .text()
            .await
            .context("Failed to read response body")?;

        let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;
        temp_file
            .write_all(content.as_bytes())
            .context("Failed to write to temporary file")?;
        temp_file
            .flush()
            .context("Failed to flush temporary file")?;

        Ok(temp_file)
    }
}
