# Building pd

The node software `pd` is part of the same repository as `pcli`, so follow
[those instructions](../pcli/install.md) to clone the repo and install dependencies.

To build `pd`, run
```bash
cargo build --release --bin pd
```

Because you are building a work-in-progress version of the node, you may see compilation warnings,
which you can safely ignore.

### Installing tendermint

You'll need to have [tendermint installed](https://docs.tendermint.com/v0.35/introduction/install.html) on your system to join your node to the testnet.

**NOTE**: be sure to install the correct version of Tendermint (`v0.35`).
