![Penumbra logo](images/penumbra-dark.png#gh-dark-mode-only)
![Penumbra logo](images/penumbra-light.png#gh-light-mode-only)

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
monetary value whatsoever. If (which is to say, _when_) you encounter bugs or wish for features to
exist, we'd love for you to reach out to us on our [Discord server][Discord].

### Building `pcli`

To get started with the Penumbra test network, you will first need to download and build the
Penumbra command line light wallet, `pcli`. This requires that you install a recent stable version
of the Rust compiler, installation instructions for which you can find
[here](https://www.rust-lang.org/learn/get-started).

Once you have installed Rust and its build tool `cargo`, you can clone the repository:

```bash
git clone https://github.com/penumbra-zone/penumbra
```

Then, build the `pcli` tool using `cargo`:

```bash
cargo build --release --bin pcli
```

### Generating your wallet

When working with `pcli`, the level of diagnostic information printed is dependent on the `RUST_LOG`
environment variable. To see progress updates and other logged information while running `pcli`, we
recommend you set `export RUST_LOG=info` in your terminal.

On first installation of `pcli`, you will need to generate a fresh wallet to use with Penumbra. You
should see something like this:

```bash
$ cargo run --quiet --release --bin pcli wallet generate
Saving wallet to /home/$USER/.local/share/pcli/penumbra_wallet.json
Saving backup wallet to /home/$USER/.local/share/penumbra-testnet-archive/penumbra-valetudo/.../penumbra_wallet.json
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
send them any amount of any asset you have. For example, if I wanted to send 1 tungsten cube token
to my friend, I could do that like this (filling in their full address at the end):

```bash
cargo run --quiet --release --bin pcli tx send 1 tungsten_cube penumbrav0t...
```

If you have the asset in your wallet to send, then so it shall be done!

### Please submit any feedback and bug reports

Thank you for helping us test the Penumbra network! If you have any feedback, please let us know in
the `#testnet-feedback` channel on our [Discord]. We would love to know about bugs, crashes,
confusing error messages, or any of the many other things that inevitably won't quite work yet. Have
fun! :)

## Building the protocol spec

The [protocol spec][protocol] is built using [mdBook] and auto-deployed on
pushes to `main`.  To build it locally:

1. Install the requirements: `cargo install mdbook mdbook-katex mdbook-mermaid`
2. To continuously build and serve the documentation: `mdbook serve`

## Development: Running a Penumbra node

This section is for developers, not for running nodes that are part of the public testnet. We won't
be ready for multiple testnet nodes until we reach our MVP2 milestone implementing staking and
delegation, at which point this will change.

Penumbra has two binaries, the daemon `pd` and the command-line light wallet interface `pcli`.

### Running `pd` with Docker

You might think that this is the preferred way to run Penumbra, **but it will only work if you have
loaded genesis state**:

```bash
docker-compose up --build -d
```

To load genesis state for a fresh Docker configuration:

**NOTE:** this will **destroy** any existing data you have stored in the Docker volumes
for pd/postgres/tendermint!

```bash
./scripts/docker_compose_freshstart.sh
```

The script will handle generating genesis JSON data and copying it to the container volumes
and restarting the containers. You should have a working setup with all containers running
after running the script:

```console
$ docker ps
CONTAINER ID   IMAGE                          COMMAND                  CREATED         STATUS         PORTS                                                                                    NAMES
b7fce1d0ffd9   tendermint/tendermint:latest   "docker-entrypoint.s…"   4 minutes ago   Up 4 minutes   0.0.0.0:6060->6060/tcp, 0.0.0.0:26656-26657->26656-26657/tcp, 0.0.0.0:27000->26660/tcp   tendermint
5a6bd39bb6f7   grafana/grafana:latest         "/run.sh"                4 minutes ago   Up 4 minutes   0.0.0.0:3000->3000/tcp                                                                   penumbra-grafana-1
b8f599963ebc   penumbra_pd                    "pd start --host 0.0…"   4 minutes ago   Up 4 minutes   0.0.0.0:26658->26658/tcp                                                                 penumbra
b4f694a238cb   postgres:13.0                  "docker-entrypoint.s…"   4 minutes ago   Up 4 minutes   0.0.0.0:5432->5432/tcp                                                                   db
9e82aa33b4ff   prom/prometheus:latest         "/bin/prometheus --c…"   4 minutes ago   Up 4 minutes   0.0.0.0:9090->9090/tcp                                                                   penumbra-prometheus-1
```

On production, you should use the production Docker Compose configuration, which will use the
managed database as well as disable various debug build configs used in dev:

```console
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d --build
```

### Details about `pcli`

Now you can interact with Penumbra using `pcli`, for instance

```bash
# Run this first in case the interface changed
# from the sample commands below
$ cargo run --bin pcli -- --help
pcli 0.1.0
The Penumbra command-line interface.

USAGE:
    pcli [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --addr <addr>                          The address of the Tendermint node [default: 127.0.0.1:26657]
    -w, --wallet-location <wallet-location>    The location of the wallet file [default: platform appdata directory]

SUBCOMMANDS:
    addr      Manages addresses
    help      Prints this message or the help of the given subcommand(s)
    query     Queries the Penumbra state
    tx        Creates a transaction
    wallet    Manages the wallet state
```

Keys will be stored in `pcli`'s data directory:

* Linux: `/home/alice/.config/pcli/penumbra_wallet.dat`
* macOS: `/Users/Alice/Library/Application Support/zone.penumbra.pcli/penumbra_wallet.dat`
* Windows: `C:\Users\Alice\AppData\Roaming\penumbra\pcli\penumbra_wallet.dat`

### Running `pd` without using Docker

You'll need to [install Tendermint][tm-install].  Be sure to install `v0.35.0`,
rather than `master`.

Initialize Tendermint:

```bash
tendermint init validator
```

This will create a default genesis file stored in `$TMHOME/config` (if set, else `~/.tendermint/config`) named `genesis.json`.

You probably want to set a log level:

```bash
export RUST_LOG=debug  # bash
```

```fish
set -x RUST_LOG debug  # fish
```

You'll need to set up a Postgres instance. Here is one way:

```bash
# create a volume for pg data
docker volume create tmp_db_data
docker run --name tmp-postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=penumbra -p 5432:5432 -v tmp_db_data:/var/lib/postgresql/data -d postgres
```

Start the Penumbra instance (you probably want to set `RUST_LOG` to `debug`):

```bash
cargo run --bin pd start -d postgres_uri
```

Start the Tendermint node:

```bash
tendermint start
```

You should be running!

To inspect the Postgres state, use:

```bash
psql -h localhost -U postgres penumbra
```

In this database terminal, you can run queries to inspect `pd`'s state.

To reset the Tendermint state, use `tendermint unsafe-reset-all`.  To reset the Postgres state,
either delete the docker volume, or run `DROP DATABASE`, or run `DROP TABLE` for each table. You
need to do **both of these** to fully reset the node, and doing only one will result in mysterious
errors.

### Creating Genesis data

To create Genesis data, you need to know the amounts, denominations, and addresses of the genesis
notes. You can then pass to `pd`'s `create-genesis` command a list of "(amount, denomination,
address)" tuples, where the tuple fields are comma-delimited and each genesis note is contained in
double quotes.  You'll want to change the addresses from this example to addresses you control:

```console
$ cargo run --bin pd -- create-genesis chain-id-goes-here \
"(100, penumbra, penumbratv01p5zmsg23f86azrraspzy8qy9kdm3rnvgfhly0mlzwjqqh9audv59wwjv27nteuplxezqx4x2j99t2rugst00tp0gz30nugxtuttknrk2ma7sa93d26q2w7gse842z3)" \
"(1, tungsten_cube, penumbratv01p5zmsg23f86azrraspzy8qy9kdm3rnvgfhly0mlzwjqqh9audv59wwjv27nteuplxezqx4x2j99t2rugst00tp0gz30nugxtuttknrk2ma7sa93d26q2w7gse842z3)"

[
  {
    "diversifier": "0d05b8215149f5d10c7d80",
    "amount": 100,
    "note_blinding": "4ea7348e26d320ca1740acb775bdfe035da6198f4b86df2c9004fae83193f309",
    "asset_denom": "penumbra",
    "transmission_key": "44438085b37711cd884dfe47efe274800b97bc6b28573a4c57a6bcf03f364403"
  },
  {
    "diversifier": "0d05b8215149f5d10c7d80",
    "amount": 1,
    "note_blinding": "72e4f60cff63ec6ae72cc842b630daaf3f063b4d3a9bc78c4422a772b7fdc409",
    "asset_denom": "tungsten_cube",
    "transmission_key": "44438085b37711cd884dfe47efe274800b97bc6b28573a4c57a6bcf03f364403"
  }
]
```

To perform genesis for a testnet, edit the `genesis.json` file stored in `$TMHOME/config/` or
`~/.tendermint/config/` (see an example in `testnets/genesis_tn001.json`). You should edit the
following fields:

* `validators` key: add the other validators and their voting power,
* `app_state` key: add the generated genesis notes,
* `chain_id` update the `chain_id` for the testnet.

Now when you start `pd` and tendermint as described above, you will see a message at the `INFO`
level indicating genesis has been performed: `consensus: penumbra::app: performing genesis for
chain_id: penumbra-tn001`.

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
