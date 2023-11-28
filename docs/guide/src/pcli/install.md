# Installing pcli

Download prebuilt binaries from the [Penumbra releases page on Github](https://github.com/penumbra-zone/penumbra/releases).
Make sure to use the most recent version available, as the version of `pcli` must
match the software currently running on the network.

Make sure choose the correct platform for your machine. After downloading the `.tar.xz` file,
extract it, and copy its contents to your `$PATH`. For example:

```
curl -O -L https://github.com/penumbra-zone/penumbra/releases/download/{{ #include ../penumbra_version.md }}/pcli-x86_64-unknown-linux-gnu.tar.xz
unxz pcli-x86_64-unknown-linux-gnu.tar.xz
tar -xf pcli-x86_64-unknown-linux-gnu.tar
sudo mv pcli-x86_64-unknown-linux-gnu/pcli /usr/local/bin/

# confirm the pcli binary is installed by running:
pcli --version
```

Only macOS and Linux are supported. If you need to use Windows,
consider using [WSL]. If you prefer to build from source,
see the [compilation guide](../dev/build.md).

[WSL]: https://learn.microsoft.com/en-us/windows/wsl/install
