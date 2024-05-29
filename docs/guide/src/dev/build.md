# Compiling from source

Penumbra is written in Rust. To build it, you will need a recent
stable version of Rust, as well as a few OS-level dependencies.
We don't support building on Windows. If you need to use Windows,
consider using [WSL] instead.

### Installing the Rust toolchain

This requires that you install a recent (>= 1.75) stable version
of the Rust compiler, installation instructions for which you can find
[here](https://www.rust-lang.org/learn/get-started). Don't forget to reload your shell so that
`cargo` is available in your `$PATH`!

You can verify the rust compiler version by running `rustc --version` which should indicate version 1.75 or later.
The project uses a `rust-toolchain.toml` file, which will ensure that your version of rust stays current enough
to build the project from source.

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

### Linking Against RocksDB (Optional)

Development builds can avoid the cost of recompiling RocksDB for storage libraries in the Cargo
workspace. This manifests as a `librocksdb-sys(build)` message when building or testing crates
in the monorepo.

#### Building `librocksdb.a` from source

First, clone the rocksdb repository:

```sh
# Clone the repository, and enter that directory.
git clone git@github.com:facebook/rocksdb.git && cd rocksdb

# Checkout the version of rocksdb used in `librocksdb-sys`.
git checkout 6a43615

# Add an environment variable pointing to this repository:
ROCKSDB_LIB_DIR=`pwd`

# Compile the static `librocksdb.a` library to link against:
make static_lib
```

#### Building `libsnappy.a` from source

next, clone the snappy repository and follow the
[instructions][snappy-build] to build it:

```
# Clone the repository, and enter that directory.
git clone git@github.com:google/snappy.git && cd snappy

# Checkout the version of snappy used in `librocksdb-sys`.
git checkout 2b63814

# Initialize the submodules.
git submodule update --init

# Build snappy using cmake.
mkdir build
cd build && cmake .. && make

# Add an environment variable pointing to the build/ directory.
SNAPPY_LIB_DIR=`pwd`
```

### Building Penumbra

Once you've built rocksdb and set the environment variable, the `librocksdb-sys` crate will search
in that directory for the compiled `librocksdb.a` static library when it is rebuilt.

[snappy-build]: https://github.com/google/snappy?tab=readme-ov-file#building
[protoc-install]: https://grpc.io/docs/protoc-installation/
[WSL]: https://learn.microsoft.com/en-us/windows/wsl/install
