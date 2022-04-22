# Validator Deployment

## Configuring your node

Run the following:

```console
tendermint init full
```

and you will have the Tendermint files created necessary for your validator:

```console
\$ ls \$HOME/.tendermint
config	data

\$ ls \$HOME/.tendermint/config/
config.toml	genesis.json	node_key.json
```

Next you'll need to modify the persistent peers list to specify our primary
testnet node as a permanent peer for your node. Open
`\$HOME/.tendermint/config/config.toml` and find the `persistent-peers` line and
add the testnet node:

```toml
persistent-peers = "20eb3596354699d5b1952311f7cb4e133ad0b6c1@testnet.penumbra.zone:26656"
```

The format is `NODE_ID@ADDRESS`.  The node ID of the primary testnet node may
change, but the current node ID can be found with:
```console
curl http://testnet.penumbra.zone:26657/status | jq ".result.node_info.id"
```
If you don't have `jq` installed, you can just look for that entry in the returned JSON.

For a validator, you'll also **need to make sure that tendermint is set to validator mode** in the `config.toml`:

```toml
mode = "validator"
```

Then you'll need to download our genesis JSON file and replace the automatically
generated one.  You can find the genesis JSON files for all of our testnets [in
our repository](https://github.com/penumbra-zone/penumbra/tree/main/testnets).

The testnet directories are numbered in increasing order, find and download the
latest JSON file and copy it to `\$HOME/.tendermint/config/genesis.json`.

You can also download the genesis JSON directly from the testnet server, though it is nested
so you need to grab a specific key:

```console
curl -X GET "http://testnet.penumbra.zone:26657/genesis" -H "accept: application/json" | jq '.result.genesis' > \$HOME/.tendermint/config/genesis.json
```

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

You should now be participating in the network as a fullnode. However your validator won't be visible
to the chain yet, as the definition hasn't been uploaded.

## Uploading your validator definition

A validator definition contains fields defining metadata regarding your validator as well as initial funding streams.

Create a `validator.json` file like so:

```console
\$ cargo run --release --bin pcli -- validator template-definition --file validator.json
\$ cat validator.json
{
  "identity_key": "penumbravalid1g2huds8klwypzczfgx67j7zp6ntq2m5fxmctkf7ja96zn49d6s9qz72hu3",
  "consensus_key": "Fodjg0m1kF/6uzcAZpRcLJswGf3EeNShLP2A+UCz8lw=",
  "name": "",
  "website": "",
  "description": "",
  "funding_streams": [
    {
      "address": "penumbrav1t1mw8270qtpgjy628fg97p2px45e860jtlw0nl3w5y7vq67qx697py9t8ppp3mhwfxv8kegg8wuny64nf60z966krx85cqznjpshqtngffpwnywtzqjklkg3qh7anxk368ywac9l",
      "rate_bps": 100
    }
  ],
  "sequence_number": 0
}
```

and adjust the configurations as desired.

By default `template-definition` will use a random consensus key that you won't have access to. Make sure to **set the `consensus_key` correctly** otherwise your instance of `tendermint` won't be using the key expected in the validator definition. You can get the correct value for `consensus_key` from your `tendermint` configs:

```console
\$ grep -A3 pub_key ~/.tendermint/config/priv_validator_key.json
  "pub_key": {
    "type": "tendermint/PubKeyEd25519",
    "value": "Fodjg0m1kF/6uzcAZpRcLJswGf3EeNShLP2A+UCz8lw="
  },
```

The `identity_key` value placed by the `template-definition` command in your `validator.json` will correspond to the key used by your wallet configs for `pcli`, i.e. `pcli` will have access to any balance delegated to the validator.

After setting up metadata, funding streams, and the correct consensus key in your `validator.json`, you can upload it to the chain:

```console
cargo run --release --bin pcli -- validator upload-definition --file validator.json
```

And verify that it's known to the chain:

```console
cargo run --release --bin pcli -- stake list-validators -i
```

However your validator doesn't have anything delegated to it and will remain in an `Inactive` state until it receives enough delegations
to place it in the active set of validators.

## Delegating to your validator

First find your validator's identity key:

```console
cargo run --release --bin pcli -- stake list-validators -i
```

And then delegate some amount of `penumbra` to it:

```console
cargo run --release --bin pcli -- stake delegate 1penumbra --to penumbravalid1g2huds8klwypzczfgx67j7zp6ntq2m5fxmctkf7ja96zn49d6s9qz72hu3
```

You should then see your balance of `penumbra` decreased and that you have received some amount of delegation tokens for your validator:

```console
cargo run --release --bin pcli -- balance
```

Voting power will be calculated on the next epoch transition after your delegation takes place.
By default, epoch transitions occur every 40 blocks. Assuming that your delegation was enough to
place your validator in the top 10 validators by voting power, it should appear in the validator list as `Active` after the next epoch transition.

## Updating your validator

First fetch your existing validator definition from the chain:

```console
cargo run --release --bin pcli -- -n testnet-preview.penumbra.zone validator fetch-definition penumbravalid1dj3mgmje7z9mwu6rl9rplue2neqlc4zwls9a7w88vscwv88ltyqs8v6g9x --file validator.json
```

Then make any changes desired and **make sure to increase by `sequence_number` by at least 1!**
The `sequence_number` is a unique, increasing identifier for the version of the validator definition.

After updating the validator definition you can upload it again to update your validator metadata on-chain:

```console
cargo run --release --bin pcli -- validator upload-definition --file validator.json
```