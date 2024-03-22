# Compiling from source

Penumbra is written in Rust. To build it, you will need a recent
stable version of Rust, as well as a few OS-level dependencies.
We don't support building on Windows. If you need to use Windows,
consider using [WSL] instead.

### Installing the Rust toolchain

This requires that you install a recent (>= 1.73) stable version
of the Rust compiler, installation instructions for which you can find
[here](https://www.rust-lang.org/learn/get-started). Don't forget to reload your shell so that
`cargo` is available in your `$PATH`!

You can verify the rust compiler version by running `rustc --version` which should indicate version 1.73 or later.

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

### Making sure that `git-lfs` is installed

Running `git lfs install` will make sure that git-lfs is correctly installed on your machine.

### Cloning the repository

Once you have installed the above packages, you can clone the repository:

```bash
git clone https://github.com/penumbra-zone/penumbra
```

To build the versions of `pcli`, `pd`, etc. compatible with the current testnet,
navigate to the `penumbra/` folder, fetch the latest from the repository, and check out the
latest tag for the current
[testnet](https://github.com/penumbra-zone/penumbra/releases):

```bash
cd penumbra && git fetch && git checkout {{ #include ../penumbra_version.md }}
```

If you want to build the most recent version compatible with the "preview" environment,
then run `git checkout main` instead.

### Building the binaries

Then, build all the project binaries using `cargo`:

```bash
cargo build --release
```

[protoc-install]: https://grpc.io/docs/protoc-installation/
[WSL]: https://learn.microsoft.com/en-us/windows/wsl/install
