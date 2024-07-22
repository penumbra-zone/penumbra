# Hermes and Penumbra

## Using a compatible Hermes version
Penumbra-compatibility exists in a fork of the Hermes software, available at:
[https://github.com/penumbra-zone/hermes](https://github.com/penumbra-zone/hermes).

```shell
git clone https://github.com/penumbra-zone/hermes
cd hermes
cargo build --release
cp -v ./target/release/hermes /usr/local/bin/hermes
```

Use the latest commit in that repo.
Eventually the necessary changes will be upstreamed to the parent repo.
Until that happens, use the forked version.

## Prerequisites

In order to run a Hermes instance for Penumbra, you'll need to prepare the following:

* The chain ID of the Penumbra network
* The chain ID of the counterparty network
* A funded Penumbra wallet, to pay fees on Penumbra (the host chain)
* A funded counterparty wallet, to pay fees on the counterparty chain
* Two (2) API endpoints for Penumbra node, `pd` gRPC and CometBFT JSON-RPC
* Two (2) API endpoints for counterparty node, app gRPC and CometBFT JSON-RPC
* A compatible version of `hermes`, built as described above.

Crucially, the wallets should be unique, dedicated solely to this instance of Hermes,
and not used by any other clients. When you have the above information, you're ready to proceed.

## Configuring Hermes

For the most part, you can follow the [official Hermes docs on configuration](https://hermes.informal.systems/documentation/configuration/configure-hermes.html).
There are two Penumbra-specific exceptions: 1) key support; and 2) on-disk view database support.

### Penumbra spend keys
The Penumbra integration does Hermes does not support the [`hermes keys add`](https://hermes.informal.systems/documentation/commands/keys/index.html)
flow for Penumbra chains. Instead, you should add the Penumbra wallet spendkey directly to the generated `config.toml` file, like so:

```toml
# Replace "XXXXXXXX" with the spend key for the Penumbra wallet.
kms_config = { spend_key = "XXXXXXXX" }
```

To find the wallet's spend key, you can view `~/.local/share/pcli/config.toml`. 

### Penumbra view database
Then, to configure on-disk persistence of the Penumbra view database, add this line to your config:

```toml
# Update the path below as appropriate for your system,
# and make sure to create the directory before starting Hermes.
view_service_storage_dir = "/home/hermes/.local/share/pcli-hermes-1"
```

Consider naming the directory `pcli-hermes-<counterparty>`, where counterparty is the name of the counterparty chain.
If you do not set this option, `hermes` will still work, but it will need to resync with the chain on startup,
which can take a long time, depending on how many blocks exist.

## Path setup

Again, see the [official Hermes docs on path setup](https://hermes.informal.systems/documentation/commands/path-setup/index.html).
In order to validate that the channels are visible on host chain, use `pcli query ibc channels` and confirm they match
what was output from the `hermes create` commands.

## Best practices

Consult the official Hermes docs for [running in production](https://hermes.informal.systems/tutorials/production/index.html),
as well as the [telemetry guide](https://hermes.informal.systems/documentation/telemetry/index.html).
You'll need to communicate the channels that you maintain to the community. How you do so is up to you.

[hermes]: https://hermes.informal.systems
