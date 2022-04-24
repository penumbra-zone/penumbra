# Joining a Testnet

Currently we only provide instructions for running a fullnode deployment. A
fullnode will sync with the network but will not have any voting power, and will
not be eligible for staking or funding stream rewards. For more information on
what a fullnode is, see the [Tendermint
documentation](https://docs.tendermint.com/v0.35/nodes/#full-node).

## Configuring your node

Run the following:

```console
tendermint init full
```

and you will have the Tendermint files created necessary for your fullnode:

```console
\$ ls \$HOME/.tendermint
config	data

\$ ls \$HOME/.tendermint/config/
config.toml	genesis.json	node_key.json
```

The `genesis.json` file Tendermint generates needs to be replaced with the
genesis file for the network you want to join.  The genesis JSON files for all
of our testnets can be found [in our repository][testnets-repo], or downloaded
directly from an existing node using the Tendermint RPC:
```console
curl -s http://testnet.penumbra.zone:26657/genesis | jq ".result.genesis" > \$HOME/.tendermint/config/genesis.json
```

Next you'll need to modify the persistent peers list to specify our primary
testnet node as a permanent peer for your node. Open
`\$HOME/.tendermint/config/config.toml` and find the `persistent-peers` line and
add the testnet node:
```toml
persistent-peers = "NODE_ID@testnet.penumbra.zone:26656"
```
The format is `NODE_ID@ADDRESS`.  You will need to replace `NODE_ID` with the
identifier for the node you want to connect to.  On a production network, this
might be stable, but our testnet nodes generate keys on deployment.  To find the
current node ID, run:
```console
curl -s http://testnet.penumbra.zone:26657/status | jq ".result.node_info.id"
```
This command uses `jq` to extract the node ID from the response, but if you
don't have `jq` installed, you can just look for that entry in the returned
JSON.

## Starting your node

After configuration is complete, you're ready to start your node.

First, start the `pd` binary:

```console
export RUST_LOG="warn,pd=debug,penumbra=debug,jmt=info" && \ # or some other logging level
cargo run --release --bin pd start --rocks-path \$HOME/.rocksdb 
```

Then (perhaps in another terminal) start Tendermint:

```console
tendermint start
```

You should now be participating in the network!

[testnets-repo]: https://github.com/penumbra-zone/penumbra/tree/main/testnets
