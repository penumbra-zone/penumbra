//! Logic for onboarding a new `pd` node onto an existing network.
//! Handles generation of config files for `pd` and `cometbft`.
use anyhow::Context;
use rand::seq::SliceRandom;
use rand_core::OsRng;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use tendermint_config::net::Address as TendermintAddress;
use url::Url;

use flate2::read::GzDecoder;
use std::io::Write;
use tokio_stream::StreamExt;

use crate::network::config::{parse_tm_address, NetworkTendermintConfig};
use crate::network::generate::NetworkValidator;

/// Bootstrap a connection to a network, via a node on that network.
/// Look up network peer info from the target node, and seed the tendermint
/// p2p settings with that peer info.
pub async fn network_join(
    output_dir: PathBuf,
    node: Url,
    node_name: &str,
    external_address: Option<TendermintAddress>,
    tm_rpc_bind: SocketAddr,
    tm_p2p_bind: SocketAddr,
) -> anyhow::Result<()> {
    let mut node_dir = output_dir;
    node_dir.push("node0");
    let genesis_url = node.join("genesis")?;
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
    let genesis = serde_json::value::from_value(genesis_json)?;
    tracing::info!("fetched genesis");

    // Look up more peers from the target node, so that generated tendermint config
    // contains multiple addresses, making peering easier.
    let mut peers = Vec::new();
    if let Some(node_tm_address) = fetch_listen_address(&node).await {
        peers.push(node_tm_address);
    } else {
        // We consider it odd that a bootstrap node has a remotely accessible
        // RPC endpoint, but no P2P listener enabled.
        tracing::warn!("Failed to find listenaddr for {}", &node);
    }
    let new_peers = fetch_peers(&node).await?;
    peers.extend(new_peers);
    tracing::info!(?peers, "Network peers for inclusion in generated configs");

    let tm_config = NetworkTendermintConfig::new(
        node_name,
        peers,
        external_address,
        Some(tm_rpc_bind),
        Some(tm_p2p_bind),
    )?;

    let tv = NetworkValidator::default();
    tm_config.write_config(node_dir, &tv, &genesis)?;
    Ok(())
}

