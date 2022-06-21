# Creating a Testnet

## Creating a genesis file

Running a local testnet requires creating a `genesis.json` describing the initial
parameters of the network.  This has two parts:

1. Tendermint-related data specifying parameters for the consensus engine;
2. Penumbra-related data specifying the initial chain state.

First, [install Tendermint][tm-install].  Be sure to install `v0.35.6`, rather
than `master`.

Next, create the Tendermint config data with
```bash
tendermint init validator
```
This will create a default genesis file stored in `\$TMHOME/config` (if set, else
`~/.tendermint/config`) named `genesis.json`, as well as a validator private key
named `priv_validator_key.json`.

The Penumbra-related data specifying the initial chain state depends on your
local key material, since a testnet with no assets you can control is not
particularly useful, so it requires some manual editing.

You'll probably want to generate a new testing wallet with
```
cargo run --bin pcli -- -w testnet_wallet.json wallet generate
# Example, create whatever addresses you want for testing
cargo run --bin pcli -- -w testnet_wallet.json addr new "Test Address 1"
cargo run --bin pcli -- -w testnet_wallet.json addr new "Test Address 2"
```

Next, produce a template with
```
cargo run --bin pd -- generate-testnet
```
and copy the `app_state` field of one the genesis files. You'll need to
edit it to match the key material you'll be using, which includes:

* changing the validator public keys to match the one Tendermint generated;
* editing the genesis allocations to use your testing addresses, or have other asset types, etc.

You may wish to edit other parts of the testnet config. Example `genesis.json`
files can be found in the `testnets/` directory if you get stuck.

## Running `pd` without using Docker

You'll need to create a `genesis.json` file as described above.

There are three components to a Penumbra node: the Tendermint instance, the `pd`
instance, and the RocksDB instance.

Start the Penumbra instance (you probably want to set `RUST_LOG` to `debug`):
```bash
cargo run --bin pd start --home \$HOME/.pd
```
Start the Tendermint node:
```bash
tendermint start
```

You should be running!

To stop the node, shut down either `pd` or `tendermint`.

Resetting the state requires multiple steps:

* To reset the Tendermint state, use `tendermint unsafe-reset-all`.
* To reset your wallet state (without deleting keys), use `pcli wallet reset`.
* To reset pd state, delete the pd state directory: `\$HOME/.pd`

You need to do **all of these** to fully reset the node, and doing only one will
result in mysterious errors.

## Running `pd` with Docker

You'll need to create a `genesis.json` file as described above.  This command
will only work if you have loaded genesis state:

```bash
docker-compose up --build -d
```

To load genesis state for a fresh Docker configuration:

**NOTE:** this will **destroy** any existing data you have stored in the Docker volumes
for pd/rocksDB/tendermint!

```bash
./scripts/docker_compose_freshstart.sh
```

The script will handle generating genesis JSON data (but not editing it).

After running the script, `$HOME/.penumbra/testnet_data/` will contain the initial configuration and state of the tendermint nodes.

**You should go in and edit the genesis JSON for `node0` (we currently only run one tendermint
node in our testnet: `~/.penumbra/testnet_data/node0/config/genesis.json`)

After configuring the genesis JSON, you can start the testnet:

`docker-compose up --build -d`

You should have a working setup with all containers running
after running the script:

```console
\$ docker ps
CONTAINER ID   IMAGE                          COMMAND                  CREATED         STATUS         PORTS                                                                                    NAMES
b7fce1d0ffd9   tendermint/tendermint:latest   "docker-entrypoint.s…"   4 minutes ago   Up 4 minutes   0.0.0.0:6060->6060/tcp, 0.0.0.0:26656-26657->26656-26657/tcp, 0.0.0.0:27000->26660/tcp   tendermint
5a6bd39bb6f7   grafana/grafana:latest         "/run.sh"                4 minutes ago   Up 4 minutes   0.0.0.0:3000->3000/tcp                                                                   penumbra-grafana-1
b8f599963ebc   penumbra_pd                    "pd start --host 0.0…"   4 minutes ago   Up 4 minutes   0.0.0.0:26658->26658/tcp                                                                 penumbra
9e82aa33b4ff   prom/prometheus:latest         "/bin/prometheus --c…"   4 minutes ago   Up 4 minutes   0.0.0.0:9090->9090/tcp                                                                   penumbra-prometheus-1
```

On production, you should use the production Docker Compose configuration, which will use the
managed database as well as disable various debug build configs used in dev:

```console
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --build
```
