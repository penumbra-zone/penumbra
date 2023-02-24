//! Methods and types used for generating testnet configurations.
//! Mostly relevant until Penumbra reaches mainnet.
use anyhow::Context;
use directories::UserDirs;
use penumbra_chain::genesis::AppState;
use penumbra_crypto::{
    keys::{SpendKey, SpendKeyBytes},
    rdsa::{SigningKey, SpendAuth, VerificationKey},
};
use penumbra_wallet::KeyStore;
use rand::Rng;
use rand_core::OsRng;
use regex::{Captures, Regex};
use serde::Deserialize;
use std::{
    env::current_dir,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    str::FromStr,
};
use tendermint::{node::Id, Genesis, Moniker, PrivateKey};
use tendermint_config::{
    net::Address as TendermintAddress, NodeKey, PrivValidatorKey, TendermintConfig,
};

pub mod generate;
pub mod join;

/// Use a hard-coded Tendermint config as a base template, substitute
/// values via a typed interface, and rerender as TOML.
pub fn generate_tm_config(
    node_name: &str,
    peers: Vec<TendermintAddress>,
    external_address: Option<TendermintAddress>,
) -> anyhow::Result<String> {
    tracing::debug!("List of TM peers: {:?}", peers);
    let moniker: Moniker = Moniker::from_str(node_name)?;
    let mut tm_config =
        TendermintConfig::parse_toml(include_str!("../../testnets/tm_config_template.toml"))
            .context("Failed to parse the TOML config template for Tendermint")?;
    tm_config.moniker = moniker;
    tm_config.p2p.seeds = peers;
    tracing::debug!("External address looks like: {:?}", external_address);
    tm_config.p2p.external_address = external_address;
    Ok(toml::to_string(&tm_config)?)
}

/// Construct a [`tendermint_config::net::Address`] from an optional node [`Id`] and `node_address`.
/// The `node_address` can be an IP address or a hostname. Supports custom ports, defaulting
/// to 26656 if not specified.
pub fn parse_tm_address(
    node_id: Option<&Id>,
    node_address: &str,
) -> anyhow::Result<TendermintAddress> {
    let mut node = String::from(node_address);
    // Default to 26656 for Tendermint port, if not specified.
    if !node.contains(':') {
        node.push_str(":26656");
    }
    match node_id {
        Some(id) => Ok(format!("{id}@{node}").parse()?),
        None => Ok(node.to_string().parse()?),
    }
}

pub struct ValidatorKeys {
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
    pub validator_spend_key: SpendKeyBytes,
}

impl ValidatorKeys {
    pub fn generate() -> Self {
        // Create the spend key for this node.
        // TODO: change to use seed phrase
        let seed = SpendKeyBytes(OsRng.gen());
        let spend_key = SpendKey::from(seed.clone());

        // Create signing key and verification key for this node.
        let validator_id_sk = spend_key.spend_auth_key();
        let validator_id_vk = VerificationKey::from(validator_id_sk);

        let validator_cons_sk = ed25519_consensus::SigningKey::new(OsRng);

        // generate consensus key for tendermint.
        let validator_cons_sk = tendermint::PrivateKey::Ed25519(
            validator_cons_sk.as_bytes().as_slice().try_into().unwrap(),
        );
        let validator_cons_pk = validator_cons_sk.public_key();

        // generate P2P auth key for tendermint.
        let node_key_sk = ed25519_consensus::SigningKey::new(OsRng);
        let signing_key_bytes = node_key_sk.as_bytes().as_slice();

        // generate consensus key for tendermint.
        let node_key_sk = tendermint::PrivateKey::Ed25519(signing_key_bytes.try_into().unwrap());
        let node_key_pk = node_key_sk.public_key();

        ValidatorKeys {
            validator_id_sk: validator_id_sk.clone(),
            validator_id_vk,
            validator_cons_sk,
            validator_cons_pk,
            node_key_sk,
            node_key_pk,
            validator_spend_key: seed,
        }
    }
}

#[derive(Deserialize)]
pub struct TendermintNodeKey {
    pub id: String,
    pub priv_key: TendermintPrivKey,
}

#[derive(Deserialize)]
pub struct TendermintPrivKey {
    #[serde(rename(serialize = "type"))]
    pub key_type: String,
    pub value: PrivateKey,
}

