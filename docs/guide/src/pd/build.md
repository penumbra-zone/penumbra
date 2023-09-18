# Building pd

The node software `pd` is part of the same repository as `pcli`, so follow
[those instructions](../pcli/install.md) to clone the repo and install dependencies.

To build `pd`, run

```bash
cargo build --release --bin pd
```

Because you are building a work-in-progress version of the node software, you may see compilation warnings,
which you can safely ignore.

### Installing CometBFT

You'll need to have [CometBFT installed](https://docs.cometbft.com/v0.34/guides/install)
on your system to join your node to the testnet.

**NOTE**: Previous versions of Penumbra used Tendermint, but as of Testnet 61 (released 2023-09-25),
only CometBFT is supported. **Do not use** any version of Tendermint, which may not work with `pd`.

Follow the [CometBFT installation instructions](https://docs.cometbft.com/v0.34/guides/install)
to install a binary. If you prefer to compile from source instead,
make sure you are compiling version `v0.34.27`:

```bash
git checkout v0.34.27
```
