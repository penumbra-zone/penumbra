
# Fullnode Deployment

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
The response data is nested, so we use `jq` to grab a specific key.

Next, grab the correct tendermint configuration from our repo. This is important to make sure your node can stay in sync with the network:

```console
curl -s https://raw.githubusercontent.com/penumbra-zone/penumbra/main/testnets/tm_config_template.toml > \$HOME/.tendermint/config/config.toml
```

Fill in any missing fields in the template, checking for them like so:

```console
grep "{}" \$HOME/.tendermint/config/config.toml
```

Next you'll need to modify the bootstrap peers list to specify our primary
testnet node as the initial peer for your node. Open
`\$HOME/.tendermint/config/config.toml` and find the `bootstrap-peers` line and
add the testnet node:
```toml
bootstrap-peers = "NODE_ID@testnet.penumbra.zone:26656"
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

You should now be syncing from the network as a fullnode!

If you see an error like the following:

```
fromProto: validatorSet proposer error: nil validator" closerError=""
```

when you run `tendermint start`, try resetting your tendermint state and trying again:

```console
rm -rf ~/.rocksdb
tendermint unsafe-reset-all
tendermint start
```
