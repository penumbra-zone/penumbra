# Installing pcli

Download prebuilt binaries from the [Penumbra releases page on Github](https://github.com/penumbra-zone/penumbra/releases).
Make sure to use the most recent version available, as the version of `pcli` must
match the software currently running on the network.

Make sure choose the correct platform for your machine. Or, you can use a one-liner install script:

```
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/penumbra-zone/penumbra/releases/download/{{ #include ../penumbra_version.md }}/pcli-installer.sh | sh

# confirm the pcli binary is installed by running:
pcli --version
```

The installer script will place the binary in `$HOME/.cargo/bin/`.

If you see an error message containing `GLIBC`, then your system is not compatible
with the precompiled binaries. See details below.

## Platform support

Only modern versions of Linux and macOS are supported, such as:

  * Ubuntu 22.04
  * Debian 12
  * Fedora 39
  * macOS 14

When checking the locally installed binary via `pcli --version`, you may see an error message similar to:

```
pcli: /lib/x86_64-linux-gnu/libstdc++.so.6: version `GLIBCXX_3.4.30' not found (required by pcli)
pcli: /lib/x86_64-linux-gnu/libstdc++.so.6: version `GLIBCXX_3.4.29' not found (required by pcli)
pcli: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.32' not found (required by pcli)
pcli: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.34' not found (required by pcli)
pcli: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.33' not found (required by pcli)
```

If you see that message, you must either switch to a supported platform, or else
[build the software from source](../dev/build.md). If you need to use Windows,
consider using [WSL].

[WSL]: https://learn.microsoft.com/en-us/windows/wsl/install