/// Expand tildes in a path.
/// Modified from `<https://stackoverflow.com/a/68233480>`
pub fn canonicalize_path(input: &str) -> PathBuf {
    let tilde = Regex::new(r"^~(/|$)").unwrap();
    if input.starts_with('/') {
        // if the input starts with a `/`, we use it as is
        input.into()
    } else if tilde.is_match(input) {
        // if the input starts with `~` as first token, we replace
        // this `~` with the user home directory
        PathBuf::from(&*tilde.replace(input, |c: &Captures| {
            if let Some(user_dirs) = UserDirs::new() {
                format!("{}{}", user_dirs.home_dir().to_string_lossy(), &c[1],)
            } else {
                c[0].to_string()
            }
        }))
    } else {
        PathBuf::from(format!("{}/{}", current_dir().unwrap().display(), input))
    }
}

/// Create local config files for `pd` and `tendermint`.
pub fn write_configs(
    node_dir: PathBuf,
    vk: &ValidatorKeys,
    genesis: &Genesis<AppState>,
    tm_config: String,
) -> anyhow::Result<()> {
    let mut pd_dir = node_dir.clone();
    let mut tm_dir = node_dir;

    pd_dir.push("pd");
    tm_dir.push("tendermint");

    let mut node_config_dir = tm_dir.clone();
    node_config_dir.push("config");

    let mut node_data_dir = tm_dir.clone();
    node_data_dir.push("data");

    fs::create_dir_all(&node_config_dir)?;
    fs::create_dir_all(&node_data_dir)?;
    fs::create_dir_all(&pd_dir)?;

    let mut genesis_file_path = node_config_dir.clone();
    genesis_file_path.push("genesis.json");
    tracing::info!(genesis_file_path = %genesis_file_path.display(), "writing genesis");
    let mut genesis_file = File::create(genesis_file_path)?;
    genesis_file.write_all(serde_json::to_string_pretty(&genesis)?.as_bytes())?;

    let mut config_file_path = node_config_dir.clone();
    config_file_path.push("config.toml");
    tracing::info!(config_file_path = %config_file_path.display(), "writing tendermint config.toml");
    let mut config_file = File::create(config_file_path)?;
    config_file.write_all(tm_config.as_bytes())?;

    // Write this node's node_key.json
    // the underlying type doesn't implement Copy or Clone (for the best)
    let priv_key =
        tendermint::PrivateKey::Ed25519(vk.node_key_sk.ed25519_signing_key().unwrap().clone());

    let node_key = NodeKey { priv_key };
    let mut node_key_file_path = node_config_dir.clone();
    node_key_file_path.push("node_key.json");
    tracing::info!(node_key_file_path = %node_key_file_path.display(), "writing node key file");
    let mut node_key_file = File::create(node_key_file_path)?;
    node_key_file.write_all(serde_json::to_string_pretty(&node_key)?.as_bytes())?;

    // Write this node's priv_validator_key.json
    let address: tendermint::account::Id = vk.validator_cons_pk.into();
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
    tracing::info!(priv_validator_key_file_path = %priv_validator_key_file_path.display(), "writing validator private key");
    let mut priv_validator_key_file = File::create(priv_validator_key_file_path)?;
    priv_validator_key_file
        .write_all(serde_json::to_string_pretty(&priv_validator_key)?.as_bytes())?;

    // Write the initial validator state:
    let mut priv_validator_state_file_path = node_data_dir.clone();
    priv_validator_state_file_path.push("priv_validator_state.json");
    tracing::info!(priv_validator_state_file_path = %priv_validator_state_file_path.display(), "writing validator state");
    let mut priv_validator_state_file = File::create(priv_validator_state_file_path)?;
    priv_validator_state_file.write_all(get_validator_state().as_bytes())?;

    // Write the validator's spend key:
    let mut validator_spend_key_file_path = node_config_dir.clone();
    validator_spend_key_file_path.push("validator_custody.json");
    tracing::info!(validator_spend_key_file_path = %validator_spend_key_file_path.display(), "writing validator custody file");
    let mut validator_spend_key_file = File::create(validator_spend_key_file_path)?;
    let validator_wallet = KeyStore {
        spend_key: vk.validator_spend_key.clone().into(),
    };
    validator_spend_key_file
        .write_all(serde_json::to_string_pretty(&validator_wallet)?.as_bytes())?;

    Ok(())
}

// Easiest to hardcode since we never change these.
pub fn get_validator_state() -> String {
    r#"{
    "height": "0",
    "round": 0,
    "step": 0
}
"#
    .to_string()
}

/// Convert an optional CLI arg into a [`PathBuf`], defaulting to
/// `~/.penumbra/testnet_data`.
pub fn get_testnet_dir(testnet_dir: Option<PathBuf>) -> PathBuf {
    // By default output directory will be in `~/.penumbra/testnet_data/`
    match testnet_dir {
        Some(o) => o,
        None => canonicalize_path("~/.penumbra/testnet_data"),
    }
}
