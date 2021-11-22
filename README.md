[Penumbra] is a fully shielded zone for the Cosmos ecosystem, providing private
trading in any cryptoasset.

## Getting involved

The primary communication hub is our [Discord]; click the link to join the
discussion there.

The (evolving) protocol spec is rendered at [protocol.penumbra.zone][protocol].

The (evolving) API documentation is rendered at [rustdoc.penumbra.zone][rustdoc].

## Building the protocol spec

The [protocol spec][protocol] is built using [mdBook] and auto-deployed on
pushes to `main`.  To build it locally:

1. Install the requirements: `cargo install mdbook mdbook-katex mdbook-mermaid`
2. To continuously build and serve the documentation: `mdbook serve`

## Running Penumbra

Penumbra has two binaries, the daemon `pd` and the command-line wallet interface `pcli`.

### Running `pd` with Docker

This is the preferred way to run Penumbra:
```
docker-compose up --build -d
```

### Running `pcli`

Now you can interact with Penumbra using `pcli`, for instance
```bash
# Run this first in case the interface changed
# from the sample commands below
cargo run --bin pcli -- --help
```
to get
```
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

### Running `pd` manually

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

You'll need to set up a Postgres instance.  Here is one way:
```bash
# create a volume for pg data
docker volume create tmp_db_data
docker run --name tmp-postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=penumbra -p 5432:5432 -v tmp_db_data:/var/lib/postgresql/data -d postgres
```

Start the Penumbra instance (you probably want to set `RUST_LOG` to `debug`):
```
cargo run --bin pd start -d postgres_uri
```
Start the Tendermint node:
```
tendermint start
```

You should be running!  

To inspect the Postgres state, use
```
psql -h localhost -U postgres penumbra
```
allowing you to run queries.

To reset the Tendermint state, use `tendermint unsafe-reset-all`.  To reset the
Postgres state, either delete the docker volume, or run `DROP DATABASE`, or run
`DROP TABLE` for each table.

### Genesis data

To create Genesis data, you need to know the amounts, denominations, and addresses of the genesis notes. You can then pass to `pd`'s` `create-genesis` command a list of "(amount, denomination, address)" tuples, where the tuple fields are comma-delimited and each genesis note is contained in double quotes:
```console
$ cargo run --bin pd -- create-genesis penumbra-tn001 \
"(100, pen, penumbra_tn001_1kpgdhlzws6kyk2cf580wtt76t9nn2vf7em3pn05y3h8ym5a6aevdxshjgsnxecv94rzsxdhng6cjp8kgchqxud06p9xka0yxv99rty3njetqqnx2hrzz4tc03956e0)" \
"(1, tungsten_cube, penumbra_tn001_1kpgdhlzws6kyk2cf580wtt76t9nn2vf7em3pn05y3h8ym5a6aevdxshjgsnxecv94rzsxdhng6cjp8kgchqxud06p9xka0yxv99rty3njetqqnx2hrzz4tc03956e0)"

{
  [
    {
      "diversifier": "b050dbfc4e86ac4b2b09a1",
      "amount": 100,
      "note_blinding": "93f7245c0e0265338ed54db574462d16a366187d3f2ff361aa94ecddadfbb103",
      "asset_denom": "pen",
      "transmission_key": "dee5afda596735313ecee219be848dce4dd3baee58d342f244266ce185a8c503"
    },
    {
      "diversifier": "b050dbfc4e86ac4b2b09a1",
      "amount": 1,
      "note_blinding": "7793daf7ac4ef421d6ad138675180b37b866cc5ca0297a846fb9301d9deb2c0d",
      "asset_denom": "tungsten_cube",
      "transmission_key": "dee5afda596735313ecee219be848dce4dd3baee58d342f244266ce185a8c503"
    }
  ]
}
```

To perform genesis for a testnet, edit the `genesis.json` file stored in `$TMHOME/config/` or `~/.tendermint/config/` (see an example in `testnets/genesis_tn001.json`). You should edit the following fields:
* `validators` key: add the other validators and their voting power,
* `app_state` key: add the generated genesis notes,
* `chain_id` update the `chain_id` for the testnet.

Now when you start `pd` and tendermint as described above, you will see a message at the `INFO` level indicating genesis has been performed: `consensus: penumbra::app: performing genesis for chain_id: penumbra-tn001`.

[Discord]: https://discord.gg/hKvkrqa3zC
[Penumbra]: https://penumbra.zone
[protocol]: https://protocol.penumbra.zone
[mdBook]: https://github.com/rust-lang/mdBook
[tm-install]: https://github.com/tendermint/tendermint/blob/master/docs/introduction/install.md#from-source
