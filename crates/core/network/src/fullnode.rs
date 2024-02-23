//! Logic for creating a Penumbra fullnode.
//!
//! Includes all key material for both Penumbra and CometBFT functionality,
//! such as spends, consensus, and optional promotion to validator.
use anyhow::Context;
use decaf377_rdsa::{SigningKey, SpendAuth, VerificationKey};

use crate::cometbft::PenumbraCometBFTConfig;
use penumbra_custody::soft_kms::Config as SoftKmsConfig;
use penumbra_genesis::AppState;
use penumbra_keys::keys::{SpendKey, SpendKeyBytes};
use rand::Rng;
use rand_core::OsRng;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::validator::PenumbraValidator;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use tendermint::Genesis;
use tendermint::Moniker;
use tendermint_config::net::Address as TendermintAddress;
use tendermint_config::NodeKey;
use tendermint_config::PrivValidatorKey;

/// A Penumbra fullnode. The struct also contains all key material required
/// for a [crate::validator::PenumbraValidator], but in Penumbra, a fullnode
/// is only promoted to Validator status by using `pcli`.
#[derive(Deserialize, Serialize)]
pub struct PenumbraNode {
    /// All key material for consensus, P2P communication, and signing.
    pub keys: PenumbraNodeKeys,
    /// Human-readable name for node
    pub moniker: Moniker,
    /// Optional `external_address` field for CometBFT config.
    /// Instructs peers to connect to this node's P2P service
    /// on this address.
    pub external_address: Option<TendermintAddress>,

    /// The local socket for binding the CometBFT RPC service,
    /// used for generating the [PenumbraCometBFTConfig].
    pub cometbft_rpc_bind: SocketAddr,
    /// The local socket for binding the CometBFT P2P service,
    /// used for generating the [PenumbraCometBFTConfig].
    pub cometbft_p2p_bind: SocketAddr,
}

impl Default for PenumbraNode {
    fn default() -> Self {
        Self {
            moniker: PenumbraCometBFTConfig::moniker(),
            keys: PenumbraNodeKeys::default(),
            external_address: None,
            cometbft_rpc_bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 26657),
            cometbft_p2p_bind: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 26656),
        }
    }
}

impl PenumbraNode {
    /// Write CometBFT config files to disk. This includes not only the `config.toml` file,
    /// but also all keypairs required for node and/or validator identity.
    pub fn write_config(
        &self,
        node_dir: PathBuf,
        genesis: &Genesis<AppState>,
        peers: Option<Vec<TendermintAddress>>,
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

        let mut cmt_config = self.cometbft_config()?;
        cmt_config.0.p2p.seeds = peers.unwrap_or_default();

        cb_config_file.write_all(toml::to_string(&cmt_config)?.as_bytes())?;

        // Write this node's node_key.json
        // the underlying type doesn't implement Copy or Clone (for the best)
        let priv_key = tendermint::PrivateKey::Ed25519(
            self.keys
                .node_key_sk
                .ed25519_signing_key()
                .expect("node key has ed25519 signing key")
                .clone(),
        );

        tracing::debug!("writing validator key files");
        let node_key = NodeKey { priv_key };
        let mut cb_node_key_file = File::create(cb_config_dir.clone().join("node_key.json"))?;
        cb_node_key_file.write_all(serde_json::to_string_pretty(&node_key)?.as_bytes())?;

        // Write this node's priv_validator_key.json
        let mut priv_validator_key_file =
            File::create(cb_config_dir.clone().join("priv_validator_key.json"))?;
        let priv_validator_key: PrivValidatorKey = self.keys.priv_validator_key()?;
        priv_validator_key_file
            .write_all(serde_json::to_string_pretty(&priv_validator_key)?.as_bytes())?;

        // Write the initial validator state:
        let mut priv_validator_state_file =
            File::create(cb_data_dir.clone().join("priv_validator_state.json"))?;
        priv_validator_state_file.write_all(PenumbraValidator::initial_state().as_bytes())?;

        // Write the validator's spend key:
        let mut validator_spend_key_file =
            File::create(cb_config_dir.clone().join("validator_custody.json"))?;
        let validator_wallet =
            SoftKmsConfig::from(SpendKey::from(self.keys.validator_spend_key.clone()));
        validator_spend_key_file
            .write_all(toml::to_string_pretty(&validator_wallet)?.as_bytes())?;

        Ok(())
    }

