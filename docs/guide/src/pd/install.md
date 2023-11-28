# Installing pd

Download prebuilt binaries from the [Penumbra releases page on Github](https://github.com/penumbra-zone/penumbra/releases).
Make sure to use the most recent version available, as the version of `pd` must
match the software currently running on the network.

Make sure choose the correct platform for your machine. After downloading the `.tar.xz` file,
extract it, and copy its contents to your `$PATH`. For example:

```
curl -O -L https://github.com/penumbra-zone/penumbra/releases/download/{{ #include ../penumbra_version.md }}/pd-x86_64-unknown-linux-gnu.tar.xz
unxz pd-x86_64-unknown-linux-gnu.tar.xz
tar -xf pd-x86_64-unknown-linux-gnu.tar
sudo mv pd-x86_64-unknown-linux-gnu/pd /usr/local/bin/

# confirm the pd binary is installed by running:
pd --version
```

If you prefer to build from source, see the [compilation guide](../dev/build.md).

### Installing CometBFT

You'll need to have [CometBFT installed](https://docs.cometbft.com/v0.37/guides/install)
on your system to join your node to the testnet.

You can download `v0.37.2` [from the CometBFT releases page](https://github.com/cometbft/cometbft/releases/tag/v0.37.2)
to install a binary. If you prefer to compile from source instead,
make sure you are compiling version `v0.37.2`.

**NOTE**: Previous versions of Penumbra used Tendermint, but as of Testnet 62 (released 2023-10-10),
only CometBFT `v0.37.2` is supported. **Do not use** any version of Tendermint, which will not work with `pd`.
