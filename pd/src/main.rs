#![allow(clippy::clone_on_copy)]
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use anyhow::Context;
use metrics_exporter_prometheus::PrometheusBuilder;
use pd::genesis::Allocation;
use penumbra_chain::params::ChainParams;
use penumbra_crypto::{
    keys::{SpendKey, SpendSeed},
    rdsa::{SigningKey, SpendAuth, VerificationKey},
};
use penumbra_proto::{
    light_wallet::light_wallet_server::LightWalletServer,
    thin_wallet::thin_wallet_server::ThinWalletServer,
};
use penumbra_stake::{FundingStream, FundingStreams, Validator};
use rand_core::OsRng;
use structopt::StructOpt;
use tonic::transport::Server;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pd",
    about = "The Penumbra daemon.",
    version = env!("VERGEN_GIT_SEMVER"),
)]
struct Opt {
    /// Command to run.
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Start running the ABCI and wallet services.
    Start {
        /// The URI used to connect to the Postgres database.
        #[structopt(short, long)]
        database_uri: String,
        /// Bind the services to this host.
        #[structopt(short, long, default_value = "127.0.0.1")]
        host: String,
        /// Bind the ABCI server to this port.
        #[structopt(short, long, default_value = "26658")]
        abci_port: u16,
        /// Bind the light wallet service to this port.
        #[structopt(short, long, default_value = "26666")]
        light_wallet_port: u16,
        /// Bind the thin wallet service to this port.
        #[structopt(short, long, default_value = "26667")]
        thin_wallet_port: u16,
        /// Bind the metrics endpoint to this port.
        #[structopt(short, long, default_value = "9000")]
        metrics_port: u16,
    },

    /// Generates a directory structure containing necessary files to run a
    /// testnet based on input configuration.
    GenerateTestnet {
        /// How many validator nodes to create configuration for.
        #[structopt(long, default_value = "4")]
        num_validator_nodes: usize,
        /// Number of blocks per epoch.
        #[structopt(long, default_value = "40")]
        epoch_duration: u64,
        /// Number of epochs before unbonding stake is released.
        #[structopt(long, default_value = "40")]
        unbonding_epochs: u64,
        /// Maximum number of validators in the consensus set.
        #[structopt(long, default_value = "10")]
        active_validator_limit: u64,
        /// Penalty to be applied to slashed validators' rates.
        /// Expressed in basis points.
        #[structopt(long, default_value = "1000")]
        slashing_penalty: u64,
        /// Path to CSV file containing initial allocations [default: latest testnet].
        #[structopt(long, parse(from_os_str))]
        allocations_input_file: Option<PathBuf>,
        /// Path to JSON file containing initial validator configs [default: latest testnet].
        #[structopt(long, parse(from_os_str))]
        validators_input_file: Option<PathBuf>,
        /// Path to directory to store output in. Must not exist.
        #[structopt(long)]
        output_dir: Option<PathBuf>,
        /// Testnet name [default: latest testnet].
        #[structopt(long)]
        chain_id: Option<String>,
        /// IP Address to start `tendermint` nodes on. Increments by three to make room for `pd` and `postgres` per node.
        #[structopt(long, default_value = "192.167.10.2")]
        starting_ip: Ipv4Addr,
    },
}

