# Devnet quickstart

This page assumes you've set up a [Penumbra developer environment](./dev-env.md),
as it references several commands like `cometbft` in order to work.
It describes how to run a Penumbra fullnode on your local workstation, for building
and testing Penumbra and related services.

## Running a local devnet

To generate a devnet genesis and run a Penumbra fullnode locally, run:

```shell
just dev
```

Running that command will: 
  
  1. build the local rust code from th elatest version on-disk
  2. generate a genesis file, writing configs to `~/.penumbra/network_data/`
  3. run the locally-built version of `pd`
  4. run `cometbft` alongside `pd`, communicating over ABCI
  5. run a prometheus/grafana [metrics setup](./metrics.md)

You can use the [process-compose] interface to view logs from any individual service.
Use `ctrl+c` to halt the setup, and run `just dev` to start it again from the latest
local source code.

## Running `pcli`
<!--
TODO:
The dev env should generate an ad-hoc wallet, and/or accept an env var of a Penumbra wallet address,
and add that wallet addr to the generated devnet genesis. Then the user could immediately get
started with read/write interactions via pcli.
-->

To interact with the chain, configure a wallet pointing at the localhost node:

```shell
cargo run --release --bin pcli -- --home ~/.local/share/pcli-localhost view reset
cargo run --release --bin pcli -- init --grpc-url http://localhost:8080 soft-kms generate
# or, to reuse an existing seed phrase:
cargo run --release --bin pcli -- init --grpc-url http://localhost:8080 soft-kms import-phrase
```

and then pass the `--home` flag to any commands you run to point `pcli` at your local node, e.g.,

```shell
cargo run --release --bin pcli -- --home ~/.local/share/pcli-localhost view balance
```

By default, `pd network generate` uses the testnet allocations from the `testnets/` directory in the git repo.
If you have an address included in those files, then use `pcli init soft-kms import-phrase`. Otherwise,
edit the `genesis.json` to add your address.

## Resetting and restarting

After making changes, you may want to reset and restart the devnet:

```shell
# stop the running node via process-compose:
ctrl+c
# destroy local network state:
cargo run --release --bin pd -- network unsafe-reset-all
```

You'll probably also want to reset your wallet state:

```shell
cargo run --release --bin pcli -- --home ~/.local/share/pcli-localhost view reset
```

At this point you're ready to generate new configs, and restart both `pd` and
`cometbft`, which you can do by running `just dev` again. Note that running `just dev`
will _reuse_ any existing state in `~/.penumbra/network_data/`. You must manually `unsafe-reset-all`
to purge that pre-existing config.

## Running smoke tests (optional)

Once you have a working devnet running, you should be able to run the [smoke tests](https://en.wikipedia.org/wiki/Smoke_testing_(software))
successfully. This can be useful if you are looking to contribute to Penumbra, or if you need to check that your setup is correct.
To run the smoke tests:

1. Make sure you have a devnet running (see previous steps)
2. Run integration tests:
```shell
PENUMBRA_NODE_PD_URL=http://127.0.0.1:8080 PCLI_UNLEASH_DANGER=yes cargo test --package pcli -- --ignored --test-threads 1
```

You can also run the entire smoke test suite with an automatic fullnode. If you do so, 
make sure to stop any fullnode running via `just dev`, as they'll conflict. Then run `just smoke`.
If you want to execute the tests against an already-running devnet, however, use manual invocations like
the `cargo test` example above.

[process-compose]: https://f1bonacc1.github.io/process-compose/launcher/
