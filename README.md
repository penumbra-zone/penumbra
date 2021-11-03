<img style="width: 25%; max-width: 11em; float: left; margin: 1em;" src="https://protocol.penumbra.zone/penumbra-glow-alpha-mask.png">

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
docker-compose up -d
```

### Running `pcli`

Now you can interact with Penumbra using `pcli`:
```
cargo run --bin pcli -- --help
```

Generate keys using:

```
cargo run --bin pcli -- generate
```

Keys will be stored in `wallet.dat` in the current working directory. To customize the location, use the `--key-location <file>` option.

### Running `pd` manually

You'll need to [install Tendermint][tm-install].  Be sure to install `v0.35.0`,
rather than `master`.

Initialize Tendermint:
```bash
tendermint init validator
```

You probably want to set a log level:
```bash
export RUST_LOG=debug  # bash
```
```fish
set -x RUST_LOG debug  # fish
```

Start the Penumbra instance (you probably want to set `RUST_LOG` to `debug`):
```
cargo run --bin pd
```
Start the Tendermint node:
```
tendermint start
```

You should be running!  To reset the Tendermint state, use `tendermint unsafe-reset-all`.

### Genesis data

To create Genesis data, you need to know the amounts, denominations, and addresses of the genesis notes. You can then pass to `pd`'s` `create-genesis` command a list of "(amount, denomination, address)" tuples, where the tuple fields are comma-delimited and each genesis note is contained in double quotes:
```
$ cargo run --bin pd -- create-genesis penumbra-tn001 \
"(100, pen, penumbra_tn001_1kpgdhlzws6kyk2cf580wtt76t9nn2vf7em3pn05y3h8ym5a6aevdxshjgsnxecv94rzsxdhng6cjp8kgchqxud06p9xka0yxv99rty3njetqqnx2hrzz4tc03956e0)" \
"(1, tungsten_cube, penumbra_tn001_1kpgdhlzws6kyk2cf580wtt76t9nn2vf7em3pn05y3h8ym5a6aevdxshjgsnxecv94rzsxdhng6cjp8kgchqxud06p9xka0yxv99rty3njetqqnx2hrzz4tc03956e0)"

{
  "notes": [
    {
      "diversifier": "b050dbfc4e86ac4b2b09a1",
      "amount": 100,
      "note_blinding": "fb5b430096940592704c911b0fdef6ae324f054ddc6ea8cb13555373af81a00b",
      "asset_id": "3213674d74c0f0a10b786838225460288830940147ed6f66bbae7af1ed759101",
      "transmission_key": "dee5afda596735313ecee219be848dce4dd3baee58d342f244266ce185a8c503"
    },
    {
      "diversifier": "b050dbfc4e86ac4b2b09a1",
      "amount": 1,
      "note_blinding": "2ca0a10d8f76a24f11e72ca1d21e18a16b112d98acdaeb62f6dde519297d080f",
      "asset_id": "7bcfef24592b30affe8f1970d719ad4a2c5930570477e5e29be80d97816edf0f",
      "transmission_key": "dee5afda596735313ecee219be848dce4dd3baee58d342f244266ce185a8c503"
    }
  ]
}
```



[Discord]: https://discord.gg/hKvkrqa3zC
[Penumbra]: https://penumbra.zone
[protocol]: https://protocol.penumbra.zone
[mdBook]: https://github.com/rust-lang/mdBook
[tm-install]: https://github.com/tendermint/tendermint/blob/master/docs/introduction/install.md#from-source
