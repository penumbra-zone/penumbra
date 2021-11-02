# Running Penumbra

Penumbra has two binaries, the daemon `pd` and the interface `pcli`.

You'll need to [install Tendermint](https://github.com/tendermint/tendermint/blob/master/docs/introduction/install.md#from-source).

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
cargo run --bin pd -- start
```
Start the Tendermint node:
```
tendermint start
```

You should be running!  To reset the Tendermint state, use `tendermint unsafe-reset-all`.

Now you can interact with Penumbra using `pcli`:
```
cargo run --bin pcli -- --help
```

## Genesis

To create Genesis data, you need to know the amounts, denominations, and addresses. You can then pass to `pd`'s` `create-genesis` command a `|`-delimited list of (amount, denomination, address) tuples, where the fields are comma-delimited:

```
$ cargo run --bin pd -- create-genesis penumbra \
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