# Requirements for running a node

In order to run a Penumbra fullnode, you'll need a machine
with sufficient resources. See specifics below.

## System requirements

We recommend using a machine with at least:

* 8GB RAM
* 2-4 vCPUS
* ~200GB persistent storage (~20GB/week)

You can host your node on hardware, or on your cloud provider of choice.

## Network requirements

A Penumbra fullnode should have a publicly routable IP address
to accept P2P connections. It's possible to run a fullnode behind NAT,
but then it won't be able to receive connections from peers.
The relevant network endpoints for running Penumbra are:

* `26656/TCP` for CometBFT P2P, should be public
* `26657/TCP` for CometBFT RPC, should be private
* `26660/TCP` for CometBFT metrics, should be private
* `26658/TCP` for Penumbra ABCI, should be private
* `9000/TCP` for Penumbra metrics, should be private
* `8080/TCP` for Penumbra gRPC, should be private
* `443/TCP` for Penumbra HTTPS, optional, should be public if enabled

You can opt in to HTTPS support for Penumbra's gRPC service by setting
the `--grpc-auto-https <DOMAIN>` option. See `pd start --help` for more info.

## Custody considerations

Validators should review the [pcli key custody](../../pcli/wallet.md#validator-custody) recommendations
for protecting the validator identity.

## CometBFT settings

When bootstrapping a new network connection via [`pd network join`](join-network.md),
`pd` will create initial CometBFT settings for the node. Node operators
should review that configuration, stored at `~/.penumbra/network_data/node0/cometbft/config/config.toml`
by default, and adapt it to their needs.

In particular, node operators should ensure that the following values are set:

```toml
[mempool]
broadcast = true
keep-invalid-txs-in-cache = false
max_tx_bytes = 98304
max_txs_bytes = 10485760
recheck = true
size = 5000

[consensus]
timeout_propose = "3000ms"
timeout_propose_delta = "500ms"
timeout_prevote = "1000ms"
timeout_prevote_delta = "500ms"
timeout_precommit = "1000ms"
timeout_precommit_delta = "500ms"
timeout_commit = "5000ms"
create_empty_blocks = true
create_empty_blocks_interval = "0ms"
```

The `mempool` settings are consensus-critical, and should not be changed without coordination.

## Security limits

The OS defaults for maximum number of open file descriptors is typically `1024`, which is too low
for running a Penumbra node. The example systemd configs raise this value to `65536` via the `LimitNOFILE`
declaration. Node operators should set this value system-wide, by editing `/etc/security/limits.conf`.

## Deployment strategies

We expect node operators to manage the lifecycle of their Penumbra deployments.
Some example configs for systemd can be found in the Penumbra repo's
[`deployments/`](https://github.com/penumbra-zone/penumbra/tree/{{ #include ../../penumbra_version.md }}/deployments) directory.
Other community solutions include:

* [Cosmos Operator] by [Strangelove] for Kubernetes
* [NixOS derivations](https://github.com/starlingcyber/infra) maintained by [Starling Cybernetics]

You should consult these configurations as a reference, and write your own
scripts to maintain your node.

Consider [joining the Penumbra Discord](../../resources.md#discord) to receive announcements
about new versions and required actions by node operators.

[Cosmos Operator]: https://github.com/strangelove-ventures/cosmos-operator/
[Strangelove]: https://strange.love/
[Starling Cybernetics]: https://starlingcyber.net/
