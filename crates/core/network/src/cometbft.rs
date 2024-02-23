//! Configuration logic for CometBFT nodes used in Penumbra.

use rand::Rng;
use rand_core::OsRng;

use anyhow::Context;

use serde::Serialize;
use std::{net::SocketAddr, str::FromStr};
use tendermint::Moniker;
use tendermint_config::{net::Address as TendermintAddress, TendermintConfig};

use crate::fullnode::PenumbraNode;
use crate::validator::PenumbraValidator;

/// Wrapper for a [TendermintConfig], with a constructor for convenient defaults.
#[derive(Serialize)]
pub struct PenumbraCometBFTConfig(pub TendermintConfig);

impl PenumbraCometBFTConfig {
    /// Generate a new CometBFT config for use with Penumbra, based on a hard-coded template file.
    pub fn new(
        node_name: &str,
        peers: Vec<TendermintAddress>,
        external_address: Option<TendermintAddress>,
        cmt_rpc_bind: Option<SocketAddr>,
        cmt_p2p_bind: Option<SocketAddr>,
    ) -> anyhow::Result<Self> {
        tracing::debug!("List of CometBFT peers: {:?}", peers);
        let mut c = Self::default();
        c.0.moniker = node_name.parse()?;
        c.0.p2p.seeds = peers;
        c.0.p2p.external_address = external_address;
        // The CometBFT config wants URLs, not SocketAddrs, so we'll prepend protocol.
        if let Some(a) = cmt_rpc_bind {
            c.0.rpc.laddr = format!("tcp://{}", a)
                .parse()
                .context("can specify localhost bind addr for rpc")?
        }
        if let Some(a) = cmt_p2p_bind {
            c.0.p2p.laddr = format!("tcp://{}", a)
                .parse()
                .context("can specify localhost bind addr for p2p")?;
        }
        Ok(Self(c.0))
    }
    /// Generate unique moniker
    pub fn moniker() -> Moniker {
        Moniker::from_str(
            format!("node-{}", hex::encode(OsRng.gen::<u32>().to_le_bytes())).as_str(),
        )
        .expect("can create moniker from str")
    }
}

impl Default for PenumbraCometBFTConfig {
    fn default() -> Self {
        let mut cmt_config =
            TendermintConfig::parse_toml(include_str!("../files/cometbft_config_template.toml"))
                .expect("failed to parse the TOML config template for CometBFT");

        cmt_config.moniker = Self::moniker();
        cmt_config.p2p.seeds = vec![];
        cmt_config.p2p.external_address = None;

        Self(cmt_config)
    }
}

impl TryFrom<PenumbraNode> for PenumbraCometBFTConfig {
    type Error = anyhow::Error;

    fn try_from(fullnode: PenumbraNode) -> anyhow::Result<Self> {
        let mut c = Self::default();
        c.0.moniker = fullnode.moniker;
        c.0.p2p.external_address = fullnode.external_address;
        Ok(c)
    }
}

impl TryFrom<PenumbraValidator> for PenumbraCometBFTConfig {
    type Error = anyhow::Error;
    fn try_from(validator: PenumbraValidator) -> anyhow::Result<Self> {
        let mut c = Self::default();
        c.0.moniker = validator.fullnode.moniker;
        c.0.p2p.external_address = validator.fullnode.external_address;
        Ok(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Confirm that we can parse a CometBFT net address from all the formats
    /// we expect to encounter, whether interpreting CLI args, or inspecting
    /// JSON-RPC results.
    fn parse_cometbft_address() -> anyhow::Result<()> {
        // Parse protocol, IP, and port, but no Id.
        let s1 = "tcp://35.226.255.25:26656";
        let cmt1: TendermintAddress = s1.parse()?;
        match cmt1 {
            TendermintAddress::Tcp {
                peer_id,
                host,
                port,
            } => {
                assert!(peer_id.is_none());
                assert!(port == 26656);
                assert!(host == "35.226.255.25");
            }
            _ => {
                anyhow::bail!("cometbft address did not parse as tcp")
            }
        }

        // Parse IP and port, without protocol or Id.
        let s2 = "35.226.255.25:26656";
        let cmt2: TendermintAddress = s2.parse()?;
        match cmt2 {
            TendermintAddress::Tcp {
                peer_id,
                host,
                port,
            } => {
                assert!(peer_id.is_none());
                assert!(port == 26656);
                assert!(host == "35.226.255.25");
            }
            _ => {
                anyhow::bail!("cometbft address did not parse as tcp")
            }
        }

        // Parse node id along with IP and port.
        let s3 = "71e9f0d989d30d11e39542baf162e91297f5103e@35.226.255.25:26656";
        let cmt3: TendermintAddress = s3.parse()?;

        match cmt3 {
            TendermintAddress::Tcp {
                peer_id,
                host,
                port,
            } => {
                assert!(peer_id.is_some());
                assert_eq!(
                    peer_id.unwrap(),
                    "71e9f0d989d30d11e39542baf162e91297f5103e".parse()?
                );
                assert!(port == 26656);
                assert!(host == "35.226.255.25");
            }
            _ => {
                anyhow::bail!("cometbft address did not parse as tcp")
            }
        }
        Ok(())
    }
}