    /// Generate a [PenumbraCometBFTConfig] based on the node settings.
    pub fn cometbft_config(&self) -> anyhow::Result<PenumbraCometBFTConfig> {
        let mut c = PenumbraCometBFTConfig::default();
        c.0.moniker = self.moniker.clone();
        c.0.p2p.external_address = self.external_address.clone();
        c.0.rpc.laddr = format!("tcp://{}", self.cometbft_rpc_bind)
            .parse()
            .context("failed to parse rpc bind addr for cometbft")?;
        c.0.p2p.laddr = format!("tcp://{}", self.cometbft_p2p_bind)
            .parse()
            .context("failed to parse p2p bind addr for cometbft")?;
        Ok(c)
    }
}

/// Collection of all keypairs required for a Penumbra fullnode and/or validator.
/// The [PenumbraNode] type is the recommended interface for wrangling these keypairs.
#[derive(Deserialize, Serialize)]
pub struct PenumbraNodeKeys {
    /// Penumbra spending key and viewing key for this node.
    pub validator_id_sk: SigningKey<SpendAuth>,
    pub validator_id_vk: VerificationKey<SpendAuth>,
    pub validator_spend_key: SpendKeyBytes,
    /// Validator consensus key for CometBFT.
    pub validator_cons_sk: tendermint::PrivateKey,
    pub validator_cons_pk: tendermint::PublicKey,
    /// P2P authentication key for CometBFT.
    pub node_key_sk: tendermint::PrivateKey,
    #[allow(unused_variables, dead_code)]
    pub node_key_pk: tendermint::PublicKey,
}

impl PenumbraNodeKeys {
    /// Format the validator keypair into a struct suitable for serialization
    /// directly as `priv_validator_key.json` for CometBFT config.
    pub fn priv_validator_key(&self) -> anyhow::Result<PrivValidatorKey> {
        let address: tendermint::account::Id = self.validator_cons_pk.into();
        let priv_key = tendermint::PrivateKey::Ed25519(
            self.validator_cons_sk
                .ed25519_signing_key()
                .ok_or_else(|| {
                    anyhow::anyhow!("Failed during loop of signing key for PenumbraValidator")
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

impl Default for PenumbraNodeKeys {
    fn default() -> Self {
        // Create the spend key for this node.
        // TODO: change to use seed phrase
        let seed = SpendKeyBytes(OsRng.gen());
        let spend_key = SpendKey::from(seed.clone());

        // Create Penumbra signing key and verification key for this node.
        let validator_id_sk = spend_key.spend_auth_key();
        let validator_id_vk = VerificationKey::from(validator_id_sk);
        let validator_cons_sk = ed25519_consensus::SigningKey::new(OsRng);

        // Generate consensus key for CometBFT, in case this node becomes a validator.
        let validator_cons_sk = tendermint::PrivateKey::Ed25519(
            validator_cons_sk
                .as_bytes()
                .as_slice()
                .try_into()
                .expect("32 bytes"),
        );
        let validator_cons_pk = validator_cons_sk.public_key();

        // Generate P2P authentication key for CometBFT.
        let node_key_sk = ed25519_consensus::SigningKey::new(OsRng);
        let signing_key_bytes = node_key_sk.as_bytes().as_slice();

        // Generate node's consensus key for CometBFT.
        let node_key_sk =
            tendermint::PrivateKey::Ed25519(signing_key_bytes.try_into().expect("32 bytes"));
        let node_key_pk = node_key_sk.public_key();

        Self {
            validator_id_sk: *validator_id_sk,
            validator_id_vk,
            validator_cons_sk,
            validator_cons_pk,
            node_key_sk,
            node_key_pk,
            validator_spend_key: seed,
        }
    }
}
