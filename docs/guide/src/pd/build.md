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

### Installing Tendermint

You'll need to have [Tendermint installed](https://docs.tendermint.com/v0.34/introduction/install.html)
on your system to join your node to the testnet. 

**NOTE**: We are currently dependent on `v0.35.9` of Tendermint, even though it has been
[officially depracted](https://interchain-io.medium.com/discontinuing-tendermint-v0-35-a-postmortem-on-the-new-networking-layer-3696c811dabc)
by the Tendermint Council. We are in the process of [rolling back to v0.34](https://github.com/penumbra-zone/penumbra/issues/1271).
In the mean time, be sure to install the correct version of Tendermint (`v0.35.9`).
**Do not use** Tendermint `0.35.8` or earlier, which has bugs in the p2p layer
that can prevent nodes from staying online.

Follow [Tendermint's installation instructions](https://docs.tendermint.com/v0.34/introduction/install.html),
but before you start compiling, make sure you are compiling version `v0.35.9`.

```bash
git checkout v0.35.9
```
