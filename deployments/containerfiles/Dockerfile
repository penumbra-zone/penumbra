# We use the latest stable version of the official Rust container,
# delegating to the `rust-toolchain.toml` file to pick a specific Rust toolchain.
FROM docker.io/rust:1-slim-bookworm AS build-env

# Install build dependencies. These packages should match what's recommended on
# https://guide.penumbra.zone/main/pcli/install.html
# We don't install git-lfs, because the git artifacts are copied in from the build host.
# In order to build pcli with ledger support, add `libdbus-1-dev` & `libusb-1.0-0-dev`.
RUN apt-get update && apt-get install -y \
        build-essential \
        pkg-config \
        libssl-dev \
        clang

WORKDIR /usr/src/penumbra
# Add rust dependency lockfiles first, to cache downloads.
COPY Cargo.lock Cargo.toml rust-toolchain.toml .

# If any rust code changed, the cache will break on copying `crates/`.
# Ideally we'd copy in all Cargo.toml files first, fetch, then copy crates.
COPY crates ./crates
# Copy static assets.
COPY assets ./assets
# Copy in summonerd contribution orchestrator.
COPY tools ./tools
# Download all workspace dependencies specified in Cargo.toml
RUN rustup install && cargo fetch
# Build Penumbra binaries
COPY . .
# Use `--bins` flag to force dependency resolution per-binary, rather than once for the workspace.
# Doing so ensures the minimal feature set is used, leaving ledger support for pcli as off by default.
RUN cargo build --release --bins

# Runtime image.
FROM docker.io/debian:bookworm-slim
ARG USERNAME=penumbra
ARG UID=1000
ARG GID=1000
# We add curl & jq so we can munge JSON during init steps for deployment.
RUN apt-get update && apt-get install -y \
        curl \
        jq \
        libssl-dev

# Add normal user account
RUN groupadd --gid ${GID} ${USERNAME} \
        && useradd -m -d /home/${USERNAME} -g ${GID} -u ${UID} ${USERNAME}

# Install chain binaries
COPY --from=build-env \
            /usr/src/penumbra/target/release/elcuity \
            /usr/src/penumbra/target/release/pcli \
            /usr/src/penumbra/target/release/pclientd \
            /usr/src/penumbra/target/release/pd \
            /usr/src/penumbra/target/release/pindexer \
            /usr/src/penumbra/target/release/pmonitor \
            /usr/src/penumbra/target/release/tct-live-edit \
            /usr/bin/

WORKDIR /home/${USERNAME}
USER ${USERNAME}
CMD [ "/usr/bin/pd" ]
