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

Follow [Tendermint's installation instructions](https://docs.tendermint.com/v0.34/introduction/install.html),
but before you start compiling, make sure you are compiling version `v0.34.21`.

```bash
git checkout v0.34.21
```