/// Query the Tendermint node's RPC endpoint at `tm_url` and return the listener
/// address for the P2P endpoint. Returns an [Option<TendermintAddress>] because
/// it's possible that no p2p listener is configured.
pub async fn fetch_listen_address(tm_url: &Url) -> Option<TendermintAddress> {
    let client = reqwest::Client::new();
    // We cannot assume the RPC URL is the same as the P2P address,
    // so we use the RPC URL to look up the P2P listening address.
    let listen_info = client
        .get(tm_url.join("net_info").ok()?)
        .send()
        .await
        .ok()?
        .json::<serde_json::Value>()
        .await
        .ok()?
        .get_mut("result")
        .and_then(|v| v.get_mut("listeners"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    tracing::debug!(?listen_info, "found listener info from bootstrap node");
    let first_entry = listen_info[0].as_str().unwrap_or_default();
    let listen_addr = parse_tm_address_listener(first_entry)?;

    // Next we'll look up the node_id, so we can assemble a self-authenticating
    // Tendermint Address, in the form of <id>@<url>.
    let node_id = client
        .get(tm_url.join("status").ok()?)
        .send()
        .await
        .ok()?
        .json::<serde_json::Value>()
        .await
        .ok()?
        .get_mut("result")
        .and_then(|v| v.get_mut("node_info"))
        .and_then(|v| v.get_mut("id"))
        .ok_or_else(|| anyhow::anyhow!("could not parse JSON from response"))
        .ok()?
        .take();

    let node_id: tendermint::node::Id = serde_json::value::from_value(node_id).ok()?;
    tracing::debug!(?node_id, "fetched node id");

    let listen_addr_url = Url::parse(&format!("{}", &listen_addr)).ok()?;
    tracing::info!(
        %listen_addr_url,
        "fetched listener address for bootstrap node"
    );
    parse_tm_address(Some(&node_id), &listen_addr_url).ok()
}

/// Query the Tendermint node's RPC endpoint at `tm_url` and return a list
/// of all known peers by their `external_address`es. Omits private/special
/// addresses like `localhost` or `0.0.0.0`.
pub async fn fetch_peers(tm_url: &Url) -> anyhow::Result<Vec<TendermintAddress>> {
    let client = reqwest::Client::new();
    let net_info_url = tm_url.join("net_info")?;
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
    let mut peers = Vec::new();
    let mut seeds = Vec::new();

    for raw_peer in net_info_peers {
        tracing::debug!(?raw_peer, "Analyzing whether to include candidate peer");
        let node_id: tendermint::node::Id = raw_peer
            .get("node_info")
            .and_then(|v| v.get("id"))
            .and_then(|v| serde_json::value::from_value(v.clone()).ok())
            .ok_or_else(|| anyhow::anyhow!("Could not parse node_info.id from JSON response"))?;

        let listen_addr = raw_peer
            .get("node_info")
            .and_then(|v| v.get("listen_addr"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!("Could not parse node_info.listen_addr from JSON response")
            })?
            // Depending on node config, there may or may not be a protocol prefix.
            // Remove it so we can treat it as a SocketAddr when checking for internal/external.
            .replace("tcp://", "");

        let moniker = raw_peer
            .get("node_info")
            .and_then(|v| v.get("moniker"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!("Could not parse node_info.moniker from JSON response")
            })?;

        // Filter out addresses that are obviously not external addresses.
        if !address_could_be_external(&listen_addr) {
            tracing::debug!(
                ?listen_addr,
                "Skipping candidate peer due to internal listener address"
            );
            continue;
        }
        // The API returns a str formatted as a SocketAddr; prepend protocol so we can handle
        // as a URL. The Tendermint config template already includes the tcp:// prefix.
        let laddr = format!("tcp://{}", listen_addr);
        let listen_url = Url::parse(&laddr).context(format!(
            "Failed to parse candidate tendermint addr as URL: {}",
            listen_addr
        ))?;
        let peer_tm_address = parse_tm_address(Some(&node_id), &listen_url)?;
        // A bit of optimism: any node with "seed" in its moniker gets to be a seed.
        if moniker.contains("seed") {
            tracing::debug!(
                ?peer_tm_address,
                moniker,
                "Found self-described seed node in candidate peers"
            );
            seeds.push(peer_tm_address)
        // Otherwise, we check if we've found enough.
        } else if peers.len() <= threshold {
            peers.push(peer_tm_address)
        }
    }
    if peers.len() < threshold && seeds.is_empty() {
        tracing::warn!(
            "bootstrap node reported only {} peers, and 0 seeds; we may have trouble peering",
            peers.len(),
        );
    }
    // TODO handle seeds and peers differently. For now, all peers are used as seeds.
    seeds.extend(peers);
    Ok(seeds)
}

/// Download a gzipped tarball from a URL, and extract its contents as the starting state
/// config for the fullnode. Allows bootstrapping from archived state, which is useful
/// for nodes joining after a chain upgrade has been performed.
///
/// Supports archive files generated via `pd export`, which contain only the rocksdb dir,
/// and via `pd migrate`, which contain the rocksdb dir, new genesis content, and a private
/// validator state file.
///
/// The `output_dir` should be the same argument as passed to `pd network --network-dir <dir> join`;
/// relative paths for pd and cometbft will be created from this base path.
///
/// The `leave_archive` argument allows you to keep the downloaded archive file after unpacking.
pub async fn unpack_state_archive(
    archive_url: Url,
    output_dir: PathBuf,
    leave_archive: bool,
) -> anyhow::Result<()> {
    let archive_filepath: std::path::PathBuf;
    // Check whether URL points to a local file
    if archive_url.scheme() == "file" {
        tracing::info!(%archive_url, "extracting compressed node state from local file");
        archive_filepath = archive_url.to_file_path().map_err(|e| {
            tracing::error!(?e);
            anyhow::anyhow!("failed to convert archive url to filepath")
        })?;
    } else {
        // Download.
        // Here we inspect HEAD so we can infer filename.
        tracing::info!(%archive_url, "downloading compressed node state");
        let response = reqwest::get(archive_url).await?;
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("pd-node-state-archive.tar.gz");

        archive_filepath = output_dir.join(fname);
        let mut download_opts = std::fs::OpenOptions::new();
        download_opts.create_new(true).write(true);
        let mut archive_file = download_opts.open(&archive_filepath)?;

        // Download via stream, in case file is too large to shove into RAM.
        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            archive_file.write_all(&chunk)?;
        }
        archive_file.flush()?;
        tracing::info!("download complete: {}", archive_filepath.display());
    }

    // Extract.
    // Re-open downloaded file for unpacking, for a fresh filehandle.
    let mut unpack_opts = std::fs::OpenOptions::new();
    unpack_opts.read(true);
    let f = unpack_opts
        .open(&archive_filepath)
        .context("failed to open local archive for extraction")?;
    let tar = GzDecoder::new(f);
    let mut archive = tar::Archive::new(tar);
    // This dir-path building is duplicated in the config gen code.
    let pd_home = output_dir.join("node0").join("pd");
    archive
        .unpack(&pd_home)
        .context("failed to extract tar.gz archive")?;

    // If the archive we consumed was generated via `pd migrate`, then it will contain
    // a new genesis file and priv_validator_state.json, both of which should be applied
    // over the generated cometbft config files. If the archive was generated via `pd export`,
    // then those extra files will be missing, and only rocksdb data will be present.
    let new_genesis = pd_home.join("genesis.json");
    let new_val_state = pd_home.join("priv_validator_state.json");
    let cometbft_dir = output_dir.join("node0").join("cometbft");
    let copy_opts = fs_extra::dir::CopyOptions::new().overwrite(true);

    if new_genesis.exists() {
        tracing::info!(new_genesis = %new_genesis.display(), "copying new genesis content from archive");
        let f = vec![new_genesis];
        fs_extra::move_items(&f, cometbft_dir.join("config"), &copy_opts)?;
    }
    if new_val_state.exists() {
        tracing::info!(new_val_state = %new_val_state.display(), "copying new priv_validator_state.json content from archive");
        let f = vec![new_val_state];
        fs_extra::move_items(&f, cometbft_dir.join("data"), &copy_opts)?;
    }

    tracing::info!("archived node state unpacked to {}", pd_home.display());

    if !leave_archive {
        // Post-extraction, clean up the downloaded tarball.
        std::fs::remove_file(archive_filepath)?;
    } else {
        tracing::info!(path = ?archive_filepath, "leaving downloaded archive on disk");
    }

    Ok(())
}

/// Check whether SocketAddress spec is likely to be externally-accessible.
/// Filters out RFC1918 and loopback addresses. Requires an address and port.
// TODO: This should return a Result, to be clearer about the expectation
// of a SocketAddr, rather than an IpAddr, as arg.
fn address_could_be_external(address: &str) -> bool {
    let addr = address.parse::<SocketAddr>().ok();
    match addr {
        Some(a) => match a.ip() {
            IpAddr::V4(ip) => !(ip.is_private() || ip.is_loopback() || ip.is_unspecified()),
            IpAddr::V6(ip) => !(ip.is_loopback() || ip.is_unspecified()),
        },
        _ => false,
    }
}

/// Extract a [TendermintAddress] obtained from the RPC `/net_info` endpoint.
/// The raw value is a String formatted as:
///
///   * `Listener(@35.226.255.25:26656)` or
///   * `Listener(@tcp://35.226.255.25:26656)` or
///   * `Listener(@)`
///
/// It may be possible for a node [Id] to proceed the `@`.
pub fn parse_tm_address_listener(s: &str) -> Option<TendermintAddress> {
    let re = regex::Regex::new(r"Listener\(.*@(tcp://)?(.*)\)").ok()?;
    let groups = re
        .captures(s)
        .expect("tendermint listener address from net_info endpoint is valid");
    let r: Option<String> = groups.get(2).map(|m| m.as_str().to_string());
    match r {
        Some(t) => {
            // Haven't observed a local addr in Listener field, but let's make sure
            // it's a public addr
            if address_could_be_external(&t) {
                t.parse::<TendermintAddress>().ok()
            } else {
                None
            }
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // The '35.226.255.25' IPv4 address used throughout these tests is a reserved
    // GCP IP, used by Penumbra Labs on the testnet cluster.
    #[test]
    fn external_address_detection() {
        assert!(!address_could_be_external("127.0.0.1"));
        assert!(!address_could_be_external("0.0.0.0"));
        assert!(!address_could_be_external("0.0.0.0:80"));
        assert!(!address_could_be_external("192.168.4.1:26657"));
        assert!(!address_could_be_external("35.226.255.25"));
        assert!(address_could_be_external("35.226.255.25:26657"));
    }

    // Some of these tests duplicate tests in the upstream Tendermint crates.
    // The underlying structs upstream are mostly just Strings, though, and since
    // our code wrangles these types for interpolation in config files from multiple
    // API endpoint sources, it's important to validate expected behavior.
    #[test]
    fn parse_tendermint_address_tcp() -> anyhow::Result<()> {
        let tm1 = parse_tm_address(None, &Url::parse("tcp://35.226.255.25:26656")?)?;
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
    // The Tendermint RPC net_info endpoint will return Listener information
    // formatted as:
    //
    //   * `Listener(@35.226.255.25:26656)` or
    //   * `Listener(@tcp://35.226.255.25:26656)` or
    //   * `Listener(@)`
    //
    // I've yet to observe a node_id preceding the `@`.
    fn parse_tendermint_address_listener() -> anyhow::Result<()> {
        let l1 = "Listener(@35.226.255.25:26656)";
        let r1 = parse_tm_address_listener(l1);
        assert!(r1 == Some("35.226.255.25:26656".parse::<TendermintAddress>()?));

        let l2 = "Listener(tcp://@35.226.255.25:26656)";
        let r2 = parse_tm_address_listener(l2);
        assert!(r2 == Some("tcp://35.226.255.25:26656".parse::<TendermintAddress>()?));

        let l3 = "Listener(@)";
        let r3 = parse_tm_address_listener(l3);
        assert!(r3.is_none());

        Ok(())
    }
    #[test]
    fn parse_tendermint_address_from_listener() -> anyhow::Result<()> {
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
