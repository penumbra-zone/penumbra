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
cargo run --bin pd
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
