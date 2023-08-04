ARG RUST_VERSION=1.71.1
FROM docker.io/rust:${RUST_VERSION}-slim-bookworm AS build-env

# Install build dependencies. These packages should match what's recommended on
# https://guide.penumbra.zone/main/pcli/install.html
# We don't install git-lfs, because the git artifacts are copied in from the build host.
RUN apt-get update && apt-get install -y \
        build-essential \
        pkg-config \
        libssl-dev \
        clang

WORKDIR /usr/src/penumbra
# Add rust dependency lockfiles first, to cache downloads.
COPY Cargo.lock Cargo.toml .
COPY crates ./crates
RUN cargo fetch
# Unfortunately, container layer cache is busted copying in the .git
# dir, which is required for using vergen, so we rebuild the rust
# code from source fresh.
COPY . .
RUN cargo build --release

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
            /usr/src/penumbra/target/release/pcli \
            /usr/src/penumbra/target/release/pclientd \
            /usr/src/penumbra/target/release/pd \
            /usr/src/penumbra/target/release/tct-live-edit \
            /usr/bin/

WORKDIR /home/${USERNAME}
USER ${USERNAME}
CMD [ "/usr/bin/pd" ]
