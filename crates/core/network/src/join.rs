//! Onboard a new `pd` node onto an existing Penumbra network.
//!
//! Handles generation of config files for `pd` and `cometbft`.
//! Requires a pre-existing remote Penumbra node, represented as a [BootstrapNode],
//! in order to look up peers and retrieve genesis information.
use rand::seq::SliceRandom;
use rand_core::OsRng;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tendermint_config::net::Address as TendermintAddress;
use url::Url;

use crate::fullnode::PenumbraNode;
use crate::validator::PenumbraValidator;

use tendermint::Genesis as CometBFTGenesis;

/// A remote [PenumbraNode], defined by a URL to the CometBFT RPC
/// service.
pub struct BootstrapNode {
    /// URL for the CometBFT RPC service, so that nodes can pull peer info.
    pub cometbft_url: Url,
}

/// A Penumbra node, defined by P2P listener address, as reported by a [BootstrapNode]'s
/// CometBFT RPC.
pub struct Peer {
    /// Human-readable name for the node in CometBFT.
    pub moniker: String,
    /// The public IPv4 address for the CometBFT P2P service.
    /// Required field, because otherwise it cannot be considered a peer.
    pub listen_addr: TendermintAddress,
}

impl Peer {
    /// Create a [Peer] from a JSON value that was returned from a [BootstrapNode]'s CometBFT
    /// RPC URL.
    pub fn from_peer_info_json(peer_info: serde_json::Value) -> anyhow::Result<Self> {
        // Poll RPC once, and store the JSON for subsequent lookups.
        let node_info: tendermint::node::Info = peer_info
            .get("node_info")
            .and_then(|v| serde_json::value::from_value(v.clone()).ok())
            .ok_or_else(|| anyhow::anyhow!("could not parse JSON from response"))?;

        // The `listen_addr` address MAY be reported with a `tcp://` prefix. Strip it if so,
        // otherwise the node_id value will be interpolated in the wrong place.
        let tm1: TendermintAddress = format!("{}@{}", node_info.id, node_info.listen_addr)
            .replace("tcp://", "")
            .parse()?;
        Ok(Self {
            moniker: node_info.moniker.to_string(),
            listen_addr: tm1,
        })
    }
}

impl BootstrapNode {
    /// Bootstrap a connection to a testnet, via a node on that testnet.
    /// Write CometBFT config and keyfiles for the generated node identity
    /// to `output_dir`, including a few peers known to the [BootstrapNode].
    pub async fn join(
        &self,
        output_dir: PathBuf,
        moniker: &str,
        external_address: Option<TendermintAddress>,
        cometbft_rpc_bind: SocketAddr,
        cometbft_p2p_bind: SocketAddr,
    ) -> anyhow::Result<()> {
        let mut node_dir = output_dir;
        node_dir.push("node0");

        let peers = self.peers().await?;
        let genesis = self.genesis().await?;

        let v = PenumbraValidator {
            fullnode: PenumbraNode {
                moniker: moniker.parse()?,
                external_address,
                cometbft_rpc_bind,
                cometbft_p2p_bind,
                ..Default::default()
            },
            ..Default::default()
        };

        v.fullnode.write_config(node_dir, &genesis, Some(peers))?;
        Ok(())
    }

