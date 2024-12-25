use anyhow::Context;
use decaf377_rdsa::{SigningKey, SpendAuth, VerificationKey};
use directories::UserDirs;
use penumbra_sdk_app::genesis::AppState;
use penumbra_sdk_custody::soft_kms::Config as SoftKmsConfig;
use penumbra_sdk_keys::keys::{SpendKey, SpendKeyBytes};
use rand::Rng;
use rand_core::OsRng;
use regex::{Captures, Regex};
use serde::Deserialize;
use std::{
    env::current_dir,
    fs::{self, File},
    io::Write,
    net::SocketAddr,
    path::PathBuf,
    str::FromStr,
};
use tendermint::{node::Id, Genesis, Moniker, PrivateKey};
use tendermint_config::{
    net::Address as TendermintAddress, NodeKey, PrivValidatorKey, TendermintConfig,
};
use url::Url;

use crate::network::generate::NetworkValidator;

/// Wrapper for a [TendermintConfig], with a constructor for convenient defaults.
pub struct NetworkTendermintConfig(pub TendermintConfig);

impl NetworkTendermintConfig {
    /// Use a hard-coded Tendermint config as a base template, substitute
    /// values via a typed interface, and rerender as TOML.
    pub fn new(
        node_name: &str,
        peers: Vec<TendermintAddress>,
        external_address: Option<TendermintAddress>,
        tm_rpc_bind: Option<SocketAddr>,
        tm_p2p_bind: Option<SocketAddr>,
    ) -> anyhow::Result<Self> {
        tracing::debug!("List of CometBFT peers: {:?}", peers);
        let moniker: Moniker = Moniker::from_str(node_name)?;
        let mut tm_config = TendermintConfig::parse_toml(include_str!(
            "../../../../../testnets/cometbft_config_template.toml"
        ))
        .context("Failed to parse the TOML config template for CometBFT")?;
        tm_config.moniker = moniker;
        tm_config.p2p.seeds = peers;
        tracing::debug!("External address looks like: {:?}", external_address);
        tm_config.p2p.external_address = external_address;
        // The CometBFT config wants URLs, not SocketAddrs, so we'll prepend protocol.
        if let Some(rpc) = tm_rpc_bind {
            tm_config.rpc.laddr =
                parse_tm_address(None, &Url::parse(format!("tcp://{}", rpc).as_str())?)?;
        }
        if let Some(p2p) = tm_p2p_bind {
            tm_config.p2p.laddr =
                parse_tm_address(None, &Url::parse(format!("tcp://{}", p2p).as_str())?)?;
        }

        Ok(Self(tm_config))
    }
}

impl NetworkTendermintConfig {
    /// Write Tendermint config files to disk. This includes not only the `config.toml` file,
    /// but also all keypairs required for node and/or validator identity.
    pub fn write_config(
        &self,
        node_dir: PathBuf,
        v: &NetworkValidator,
        genesis: &Genesis<AppState>,
    ) -> anyhow::Result<()> {
        // We'll also create the pd state directory here, since it's convenient.
        let pd_dir = node_dir.clone().join("pd");
        let cb_data_dir = node_dir.clone().join("cometbft").join("data");
        let cb_config_dir = node_dir.clone().join("cometbft").join("config");

        tracing::info!(config_dir = %node_dir.display(), "Writing validator configs to");

        fs::create_dir_all(pd_dir)?;
        fs::create_dir_all(&cb_data_dir)?;
        fs::create_dir_all(&cb_config_dir)?;

        let genesis_file_path = cb_config_dir.clone().join("genesis.json");
        tracing::debug!(genesis_file_path = %genesis_file_path.display(), "writing genesis");
        let mut genesis_file = File::create(genesis_file_path)?;
        genesis_file.write_all(serde_json::to_string_pretty(&genesis)?.as_bytes())?;

        let cb_config_filepath = cb_config_dir.clone().join("config.toml");
        tracing::debug!(cometbft_config = %cb_config_filepath.display(), "writing cometbft config.toml");
        let mut cb_config_file = File::create(cb_config_filepath)?;
        cb_config_file.write_all(toml::to_string(&self.0)?.as_bytes())?;

        // Write this node's node_key.json
        // the underlying type doesn't implement Copy or Clone (for the best)
        let priv_key = tendermint::PrivateKey::Ed25519(
            v.keys
                .node_key_sk
                .ed25519_signing_key()
                .expect("node key has ed25519 signing key")
                .clone(),
        );

        let node_key = NodeKey { priv_key };
        let cb_node_key_filepath = cb_config_dir.clone().join("node_key.json");
        tracing::debug!(cb_node_key_filepath = %cb_node_key_filepath.display(), "writing node key file");
        let mut cb_node_key_file = File::create(cb_node_key_filepath)?;
        cb_node_key_file.write_all(serde_json::to_string_pretty(&node_key)?.as_bytes())?;

        // Write this node's priv_validator_key.json
        let priv_validator_key_filepath = cb_config_dir.clone().join("priv_validator_key.json");
        tracing::debug!(priv_validator_key_filepath = %priv_validator_key_filepath.display(), "writing validator private key");
        let mut priv_validator_key_file = File::create(priv_validator_key_filepath)?;
        let priv_validator_key: PrivValidatorKey = v.keys.priv_validator_key()?;
        priv_validator_key_file
            .write_all(serde_json::to_string_pretty(&priv_validator_key)?.as_bytes())?;

        // Write the initial validator state:
        let priv_validator_state_filepath = cb_data_dir.clone().join("priv_validator_state.json");
        tracing::debug!(priv_validator_state_filepath = %priv_validator_state_filepath.display(), "writing validator state");
        let mut priv_validator_state_file = File::create(priv_validator_state_filepath)?;
        priv_validator_state_file.write_all(NetworkValidator::initial_state().as_bytes())?;

        // Write the validator's spend key:
        let validator_spend_key_filepath = cb_config_dir.clone().join("validator_custody.json");
        tracing::debug!(validator_spend_key_filepath = %validator_spend_key_filepath.display(), "writing validator custody file");
        let mut validator_spend_key_file = File::create(validator_spend_key_filepath)?;
        let validator_wallet = SoftKmsConfig::from(
            SpendKey::try_from(v.keys.validator_spend_key.clone())
                .expect("spend key should be valid"),
        );
        validator_spend_key_file
            .write_all(toml::to_string_pretty(&validator_wallet)?.as_bytes())?;

        Ok(())
    }
}

