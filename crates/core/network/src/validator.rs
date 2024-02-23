//! Representation of a Penumbra Validator, specifically
//! for use in configuration files for networks.

use crate::fullnode::PenumbraNode;
use anyhow::{Context, Result};

use penumbra_keys::{keys::SpendKey, Address};

use penumbra_shielded_pool::genesis::Allocation;
use penumbra_stake::{
    validator::Validator as StakeValidator, DelegationToken, FundingStream, FundingStreams,
    GovernanceKey, IdentityKey,
};
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DefaultOnError;
use std::{fs::File, io::Read, path::PathBuf, str::FromStr};

use tendermint::node;
use tendermint_config::net::Address as TendermintAddress;

/// Represents a Penumbra validator entirely, including all metadata
/// fields, as well as full keypair info. Intended for generating config files.
// We don't use `penumbra_stake::validator::Validator` because it only stores the public
// half of the CometBFT keypair. While we could embed that Validator as a subfield,
// then we'd need full support for `penumbra_stake::FundingStreams`.
#[serde_as]
#[derive(Deserialize)]
pub struct PenumbraValidator {
    pub name: String,
    pub website: String,
    pub description: String,
    pub funding_streams: Vec<crate::generate::FundingStream>,
    /// All validator identities
    pub sequence_number: u32,
    pub peer_address_template: Option<String>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub fullnode: PenumbraNode,
}

impl PenumbraValidator {
    /// Import validator configs from a JSON file.
    pub fn from_json_file(json_filepath: PathBuf) -> Result<Vec<PenumbraValidator>> {
        let validators_file = File::open(&json_filepath)
            .with_context(|| format!("cannot open file {json_filepath:?}"))?;
        Self::from_reader(validators_file)
    }
    /// Import validator configs from a reader object that emits JSON.
    pub fn from_reader(input: impl Read) -> Result<Vec<PenumbraValidator>> {
        // TODO: handle the missing `fullnode` field in the JSON file.
        Ok(serde_json::from_reader(input)?)
    }
    /// Generate iniital delegation allocation for inclusion in genesis.
    pub fn delegation_allocation(&self) -> Result<Allocation> {
        let spend_key = SpendKey::from(self.fullnode.keys.validator_spend_key.clone());
        let fvk = spend_key.full_viewing_key();
        let ivk = fvk.incoming();
        let (dest, _dtk_d) = ivk.payment_address(0u32.into());

        let identity_key: IdentityKey = IdentityKey(*fvk.spend_verification_key());
        let delegation_denom = DelegationToken::from(&identity_key).denom();
        Ok(Allocation {
            address: dest,
            // Add an initial allocation of 25,000 delegation tokens,
            // starting them with 2.5x the individual allocations to discord users.
            // 25,000 delegation tokens * 1e6 udelegation factor
            raw_amount: (25_000 * 10u128.pow(6)).into(),
            raw_denom: delegation_denom.to_string(),
        })
    }
    /// Return a URL for CometBFT P2P service for this node.
    ///
    /// In order for the set of genesis validators to communicate with each other,
    /// they must have initial peer information seeded into their CometBFT config files.
    /// If an `external_address` was set, use that. Next, check for a `peer_address_template`.
    /// Finally, fall back to localhost.
    pub fn peering_address(&self) -> anyhow::Result<TendermintAddress> {
        let tm_node_id = node::Id::from(
            self.fullnode
                .keys
                .node_key_pk
                .ed25519()
                .expect("ed25519 key"),
        );
        let r: TendermintAddress = match &self.fullnode.external_address {
            // The `external_address` is a TendermintAddress, so unpack as enum to retrieve
            // the host/port info.
            Some(a) => match a {
                TendermintAddress::Tcp {
                    peer_id: _,
                    host,
                    port,
                } => format!("{tm_node_id}@{}:{}", host, port).parse()?,
                // The other enum type is TendermintAddress::Unix, see
                // https://docs.rs/tendermint-config/0.33.0/tendermint_config/index.html
                _ => {
                    anyhow::bail!(
                        "Only TCP format is supported for tendermint addresses: {}",
                        a
                    );
                }
            },
            None => match &self.peer_address_template {
                Some(t) => format!("{tm_node_id}@{t}:26656").parse()?,
                None => format!("{tm_node_id}@127.0.0.1:26656").parse()?,
            },
        };
        Ok(r)
    }

    /// Hardcoded initial state for Tendermint, used for writing configs.
    // Easiest to hardcode since we never change these.
    pub fn initial_state() -> String {
        r#"{
        "height": "0",
        "round": 0,
        "step": 0
    }
    "#
        .to_string()
    }
}

impl Default for PenumbraValidator {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            website: "".to_string(),
            description: "".to_string(),
            funding_streams: Vec::<crate::generate::FundingStream>::new(),
            sequence_number: 0,
            peer_address_template: None,
            fullnode: PenumbraNode::default(),
        }
    }
}

/// The core Penumbra application logic deals with [StakeValidator]s, so we must make sure
/// our convenient wrapper type can resolve as a [StakeValidator] when needed.
impl TryFrom<&PenumbraValidator> for StakeValidator {
    type Error = anyhow::Error;
    fn try_from(tv: &PenumbraValidator) -> anyhow::Result<StakeValidator> {
        Ok(StakeValidator {
            // Currently there's no way to set validator keys beyond
            // manually editing the genesis.json. Otherwise they
            // will be randomly generated keys.
            identity_key: IdentityKey(tv.fullnode.keys.validator_id_vk),
            governance_key: GovernanceKey(tv.fullnode.keys.validator_id_vk),
            consensus_key: tv.fullnode.keys.validator_cons_pk,
            name: tv.name.clone(),
            website: tv.website.clone(),
            description: tv.description.clone(),
            enabled: true,
            funding_streams: FundingStreams::try_from(
                tv.funding_streams
                    .iter()
                    .map(|fs| {
                        Ok(FundingStream::ToAddress {
                            address: Address::from_str(&fs.address)
                                .context("invalid funding stream address in validators.json")?,
                            rate_bps: fs.rate_bps,
                        })
                    })
                    .collect::<Result<Vec<FundingStream>, anyhow::Error>>()?,
            )
            .context("unable to construct funding streams from validators.json")?,
            sequence_number: tv.sequence_number,
        })
    }
}

impl TryInto<StakeValidator> for PenumbraValidator {
    type Error = anyhow::Error;
    fn try_into(self) -> anyhow::Result<StakeValidator> {
        Ok(StakeValidator {
            // Currently there's no way to set validator keys beyond
            // manually editing the genesis.json. Otherwise they
            // will be randomly generated keys.
            identity_key: IdentityKey(self.fullnode.keys.validator_id_vk),
            governance_key: GovernanceKey(self.fullnode.keys.validator_id_vk),
            consensus_key: self.fullnode.keys.validator_cons_pk,
            name: self.name.clone(),
            website: self.website.clone(),
            description: self.description.clone(),
            enabled: true,
            funding_streams: FundingStreams::try_from(
                self.funding_streams
                    .iter()
                    .map(|fs| {
                        Ok(FundingStream::ToAddress {
                            address: Address::from_str(&fs.address)
                                .context("invalid funding stream address in validators.json")?,
                            rate_bps: fs.rate_bps,
                        })
                    })
                    .collect::<Result<Vec<FundingStream>, anyhow::Error>>()?,
            )
            .context("unable to construct funding streams from validators.json")?,
            sequence_number: self.sequence_number,
        })
    }
}
