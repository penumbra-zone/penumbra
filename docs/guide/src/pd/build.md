# Building pd

The node software `pd` is part of the same repository as `pcli`, so follow
[those instructions](../pcli/install.md) to clone the repo and install dependencies.

To build `pd`, run

```bash
cargo build --release --bin pd
```

Because you are building a work-in-progress version of the node software, you may see compilation warnings,
which you can safely ignore.

### Installing Tendermint

You'll need to have [Tendermint installed](https://docs.tendermint.com/v0.34/introduction/install.html)
on your system to join your node to the testnet. 

**NOTE**: Previous versions of Penumbra used Tendermint 0.35, which
has now been [officially
deprecated](https://interchain-io.medium.com/discontinuing-tendermint-v0-35-a-postmortem-on-the-new-networking-layer-3696c811dabc)
by the Tendermint Council. We have now [rolled back to
v0.34](https://github.com/penumbra-zone/penumbra/issues/1271).
**Do not use** Tendermint `0.35`, which will no longer work with `pd`.
that can prevent nodes from staying online.

Follow [Tendermint's installation instructions](https://docs.tendermint.com/v0.34/introduction/install.html),
but before you start compiling, make sure you are compiling version `v0.34.23`.

```bash
git checkout v0.34.23
```

[protoc-install]: https://grpc.io/docs/protoc-installation/
