# Installing pd
There are many ways to configure and run Penumbra. The easiest is to download
binaries for `pd` and `cometbft` on a Linux system. For alternatives, see
[deployment strategies](./requirements.md#deployment-strategies).
If you want a detailed guide, see the [tutorial on running a node](../../tutorials/running-node.md).

## Quickstart
Download prebuilt binaries from the [Penumbra releases page on Github](https://github.com/penumbra-zone/penumbra/releases).
Make sure to use the most recent version available, as the version of `pd` must
match the software currently running on the network, to choose the correct platform for your machine.

After downloading the `.tar.gz` file, extract it, and copy its contents to your `$PATH`. For example:

```
curl -sSfL -O https://github.com/penumbra-zone/penumbra/releases/download/{{ #include ../../penumbra_version.md }}/pd-x86_64-unknown-linux-gnu.tar.gz
tar -xf pd-x86_64-unknown-linux-gnu.tar.gz
sudo mv pd-x86_64-unknown-linux-gnu/pd /usr/local/bin/

# confirm the pd binary is installed by running:
pd --version
```

There's also a one-liner install script available on the release page, which will install `pd` to `$HOME/.cargo/bin/`.

### Installing CometBFT

You'll need to have [CometBFT installed](https://docs.cometbft.com/v0.37/guides/install) on your system to join your node to the testnet.
You must use a compatible version of CometBFT. Any version in the `v0.37.x` series will work, such as `{{ #include ../../cometbft_version.md }}`,
which you can download [from the CometBFT releases page](https://github.com/cometbft/cometbft/releases/tag/{{ #include ../../cometbft_version.md }}).
If you prefer to compile from source instead, make sure you are compiling the correct version by checking out its tag
in the CometBFT repo before building.