/// Construct a [`tendermint_config::net::Address`] from an optional node [`Id`] and `node_address`.
/// The `node_address` can be an IP address or a hostname. Supports custom ports, defaulting
/// to 26656 if not specified.
pub fn parse_tm_address(
    node_id: Option<&Id>,
    node_address: &Url,
) -> anyhow::Result<TendermintAddress> {
    let hostname = match node_address.host() {
        Some(h) => h,
        None => {
            anyhow::bail!(format!("Could not find hostname in URL: {}", node_address))
        }
    };
    // Default to 26656 for Tendermint port, if not specified.
    let port = node_address.port().unwrap_or(26656);
    match node_id {
        Some(id) => Ok(format!("{id}@{hostname}:{port}").parse()?),
        None => Ok(format!("{hostname}:{port}").parse()?),
    }
}

/// Collection of all keypairs required for a Penumbra validator.
/// Used to generate a stable identity for a [`NetworkValidator`].
#[derive(Deserialize)]
pub struct ValidatorKeys {
    /// Penumbra spending key and viewing key for this node.
    pub validator_id_sk: SigningKey<SpendAuth>,
    pub validator_id_vk: VerificationKey<SpendAuth>,
    pub validator_spend_key: SpendKeyBytes,
    /// Consensus key for tendermint.
    pub validator_cons_sk: tendermint::PrivateKey,
    pub validator_cons_pk: tendermint::PublicKey,
    /// P2P auth key for tendermint.
    pub node_key_sk: tendermint::PrivateKey,
    #[allow(unused_variables, dead_code)]
    pub node_key_pk: tendermint::PublicKey,
}

impl ValidatorKeys {
    /// Use a hard-coded seed to generate a new set of validator keys.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        // Create the spend key for this node.
        let seed = SpendKeyBytes(seed);
        let spend_key = SpendKey::from(seed.clone());

        // Create signing key and verification key for this node.
        let validator_id_sk = spend_key.spend_auth_key();
        let validator_id_vk = VerificationKey::from(validator_id_sk);

        let validator_cons_sk = ed25519_consensus::SigningKey::new(OsRng);

        // generate consensus key for tendermint.
        let validator_cons_sk = tendermint::PrivateKey::Ed25519(
            validator_cons_sk
                .as_bytes()
                .as_slice()
                .try_into()
                .expect("32 bytes"),
        );
        let validator_cons_pk = validator_cons_sk.public_key();

        // generate P2P auth key for tendermint.
        let node_key_sk = ed25519_consensus::SigningKey::from(seed.0);
        let signing_key_bytes = node_key_sk.as_bytes().as_slice();

        // generate consensus key for tendermint.
        let node_key_sk =
            tendermint::PrivateKey::Ed25519(signing_key_bytes.try_into().expect("32 bytes"));
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
            validator_cons_sk
                .as_bytes()
                .as_slice()
                .try_into()
                .expect("32 bytes"),
        );
        let validator_cons_pk = validator_cons_sk.public_key();

        // generate P2P auth key for tendermint.
        let node_key_sk = ed25519_consensus::SigningKey::from(seed.0);
        let signing_key_bytes = node_key_sk.as_bytes().as_slice();

        // generate consensus key for tendermint.
        let node_key_sk =
            tendermint::PrivateKey::Ed25519(signing_key_bytes.try_into().expect("32 bytes"));
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
    /// Format the p2p consensus keypair into a struct suitable for serialization
    /// directly as `priv_validator_key.json` for Tendermint config.
    pub fn priv_validator_key(&self) -> anyhow::Result<PrivValidatorKey> {
        let address: tendermint::account::Id = self.validator_cons_pk.into();
        let priv_key = tendermint::PrivateKey::Ed25519(
            self.validator_cons_sk
                .ed25519_signing_key()
                .ok_or_else(|| {
                    anyhow::anyhow!("Failed during loop of signing key for NetworkValidator")
                })?
                .clone(),
        );
        let priv_validator_key = PrivValidatorKey {
            address,
            pub_key: self.validator_cons_pk,
            priv_key,
        };
        Ok(priv_validator_key)
    }
}

impl Default for ValidatorKeys {
    fn default() -> Self {
        Self::generate()
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
    let tilde = Regex::new(r"^~(/|$)").expect("tilde regex is valid");
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
        PathBuf::from(format!(
            "{}/{}",
            current_dir()
                .expect("current working dir is valid")
                .display(),
            input
        ))
    }
}

/// Convert an optional CLI arg into a [`PathBuf`], defaulting to
/// `~/.penumbra/network_data`.
pub fn get_network_dir(network_dir: Option<PathBuf>) -> PathBuf {
    // By default output directory will be in `~/.penumbra/network_data/`
    match network_dir {
        Some(o) => o,
        None => canonicalize_path("~/.penumbra/network_data"),
    }
}

/// Check that a [Url] has all the necessary parts defined for use as a CLI arg.
pub fn url_has_necessary_parts(url: &Url) -> bool {
    url.scheme() != "" && url.has_host() && url.port().is_some()
}
