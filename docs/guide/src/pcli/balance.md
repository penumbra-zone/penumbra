# Viewing Balances

Once you've received your first tokens, you can scan the chain to import them
into your local wallet (this may take a few minutes the first time you run it):

```bash
cargo run --quiet --release --bin pcli view sync
```

Syncing is performed automatically, but running the `sync` subcommand will
ensure that the client state is synced to a recent state, so that future
invocations of `pcli` commands don't need to wait.

If someone sent you testnet assets, you should be able to see them now by running:

```bash
cargo run --quiet --release --bin pcli view balance
```

This will print a table of assets by balance in each.  The `balance` view just
shows asset amounts. To see more information about delegation tokens and the stake they represent, use

```bash
cargo run --quiet --release --bin pcli view staked
```