    /// Fetch genesis directly from remote URL.
    pub async fn genesis(&self) -> anyhow::Result<CometBFTGenesis<penumbra_genesis::AppState>> {
        let genesis_url = self.cometbft_url.join("genesis")?;
        tracing::info!(%genesis_url, "fetching genesis");
        // We need to download the genesis data and the node ID from the remote node.
        // TODO: replace with TendermintProxyServiceClient
        let client = reqwest::Client::new();
        let genesis_json = client
            .get(genesis_url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?
            .get_mut("result")
            .and_then(|v| v.get_mut("genesis"))
            .ok_or_else(|| anyhow::anyhow!("could not parse JSON from response"))?
            .take();
        let genesis: CometBFTGenesis<penumbra_genesis::AppState> =
            serde_json::value::from_value(genesis_json)?;
        tracing::info!("fetched genesis");
        Ok(genesis)
    }

    /// Query the [BootstrapNode]'s CometBFT RPC endpoint and return the listener address
    /// for the P2P endpoint, if one is set. Returns an [Option<TendermintAddress>] because
    /// it's possible that no P2P listener is configured.
    pub async fn listen_address(&self) -> anyhow::Result<Option<TendermintAddress>> {
        let client = reqwest::Client::new();
        // Poll RPC once, and store the JSON for subsequent lookups.
        let n: tendermint::node::Info = client
            .get(self.cometbft_url.join("status")?)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?
            .get_mut("result")
            .and_then(|v| v.get_mut("node_info"))
            .and_then(|v| serde_json::value::from_value(v.clone()).ok())
            .ok_or_else(|| anyhow::anyhow!("could not parse JSON from response"))?;

        tracing::debug!(node_id=%n.id, "fetched node id");
        let cmt_addr: TendermintAddress = format!("{}@{}", n.id, n.listen_addr).parse()?;
        tracing::info!(
            %cmt_addr,
            "fetched listener address for bootstrap node"
        );
        Ok(Some(cmt_addr))
    }

    /// Query the [BootstrapNode]'s CometBFT RPC URL and return a list
    /// of [Peer]s found via its `/net_info` route. Only candidates with
    /// a public IPv4 address in their `listen_addr` fields will be considered.
    /// Therefore, addresses like `localhost` or `0.0.0.0` will be excluded.
    /// Additionally, if a [Peer] contains the string "seed" in its moniker,
    /// it will be included.
    pub async fn peers(&self) -> anyhow::Result<Vec<TendermintAddress>> {
        // Look up more peers from the target node, so that generated tendermint config
        // contains multiple addresses, making peering easier.
        let mut peers = Vec::new();
        match self.listen_address().await? {
            Some(l) => peers.push(l),
            None => {
                // We consider it odd that a bootstrap node has a remotely accessible
                // RPC endpoint, but no P2P listener enabled.
                tracing::warn!("failed to find listenaddr for {}", self.cometbft_url)
            }
        }

        // Fetch the `/net_info` route on a CometBFT RPC URL`, and pluck out the list of peers.
        // For each peer, will evaluate whether it has an external listener configured,
        // and if so, include it for consideration.
        let client = reqwest::Client::new();
        let net_info_url = self.cometbft_url.join("net_info")?;
        tracing::debug!(%net_info_url, "Fetching peers of bootstrap node");
        let mut net_info_peers = client
            .get(net_info_url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?
            .get("result")
            .and_then(|v| v.get("peers"))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        if net_info_peers.is_empty() {
            tracing::warn!(
                ?net_info_peers,
                "bootstrap node reported 0 peers; we'll have no way to get blocks"
            );
        }

        // Randomize the ordering of the peer candidates, so that different nodes
        // joining are more likely to get different peers, as we select a subset.
        net_info_peers.shuffle(&mut OsRng);

        // We'll look for a handful of peers and use those in our config.
        // We don't want to naively add all of the bootstrap node's peers,
        // instead trusting gossip to find peers dynamically over time.
        // We'll also special-case nodes that contain the string "seed" in their moniker:
        // those nodes should be assumed to seed nodes. This is optimistic, but should result
        // in more solid peering behavior than the previous strategy of declaring all fullnodes
        // as seeds within the joining node's CometBFT config.
        let threshold = 5;
        let mut seeds = Vec::new();

        for raw_peer in net_info_peers {
            let p = match Peer::from_peer_info_json(raw_peer) {
                Ok(p) => p,
                Err(_) => {
                    tracing::debug!("failed to parse peer candidate net_info, skipping");
                    continue;
                }
            };
            // Filter out addresses that are obviously not external addresses.
            if !(address_could_be_external(p.listen_addr.clone())) {
                tracing::trace!(
                    listen_addr=?p.listen_addr,
                    "skipping candidate peer due to internal listener address"
                );
                continue;
            }
            // A bit of optimism: any node with "seed" in its moniker gets to be a seed.
            if p.moniker.contains("seed") {
                tracing::debug!(
                    peer_cmt_address=?p.listen_addr,
                    moniker=p.moniker,
                    "found self-described seed node in candidate peers"
                );
                seeds.push(p.listen_addr)
            // Otherwise, we check if we've found enough.
            } else if peers.len() <= threshold {
                peers.push(p.listen_addr)
            }
        }
        if peers.len() < threshold && seeds.is_empty() {
            tracing::warn!(
                "bootstrap node reported only {} peers, and 0 seeds; we may have trouble peering",
                peers.len(),
            );
        }
        // TODO handle seeds and peers differently. For now, all peers are used as seeds.
        peers.extend(seeds);

        let peers_logging: Vec<String> = peers.clone().iter().map(|p| p.to_string()).collect();
        tracing::info!(peers=?peers_logging, "network peers for inclusion in generated configs");
        Ok(peers)
    }
}

/// Check whether SocketAddress spec is likely to be externally-accessible.
/// Filters out RFC1918 and loopback addresses. Requires an address and port.
fn address_could_be_external(address: TendermintAddress) -> bool {
    // In order to check public/private status, we must parse the TendermintAddress
    // as a SocketAddr. To do that, we'll need to strip out the protocol and node_id, if any.
    // TODO: wrangle this better, maybe as Tcp enum?
    let mut maybe_socket = format!("{}", &address).replace("tcp://", "");
    maybe_socket = maybe_socket
        .split('@')
        .last()
        .unwrap_or_default()
        .to_string();
    let addr: SocketAddr = match maybe_socket.parse().ok() {
        Some(a) => a,
        None => {
            tracing::debug!(%address, "failed to parse TendermintAddress as SocketAddr");
            return false;
        }
    };

    match addr.ip() {
        IpAddr::V4(ip) => !(ip.is_private() || ip.is_loopback() || ip.is_unspecified()),
        IpAddr::V6(ip) => !(ip.is_loopback() || ip.is_unspecified()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // The '35.226.255.25' IPv4 address used throughout these tests is a reserved
    // GCP IP, used by Penumbra Labs on the testnet cluster.
    #[test]
    fn external_address_detection() -> anyhow::Result<()> {
        assert!(!address_could_be_external("0.0.0.0:80".parse()?));
        assert!(!address_could_be_external("192.168.4.1:26657".parse()?));
        assert!(address_could_be_external("35.226.255.25:26657".parse()?));
        Ok(())
    }

    // Some of these tests duplicate tests in the upstream Tendermint crates.
    // The underlying structs upstream are mostly just Strings, though, and since
    // our code wrangles these types for interpolation in config files from multiple
    // API endpoint sources, it's important to validate expected behavior.
    #[test]
    fn parse_cmt_addr_tcp() -> anyhow::Result<()> {
        let tm1: TendermintAddress = "tcp://35.226.255.25:26656".parse()?;
        if let TendermintAddress::Tcp {
            peer_id,
            host,
            port,
        } = tm1
        {
            assert!(peer_id.is_none());
            assert!(port == 26656);
            assert!(host == "35.226.255.25");
        }
        Ok(())
    }

    #[test]
    fn parse_cmt_addr_with_node_id() -> anyhow::Result<()> {
        let s = "tcp://00716b3dd31cf894cce3fc9d05f2f374e68222e8@34.135.6.235:2665";
        let _a: TendermintAddress = s.parse()?;
        Ok(())
    }
    #[test]
    fn parse_cmt_addr_from_listener() -> anyhow::Result<()> {
        // Most upstream Tendermint types are just String structs, so there's
        // no handling of the `Listener()` wrapper. We must regex it out.
        let l = tendermint::node::info::ListenAddress::new(
            "Listener(@35.226.255.25:26656)".to_string(),
        );
        let tm1 = TendermintAddress::from_listen_address(&l);
        assert!(tm1.is_none());
        Ok(())
    }
}
