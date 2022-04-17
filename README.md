![Penumbra logo](docs/images/penumbra-dark.svg#gh-dark-mode-only)
![Penumbra logo](docs/images/penumbra-light-bw.svg#gh-light-mode-only)

[Penumbra] is a fully shielded zone for the Cosmos ecosystem, allowing anyone to securely transact,
stake, swap, or marketmake without broadcasting their personal information to the world.

## Getting involved

The primary communication hub is our [Discord]; click the link to join the
discussion there.

The (evolving) protocol spec is rendered at [protocol.penumbra.zone][protocol].

The (evolving) API documentation is rendered at [rustdoc.penumbra.zone][rustdoc].

To participate in our test network, [keep reading below](#getting-started-on-the-test-network).

For instructions on how to set up a node, [jump down and read on](#running-a-penumbra-node).

## Getting started on the test network

[Penumbra Labs][Penumbra] runs a test network for the latest version of the Penumbra protocol, and
we would be delighted for you to try it out! [Per our development plan, we'll be launching (and
crashing) lots of testnets](https://penumbra.zone/blog/how-were-building-penumbra), and users should
expect data loss at this early stage. Keep in mind that the tokens on the test network have no
monetary value whatsoever, and we'll be shutting down the testnet when we reach our next milestone
(or before). If (which is to say, _when_) you encounter bugs or wish for features to exist, we'd
love for you to reach out to us on our [Discord server][Discord].

At the moment the testnet is a single node, and running testnet nodes won't be possible until our
next milestone (MVP2), implementing [staking and
delegation](https://penumbra.zone/technology/stake).

### Building `pcli`

To get started with the Penumbra test network, you will first need to download and build the
Penumbra command line light wallet, `pcli`.

#### Installing the Rust toolchain

This requires that you install a recent stable version
of the Rust compiler, installation instructions for which you can find
[here](https://www.rust-lang.org/learn/get-started). Don't forget to reload your shell so that
`cargo` is available in your `$PATH`!

#### Installing build prerequisites

**On Linux:** you may need to install some additional packages in order to build `pcli`,
depending on your distribution. For a bare-bones Ubuntu installation, you can run:

```bash
sudo apt-get install build-essential pkg-config libssl-dev
```

**On macOS:** you may need to install the command-line developer tools if you have never done so:

```bash
xcode-select --install
```

#### Cloning the repository

Once you have installed the above tools, you can clone the repository:

```bash
git clone https://github.com/penumbra-zone/penumbra
```

To build the version of `pcli` compatible with the current testnet, check out the latest tag for
the current test net:

```bash
cd penumbra && git checkout 007-herse && cargo update
```

#### Building the `pcli` wallet software

Then, build the `pcli` tool using `cargo`:

```bash
cargo build --release --bin pcli
```

Because you are building a work-in-progress version of the client, you may see compilation warnings,
which you can safely ignore.

### Generating your wallet

**Hint:** When working with `pcli`, the level of diagnostic information printed is dependent on the `RUST_LOG`
environment variable. To see progress updates and other logged information while running `pcli`, we
recommend you set `export RUST_LOG=info` in your terminal.

On first installation of `pcli`, you will need to generate a fresh wallet to use with Penumbra. You
should see something like this:

```bash
$ cargo run --quiet --release --bin pcli wallet generate
Saving wallet to /home/$USER/.local/share/pcli/penumbra_wallet.json
Saving backup wallet to /home/$USER/.local/share/penumbra-testnet-archive/penumbra-euporie/.../penumbra_wallet.json
```

Penumbra's design allows you to create arbitrarily many publicly unlinkable addresses which all
correspond to your own wallet. When you first created your wallet above, `pcli` created your first
address, labeled `Default`. When you list your addresses, you should see something like this:

```bash
$ cargo run --quiet --release --bin pcli addr list
 Index  Label    Address
 0      Default  penumbrav0t1...
```

### Getting testnet tokens on the [Discord] in the `#testnet-faucet` channel

In order to use the testnet, it's first necessary for you to get some testnet tokens. The current
way to do this is to join our [Discord] and post your address in the `#testnet-faucet` channel.
We'll send your address some tokens on the testnet for you to send to your friends! :)

Just keep in mind: **testnet tokens do not have monetary value**, and in order to keep the
signal-to-noise ratio high on the server, requests for tokens in other channels will be deleted
without response. Please do not DM Penumbra Labs employees asking for testnet tokens; the correct
venue is the dedicated channel.

### Synchronizing your wallet

Once you've received your first tokens, you can scan the chain to import them into your local
wallet (this may take a few minutes the first time you run it):

```bash
cargo run --quiet --release --bin pcli sync
```

If someone sent you testnet assets, you should be able to see them now by running:

```bash
cargo run --quiet --release --bin pcli balance
```

This will print a table of assets by balance in each.

### Sending transactions

Now, for the fun part: sending transactions. If you have someone else's testnet address, you can
send them any amount of any asset you have. For example, if I wanted to send 10 penumbra tokens
to my friend, I could do that like this (filling in their full address at the end):

```bash
cargo run --quiet --release --bin pcli tx send 10penumbra --to penumbrav1t...
```

Notice that asset amounts are typed amounts, specified without a space between the amount (`10`)
and the asset name (`penumbra`).

If you have the asset in your wallet to send, then so it shall be done!

### Please submit any feedback and bug reports

Thank you for helping us test the Penumbra network! If you have any feedback, please let us know in
the `#testnet-feedback` channel on our [Discord]. We would love to know about bugs, crashes,
confusing error messages, or any of the many other things that inevitably won't quite work yet. Have
fun! :)

## Connecting a node to a testnet

A fullnode can be joined to our testnet deployments and optionally behave as a validator.

The node binary (`pd`) software is part of the same repository as `pcli`, so ensure that you've [cloned the repository](https://github.com/penumbra-zone/penumbra#cloning-the-repository).

### Building the `pd` node binary

Then, build the `pd` binary using `cargo`:

```bash
cargo build --release --bin pd
```

Because you are building a work-in-progress version of the node, you may see compilation warnings,
which you can safely ignore.

### Installing tendermint

You'll need to have [tendermint installed](https://docs.tendermint.com/master/introduction/install.html) on your system to join your node to the testnet.


### Configuring your node

Currently we only provide instructions for running a fullnode deployment. A fullnode will sync with the network but will not have any voting power,
and will not be eligible for staking or funding stream rewards. For more information on what a fullnode is, see the [Tendermint documentation](https://docs.tendermint.com/v0.35/nodes/#full-node).

Run the following:

```console
$ tendermint init full
```

and you will have the Tendermint files created necessary for your fullnode:

```console
$ ls $HOME/.tendermint
config	data

$ ls $HOME/.tendermint/config/
config.toml	genesis.json	node_key.json
```

Next you'll need to modify the persistent peers list to specify our primary testnet node as a permanent peer for your node. Open `$HOME/.tendermint/config/config.toml` and find the `persistent-peers` line and make it look like so:

```toml
persistent-peers = "TODO_IDENTIFIER_GOES_HERE@192.167.10.21:26656"
```

Then you'll need to download our genesis JSON file and replace the automatically generated one.
You can find the genesis JSON files for all of our testnets [in our repository](https://github.com/penumbra-zone/penumbra/tree/main/testnets).

The testnet directories are numbered in increasing order, find and download the latest JSON file and copy it to `$HOME/.tendermint/config/genesis.json`.

### Starting your node

After configuration is complete, you're ready to start your node.

First, start the `pd` binary:

```console
$ cargo run --release --bin pd start --rocks-path $HOME/.rocksdb &
$ ps | grep pd
52949 ttys001    0:00.32 target/release/pd start --rocks-path $HOME/.rocksdb
```

Then start tendermint:

```console
$ tendermint start
```

You should now be participating in the network!

## Building the protocol spec

The [protocol spec][protocol] is built using [mdBook] and auto-deployed on
pushes to `main`.  To build it locally:

1. Install the requirements: `cargo install mdbook mdbook-katex mdbook-mermaid`
2. To continuously build and serve the documentation: run `mdbook serve` from `docs/protocol`.

## Development

This section is for developers, not for running nodes that are part of the
public testnet. We won't be ready for multiple testnet nodes until we reach our
MVP2 milestone implementing staking and delegation, at which point this will
change.

Penumbra has two binaries, the command-line light wallet interface `pcli` and
the daemon `pd`.  However, the daemon cannot be run alone; it requires both
Tendermint (to handle network communication and drive the consensus state) and
RocksDB (to act as a data store).

### SQLite compilation setup

The wallet server uses SQLite via `sqlx` as its backing store.  The type-safe
query macros require compile-time information about the database schemas.
Normally, this information is cached in the crate's `sqlx-data.json`, and
nothing extra is required to build.

However, when editing the wallet server's database code, it's necessary to work
with a development database:

1.  The database structure is defined in the `migrations/` directory of the
wallet server crate.
2.  Set the `DATABASE_URL` environment variable to point to the SQLite location.
For instance,
```
export DATABASE_URL="sqlite:///tmp/pwalletd-dev-db.sqlite"
```
will set the shell environment variable to the same one set in the project's
`.vscode/settings.json`.
3.  From the `wallet-next` directory, run `cargo sqlx database setup` to create
the database and run migrations.
4.  From the `wallet-next` directory, run
`cargo sqlx prepare -- --lib=penumbra-wallet-next`
to regenerate the `sqlx-data.json` file that allows offline compilation.

### Creating a genesis file

Running a local testnet requires creating a `genesis.json` describing the initial
parameters of the network.  This has two parts:

1. Tendermint-related data specifying parameters for the consensus engine;
2. Penumbra-related data specifying the initial chain state.

First, [install Tendermint][tm-install].  Be sure to install `v0.35.0`, rather
than `master`.

Next, create the Tendermint config data with
```bash
tendermint init validator
```
This will create a default genesis file stored in `$TMHOME/config` (if set, else
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

### Running `pd` without using Docker

You'll need to create a `genesis.json` file as described above.

There are three components to a Penumbra node: the Tendermint instance, the `pd`
instance, and the RocksDB instance.

Start the Penumbra instance (you probably want to set `RUST_LOG` to `debug`):
```bash
cargo run --bin pd start --rocks-path $HOME/.rocksdb
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
* To reset RocksDB state, delete the RocksDB state file: `$HOME/.rocksdb`

You need to do **all of these** to fully reset the node, and doing only one will
result in mysterious errors.

### Running `pd` with Docker

You'll need to create a `genesis.json` file as described above.  This command
will only work if you have loaded genesis state:

```bash
docker-compose up --build -d
```

To load genesis state for a fresh Docker configuration:

**NOTE:** this will **destroy** any existing data you have stored in the Docker volumes
for pd/rocksDB/tendermint!

```bash
./scripts/docker_compose_freshstart.sh ~/scratch/testnet_build
# the ~/scratch/testnet_build directory should be the root of the volume mounted
# to the tendermint node containers in docker-compose.yml
```

The script will handle generating genesis JSON data (but not editing it).

After running the script, the data directory provided to the script will contain the initial configuration and state of the tendermint nodes.

**You should go in and edit the genesis JSON for `node0` (we currently only run one tendermint
node in our testnet: `~/scratch/testnet_build/node0/config/genesis.json`)

After configuring the genesis JSON, you can start the testnet:

`docker-compose up --build -d`

You should have a working setup with all containers running
after running the script:

```console
$ docker ps
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


### Metrics

When adding new metrics, please following the [Prometheus metrics naming
guidelines](https://prometheus.io/docs/practices/naming/). Use plurals for consistency. For the
application prefix part of the name, use `node` for the Penumbra node.

[Discord]: https://discord.gg/hKvkrqa3zC
[Penumbra]: https://penumbra.zone
[protocol]: https://protocol.penumbra.zone
[mdBook]: https://github.com/rust-lang/mdBook
[rustdoc]: https://rustdoc.penumbra.zone
[tm-install]: https://github.com/tendermint/tendermint/blob/master/docs/introduction/install.md#from-source
