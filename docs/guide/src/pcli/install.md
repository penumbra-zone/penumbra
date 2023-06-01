# Installing `pcli`

### Installing the Rust toolchain

This requires that you install a recent stable version
of the Rust compiler, installation instructions for which you can find
[here](https://www.rust-lang.org/learn/get-started). Don't forget to reload your shell so that
`cargo` is available in your `\$PATH`!

`pcli` requires `rustfmt` as part of the build process --- depending on your
OS/install method for Rust, you may have to install that separately.

### Installing build prerequisites

#### Linux

You may need to install some additional packages in order to build `pcli`,
depending on your distribution. For a bare-bones Ubuntu installation, you can
run:

```bash
sudo apt-get install build-essential pkg-config libssl-dev clang git-lfs
```

For a minimal Fedora/CentOS/RHEL image, you can run:

```bash
sudo dnf install openssl-devel clang git cargo rustfmt git-lfs
```

#### macOS

You may need to install the command-line developer tools if you have never done
so:
```bash
xcode-select --install
```

You'll also need to install Git LFS, which you can do [via Homebrew](https://docs.github.com/en/repositories/working-with-files/managing-large-files/installing-git-large-file-storage?platform=mac):

```bash
brew install git-lfs
```

### Cloning the repository

Once you have installed the above tools, you can clone the repository:

```bash
git clone https://github.com/penumbra-zone/penumbra
```

To build the version of `pcli` compatible with the current testnet, navigate to
the penumbra folder, fetch the latest from the repository, and check out the
latest tag for the current
[testnet](https://github.com/penumbra-zone/penumbra/releases):

```bash
cd penumbra && git fetch && git checkout v0.53.0
```

### Building the `pcli` client software

Then, build the `pcli` tool using `cargo`:

```bash
cargo build --release --bin pcli
```

Because you are building a work-in-progress version of the client, you may see compilation warnings,
which you can safely ignore.

[protoc-install]: https://grpc.io/docs/protoc-installation/
