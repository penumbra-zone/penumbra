# Installing pd

Download prebuilt binaries from the [Penumbra releases page on Github](https://github.com/penumbra-zone/penumbra/releases).
Make sure to use the most recent version available, as the version of `pd` must
match the software currently running on the network.

Make sure to choose the correct platform for your machine. After downloading the `.tar.xz` file,
extract it, and copy its contents to your `$PATH`. For example:

```
curl -sSfL -O https://github.com/penumbra-zone/penumbra/releases/download/{{ #include ../penumbra_version.md }}/pd-x86_64-unknown-linux-gnu.tar.xz
unxz pd-x86_64-unknown-linux-gnu.tar.xz
tar -xf pd-x86_64-unknown-linux-gnu.tar
sudo mv pd-x86_64-unknown-linux-gnu/pd /usr/local/bin/

# confirm the pd binary is installed by running:
pd --version
```
As of v0.64.1 (released 2023-12-12), we build Linux binaries on Ubuntu 22.04. If these binaries don't work for you out of the box,
you'll need to [build from source](../dev/build.md), or use the container images.

### Installing CometBFT

You'll need to have [CometBFT installed](https://docs.cometbft.com/v0.37/guides/install)
on your system to join your node to the testnet.

You must use a specific version of CometBFT, `{{ #include ../cometbft_version.md }}`, which you can download
[from the CometBFT releases page](https://github.com/cometbft/cometbft/releases/tag/{{ #include ../cometbft_version.md }}).
If you prefer to compile from source instead, make sure you are compiling version `{{ #include ../cometbft_version.md }}`.

Previous versions of Penumbra used Tendermint, but as of Testnet 62 (released 2023-10-10),
only CometBFT is supported. **Do not use** any version of Tendermint, which will not work with `pd`.