// Extracted from tonic's remote_addr implementation; we'd like to instrument
// spans with the remote addr at the server level rather than at the individual
// request level, but the hook available to do that gives us an http::Request
// rather than a tonic::Request, so the tonic::Request::remote_addr method isn't
// available.
fn remote_addr(req: &http::Request<()>) -> Option<SocketAddr> {
    use tonic::transport::server::TcpConnectInfo;
    // NOTE: needs to also check TlsConnectInfo if we use TLS
    req.extensions()
        .get::<TcpConnectInfo>()
        .and_then(|i| i.remote_addr())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let opt = Opt::from_args();

    match opt.cmd {
        Command::Start {
            host,
            database_uri,
            abci_port,
            light_wallet_port,
            thin_wallet_port,
            metrics_port,
        } => {
            tracing::info!(
                ?host,
                ?database_uri,
                ?abci_port,
                ?light_wallet_port,
                ?thin_wallet_port,
                "starting pd"
            );
            // Initialize state
            let (state_reader, state_writer) = pd::state::new(&database_uri).await?;

            let consensus = pd::Consensus::new(state_writer).await?;
            let mempool = pd::Mempool::new(state_reader.clone());
            let info = pd::Info::new(state_reader.clone());
            let snapshot = pd::Snapshot {};

            let abci_server = tokio::spawn(
                tower_abci::Server::builder()
                    .consensus(consensus)
                    .snapshot(snapshot)
                    .mempool(mempool)
                    .info(info)
                    .finish()
                    .unwrap()
                    .listen(format!("{}:{}", host, abci_port)),
            );

            let light_wallet_server = tokio::spawn(
                Server::builder()
                    .trace_fn(|req| match remote_addr(req) {
                        Some(remote_addr) => tracing::error_span!("light_wallet", ?remote_addr),
                        None => tracing::error_span!("light_wallet"),
                    })
                    .add_service(LightWalletServer::new(state_reader.clone()))
                    .serve(
                        format!("{}:{}", host, light_wallet_port)
                            .parse()
                            .expect("this is a valid address"),
                    ),
            );
            let thin_wallet_server = tokio::spawn(
                Server::builder()
                    .trace_fn(|req| match remote_addr(req) {
                        Some(remote_addr) => tracing::error_span!("thin_wallet", ?remote_addr),
                        None => tracing::error_span!("thin_wallet"),
                    })
                    .add_service(ThinWalletServer::new(state_reader.clone()))
                    .serve(
                        format!("{}:{}", host, thin_wallet_port)
                            .parse()
                            .expect("this is a valid address"),
                    ),
            );

            // This service lets Prometheus pull metrics from `pd`
            PrometheusBuilder::new()
                .with_http_listener(
                    format!("{}:{}", host, metrics_port)
                        .parse::<SocketAddr>()
                        .expect("this is a valid address"),
                )
                .install()
                .expect("metrics service set up");

            pd::register_all_metrics();

            // TODO: better error reporting
            // We error out if either service errors, rather than keep running
            tokio::select! {
                x = abci_server => x?.map_err(|e| anyhow::anyhow!(e))?,
                x = light_wallet_server => x?.map_err(|e| anyhow::anyhow!(e))?,
                x = thin_wallet_server => x?.map_err(|e| anyhow::anyhow!(e))?,
            };
        }
        Command::GenerateTestnet {
            num_validator_nodes,
            // TODO this config is gated on a "populate persistent peers"
            // setting in the Go tendermint binary. Populating the persistent
            // peers will be useful in local setups until peer discovery via a seed
            // works.
            starting_ip: _,
            epoch_duration,
            unbonding_epochs,
            active_validator_limit,
            allocations_input_file,
            validators_input_file,
            output_dir,
            chain_id,
            slashing_penalty,
        } => {
            use rand::Rng;
            use std::{
                fs,
                fs::File,
                io::Write,
                str::FromStr,
                time::{Duration, SystemTime, UNIX_EPOCH},
            };

            // Build script computes the latest testnet name and sets it as an env variable
            let chain_id = chain_id.unwrap_or_else(|| env!("PD_LATEST_TESTNET_NAME").to_string());

            let randomizer = OsRng.gen::<u32>();
            let chain_id = format!("{}-{}", chain_id, hex::encode(&randomizer.to_le_bytes()));

            use pd::{genesis, genesis::ValidatorPower, testnet::*};
            use penumbra_crypto::Address;
            use penumbra_stake::IdentityKey;
            use tendermint::{account::Id, public_key::Algorithm, Genesis, Time};
            use tendermint_config::{NodeKey, PrivValidatorKey};

            assert!(
                num_validator_nodes > 0,
                "must have at least one validator node"
            );

            let genesis_time = Time::from_unix_timestamp(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("time travels linearly in a forward direction")
                    .as_secs() as i64,
                0,
            )
            .expect("able to convert current time into Time");

            // By default output directory will be in `~/.penumbra/testnet_data/`
            let output_dir = match output_dir {
                Some(o) => o,
                None => canonicalize_path("~/.penumbra/testnet_data"),
            };

            // Parse allocations from input file or default to latest testnet allocations computed
            // in the build script
            let mut allocations = if let Some(allocations_input_file) = allocations_input_file {
                let allocations_file = File::open(&allocations_input_file)
                    .with_context(|| format!("cannot open file {:?}", allocations_input_file))?;
                parse_allocations(allocations_file).with_context(|| {
                    format!(
                        "could not parse allocations file {:?}",
                        allocations_input_file
                    )
                })?
            } else {
                static LATEST_ALLOCATIONS: &str =
                    include_str!(env!("PD_LATEST_TESTNET_ALLOCATIONS"));
                parse_allocations(std::io::Cursor::new(LATEST_ALLOCATIONS)).with_context(|| {
                    format!(
                        "could not parse default latest testnet allocations file {:?}",
                        env!("PD_LATEST_TESTNET_ALLOCATIONS")
                    )
                })?
            };

            // Parse validators from input file or default to latest testnet validators computed in
            // the build script
            let validators = if let Some(validators_input_file) = validators_input_file {
                let validators_file = File::open(&validators_input_file)
                    .with_context(|| format!("cannot open file {:?}", validators_input_file))?;
                parse_validators(validators_file).with_context(|| {
                    format!(
                        "could not parse validators file {:?}",
                        validators_input_file
                    )
                })?
            } else {
                static LATEST_VALIDATORS: &str = include_str!(env!("PD_LATEST_TESTNET_VALIDATORS"));
                parse_validators(std::io::Cursor::new(LATEST_VALIDATORS)).with_context(|| {
                    format!(
                        "could not parse default latest testnet validators file {:?}",
                        env!("PD_LATEST_TESTNET_VALIDATORS")
                    )
                })?
            };

            struct ValidatorKeys {
                // Penumbra spending key and viewing key for this node.
                pub validator_id_sk: SigningKey<SpendAuth>,
                pub validator_id_vk: VerificationKey<SpendAuth>,
                // Consensus key for tendermint.
                pub validator_cons_sk: tendermint::PrivateKey,
                pub validator_cons_pk: tendermint::PublicKey,
                // P2P auth key for tendermint.
                pub node_key_sk: tendermint::PrivateKey,
                #[allow(unused_variables, dead_code)]
                pub node_key_pk: tendermint::PublicKey,
                pub validator_spendseed: SpendSeed,
            }
            let mut validator_keys = Vec::<ValidatorKeys>::new();
            // Generate a keypair for each validator
            for _ in 0..num_validator_nodes {
                // Create the spend key for this node.
                let seed = SpendSeed(OsRng.gen());
                let spend_key = SpendKey::from(seed.clone());

                // Create signing key and verification key for this node.
                let validator_id_sk = spend_key.spend_auth_key();
                let validator_id_vk = VerificationKey::from(validator_id_sk);

                // generate consensus key for tendermint.
                let validator_cons_sk =
                    tendermint::PrivateKey::Ed25519(ed25519_consensus::SigningKey::new(OsRng));
                let validator_cons_pk = validator_cons_sk.public_key();

                // generate P2P auth key for tendermint.
                let node_key_sk =
                    tendermint::PrivateKey::Ed25519(ed25519_consensus::SigningKey::new(OsRng));
                let node_key_pk = node_key_sk.public_key();

                let vk = ValidatorKeys {
                    validator_id_sk: validator_id_sk.clone(),
                    validator_id_vk,
                    validator_cons_sk,
                    validator_cons_pk,
                    node_key_sk,
                    node_key_pk,
                    validator_spendseed: seed,
                };

                let fvk = spend_key.full_viewing_key();
                let ivk = fvk.incoming();
                let (dest, _dtk_d) = ivk.payment_address(0u64.into());

                // Add a default 1 upenumbra allocation to the validator.
                allocations.push(Allocation {
                    address: dest,
                    amount: 1,
                    denom: "upenumbra".to_string(),
                });

                validator_keys.push(vk);
            }

            for (n, vk) in validator_keys.iter().enumerate() {
                let node_name = format!("node{}", n);

                let app_state = genesis::AppState {
                    allocations: allocations.clone(),
                    chain_params: ChainParams {
                        chain_id: chain_id.clone(),
                        epoch_duration,
                        unbonding_epochs,
                        active_validator_limit,
                        slashing_penalty,
                        ibc_enabled: false,
                        inbound_ics20_transfers_enabled: false,
                        outbound_ics20_transfers_enabled: false,
                    },
                    validators: validators
                        .iter()
                        .map(|v| {
                            Ok(ValidatorPower {
                                validator: Validator {
                                    // Currently there's no way to set validator keys beyond
                                    // manually editing the genesis.json. Otherwise they
                                    // will be randomly generated keys.
                                    identity_key: IdentityKey(vk.validator_id_vk),
                                    consensus_key: vk.validator_cons_pk,
                                    name: v.name.clone(),
                                    website: v.website.clone(),
                                    description: v.description.clone(),
                                    funding_streams: FundingStreams::try_from(
                                        v.funding_streams
                                            .iter()
                                            .map(|fs| {
                                                Ok(FundingStream {
                                            address: Address::from_str(&fs.address).map_err(|_|
                                                anyhow::anyhow!("invalid funding stream address in validators.json"),
                                            )?,
                                            rate_bps: fs.rate_bps,
                                        })
                                            })
                                            .collect::<Result<Vec<FundingStream>, anyhow::Error>>()?,
                                    )
                                    .map_err(|_|
                                        anyhow::anyhow!("unable to construct funding streams from validators.json"),
                                    )?,
                                    sequence_number: v.sequence_number,
                                },
                                power: v.voting_power.into(),
                            })
                        })
                        .collect::<Result<Vec<ValidatorPower>,anyhow::Error>>()?,
                };

                // Create the directory for this node
                let mut node_dir = output_dir.clone();
                node_dir.push(&node_name);

                let mut node_config_dir = node_dir.clone();
                node_config_dir.push("config");

                let mut node_data_dir = node_dir.clone();
                node_data_dir.push("data");

                fs::create_dir_all(&node_config_dir)?;
                fs::create_dir_all(&node_data_dir)?;

                // Write this node's tendermint genesis.json file
                let validator_genesis = Genesis {
                    genesis_time,
                    chain_id: chain_id
                        .parse::<tendermint::chain::Id>()
                        .expect("able to create chain ID"),
                    initial_height: 0,
                    consensus_params: tendermint::consensus::Params {
                        block: tendermint::block::Size {
                            max_bytes: 22020096,
                            max_gas: -1,
                            // minimum time increment between consecutive blocks
                            time_iota_ms: 500,
                        },
                        // TODO Should these correspond with values used within `pd` for penumbra epochs?
                        evidence: tendermint::evidence::Params {
                            max_age_num_blocks: 100000,
                            // 1 day
                            max_age_duration: tendermint::evidence::Duration(Duration::new(
                                86400, 0,
                            )),
                            max_bytes: 1048576,
                        },
                        validator: tendermint::consensus::params::ValidatorParams {
                            pub_key_types: vec![Algorithm::Ed25519],
                        },
                        version: Some(tendermint::consensus::params::VersionParams {
                            app_version: 0,
                        }),
                    },
                    // always empty in genesis json
                    app_hash: vec![],
                    app_state,
                    // List of initial validators. Note this may be overridden entirely by
                    // the application, and may be left empty to make explicit that the
                    // application will initialize the validator set with ResponseInitChain.
                    // - https://docs.tendermint.com/v0.32/tendermint-core/using-tendermint.html
                    // For penumbra, we can leave this empty since the app_state also contains Validator
                    // configs.
                    validators: vec![],
                };
                let mut genesis_file_path = node_config_dir.clone();
                genesis_file_path.push("genesis.json");
                println!(
                    "Writing {} genesis file to: {}",
                    &node_name,
                    genesis_file_path.display()
                );
                let mut genesis_file = File::create(genesis_file_path)?;
                genesis_file
                    .write_all(serde_json::to_string_pretty(&validator_genesis)?.as_bytes())?;

                // Write this node's config.toml
                // Note that this isn't a re-implementation of the `Config` type from
                // Tendermint (https://github.com/tendermint/tendermint/blob/6291d22f46f4c4f9121375af700dbdafa51577e7/config/config.go#L92)
                // so if they change their defaults or the available fields, that won't be reflected in our template.
                let tm_config = generate_tm_config(&node_name);
                let mut config_file_path = node_config_dir.clone();
                config_file_path.push("config.toml");
                println!(
                    "Writing {} config file to: {}",
                    &node_name,
                    config_file_path.display()
                );
                let mut config_file = File::create(config_file_path)?;
                config_file.write_all(tm_config.as_bytes())?;

                // Write this node's node_key.json
                // the underlying type doesn't implement Copy or Clone (for the best)
                let priv_key = tendermint::PrivateKey::Ed25519(
                    vk.node_key_sk.ed25519_signing_key().unwrap().clone(),
                );
                let node_key = NodeKey { priv_key };
                let mut node_key_file_path = node_config_dir.clone();
                node_key_file_path.push("node_key.json");
                println!(
                    "Writing {} node key file to: {}",
                    &node_name,
                    node_key_file_path.display()
                );
                let mut node_key_file = File::create(node_key_file_path)?;
                node_key_file.write_all(serde_json::to_string_pretty(&node_key)?.as_bytes())?;

                // Write this node's priv_validator_key.json
                let address: Id = vk.validator_cons_pk.into();

                // the underlying type doesn't implement Copy or Clone (for the best)
                let priv_key = tendermint::PrivateKey::Ed25519(
                    vk.validator_cons_sk.ed25519_signing_key().unwrap().clone(),
                );
                let priv_validator_key = PrivValidatorKey {
                    address,
                    pub_key: vk.validator_cons_pk,
                    priv_key,
                };
                let mut priv_validator_key_file_path = node_config_dir.clone();
                priv_validator_key_file_path.push("priv_validator_key.json");
                println!(
                    "Writing {} priv validator key file to: {}",
                    &node_name,
                    priv_validator_key_file_path.display()
                );
                let mut priv_validator_key_file = File::create(priv_validator_key_file_path)?;
                priv_validator_key_file
                    .write_all(serde_json::to_string_pretty(&priv_validator_key)?.as_bytes())?;

                // Write the initial validator state:
                let mut priv_validator_state_file_path = node_data_dir.clone();
                priv_validator_state_file_path.push("priv_validator_state.json");
                println!(
                    "Writing {} priv validator state file to: {}",
                    &node_name,
                    priv_validator_state_file_path.display()
                );
                let mut priv_validator_state_file = File::create(priv_validator_state_file_path)?;
                priv_validator_state_file.write_all(get_validator_state().as_bytes())?;

                // Write the validator's signing key:
                let mut validator_signingkey_file_path = node_config_dir.clone();
                validator_signingkey_file_path.push("validator_signingkey.json");
                println!(
                    "Writing {} validator signing key file to: {}",
                    &node_name,
                    validator_signingkey_file_path.display()
                );
                let mut validator_signingkey_file = File::create(validator_signingkey_file_path)?;
                validator_signingkey_file
                    .write_all(serde_json::to_string_pretty(&vk.validator_id_sk)?.as_bytes())?;

                // Write the validator's spend seed:
                let mut validator_spendseed_file_path = node_config_dir.clone();
                validator_spendseed_file_path.push("validator_spendseed.json");
                println!(
                    "Writing {} validator spend seed file to: {}",
                    &node_name,
                    validator_spendseed_file_path.display()
                );
                let mut validator_spendseed_file = File::create(validator_spendseed_file_path)?;
                validator_spendseed_file
                    .write_all(serde_json::to_string_pretty(&vk.validator_spendseed)?.as_bytes())?;

                println!("-------------------------------------");
            }
        }
    }

    Ok(())
}
