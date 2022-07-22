# Building pd

The node software `pd` is part of the same repository as `pcli`, so follow
[those instructions](../pcli/install.md) to clone the repo and install dependencies.

You may need to install some additional packages in order to build `pd`,
depending on your distribution. For a bare-bones Ubuntu installation, you can
run

```bash
sudo apt-get install clang
```

To build `pd`, run

```bash
cargo build --release --bin pd
```

Because you are building a work-in-progress version of the node, you may see compilation warnings,
which you can safely ignore.

### Installing tendermint

You'll need to have [tendermint installed](https://docs.tendermint.com/v0.35/introduction/install.html) on your system to join your node to the testnet.

**NOTE**: be sure to install the correct version of Tendermint (`v0.35.9`).
**Do not use** Tendermint `0.35.8` or earlier, which has bugs in the p2p layer
that can prevent nodes from staying online.
