FROM --platform=$BUILDPLATFORM rust:1.65.0-bullseye AS build-env

RUN rustup component add rustfmt

ARG TARGETARCH
ARG BUILDARCH

RUN if [ "${TARGETARCH}" = "arm64" ]; then \
      rustup target add aarch64-unknown-linux-gnu; \
      if [ "${BUILDARCH}" != "arm64" ]; then \
        dpkg --add-architecture arm64; \
        apt update && apt install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu; \
        ln -s /usr/aarch64-linux-gnu/include/bits /usr/include/bits; \
        ln -s /usr/aarch64-linux-gnu/include/sys /usr/include/sys; \
        ln -s /usr/aarch64-linux-gnu/include/gnu /usr/include/gnu; \
      else \
        apt update; \
      fi; \
      apt install -y libssl1.1:arm64 libssl-dev:arm64 openssl:arm64 libclang-dev clang; \
    elif [ "${TARGETARCH}" = "amd64" ]; then \
      rustup target add x86_64-unknown-linux-gnu; \
      if [ "${BUILDARCH}" != "amd64" ]; then \
        dpkg --add-architecture amd64; apt update; \
        apt update && apt install -y gcc-x86_64-linux-gnu g++-x86_64-linux-gnu; \
        ln -s /usr/x86_64-linux-gnu/include/bits /usr/include/bits; \
        ln -s /usr/x86_64-linux-gnu/include/sys /usr/include/sys; \
        ln -s /usr/x86_64-linux-gnu/include/gnu /usr/include/gnu; \
      else \
        apt update; \
      fi; \
      apt install -y libssl1.1:amd64 libssl-dev:amd64 openssl:amd64 libclang-dev clang; \
    fi

WORKDIR /usr/src

ADD . .

RUN if [ "$TARGETARCH" = "arm64" ] && [ "$BUILDARCH" != "arm64" ]; then \
      cargo fetch --target aarch64-unknown-linux-gnu; \
    elif [ "$TARGETARCH" = "amd64" ] && [ "$BUILDARCH" != "amd64" ]; then \
      cargo fetch --target x86_64-unknown-linux-gnu; \
    else \
      cargo fetch --target $(uname -m)-unknown-linux-gnu; \
    fi;

RUN if [ "$TARGETARCH" = "arm64" ] && [ "$BUILDARCH" != "arm64" ]; then \
      export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
        CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
        CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
        PKG_CONFIG_SYSROOT_DIR=/usr/aarch64-linux-gnu; \
      cargo build --release --target aarch64-unknown-linux-gnu; \
    elif [ "$TARGETARCH" = "amd64" ] && [ "$BUILDARCH" != "amd64" ]; then \
      export CARGO_TARGET_x86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
        CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc \
        CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++ \
        PKG_CONFIG_SYSROOT_DIR=/usr/x86_64-linux-gnu; \
      cargo build --release --target x86_64-unknown-linux-gnu; \
    else \
      cargo build --release --target $(uname -m)-unknown-linux-gnu;\
    fi;

# Copy all binaries to /root/bin, for a single place to copy into final image.
RUN mkdir /root/bin
RUN if [ "${TARGETARCH}" = "arm64" ]; then ARCH=aarch64; \
    elif [ "${TARGETARCH}" = "amd64" ]; then ARCH=x86_64; fi; \
    cp /usr/src/target/${ARCH}-unknown-linux-gnu/release/pcli \
      /usr/src/target/${ARCH}-unknown-linux-gnu/release/pd \
      /usr/src/target/${ARCH}-unknown-linux-gnu/release/pclientd \
      /root/bin

# Use minimal busybox from Strangelove infra-toolkit image for final scratch image
FROM ghcr.io/strangelove-ventures/infra-toolkit:v0.0.6 AS busybox-min
RUN addgroup --gid 1000 -S penumbra && adduser --uid 1000 -S penumbra -G penumbra

# Use ln and rm from full featured busybox for assembling final image
FROM busybox:1.34.1-musl AS busybox-full

# Use TARGETARCH image for determining necessary libs
FROM rust:1.65.0-bullseye as target-arch-libs
RUN apt update && apt install -y clang libssl1.1 openssl

# Determine library dependencies of built binaries and copy to indexed path in /root/lib_abs for copying to final image.
# Absolute path of each library is appended to /root/lib_abs.list for restoring in final image.
COPY --from=build-env /root/bin /root/bin
RUN mkdir -p /root/lib_abs && touch /root/lib_abs.list
RUN bash -c \
  'readarray -t LIBS < <( for b in $(ls -1 /root/bin/* | sort -u) ; do ldd $b | awk "{print \$1, \$2, \$3 }" | sort -u; done | sort -u ; ) ; \
    i=0; for LIB in "${LIBS[@]}"; do \
      PATH1=$(echo $LIB | awk "{print \$1}") ; \
      if [ "$PATH1" = "linux-vdso.so.1" ]; then continue; fi; \
      if [ "$PATH1" = "/lib/ld-linux-aarch64.so.1" ]; then continue; fi; \
      PATH2=$(echo $LIB | awk "{print \$3}") ; \
      if [ -n "$PATH2" ]; then \
        cp $PATH2 /root/lib_abs/$i ; \
        echo $PATH2 >> /root/lib_abs.list; \
      else \
        cp $PATH1 /root/lib_abs/$i ; \
        echo $PATH1 >> /root/lib_abs.list; \
      fi; \
      ((i = i + 1)) ;\
  done'

# Build final image from scratch
FROM scratch

WORKDIR /bin

# Install ln (for making hard links), rm (for cleanup), mv, mkdir, and dirname from full busybox image (will be deleted, only needed for image assembly)
COPY --from=busybox-full /bin/ln /bin/rm /bin/mv /bin/mkdir /bin/dirname ./

# Install minimal busybox image as shell binary (will create hardlinks for the rest of the binaries to this data)
COPY --from=busybox-min /busybox/busybox /bin/sh

# Add hard links for read-only utils, then remove ln and rm
# Will then only have one copy of the busybox minimal binary file with all utils pointing to the same underlying inode
RUN ln sh pwd && \
    ln sh ls && \
    ln sh cat && \
    ln sh less && \
    ln sh grep && \
    ln sh sleep && \
    ln sh env && \
    ln sh tar && \
    ln sh tee && \
    ln sh du

# Install chain binaries
COPY --from=build-env /root/bin /bin

# Copy over libraries
COPY --from=target-arch-libs /root/lib_abs /root/lib_abs
COPY --from=target-arch-libs /root/lib_abs.list /root/lib_abs.list

# Move libraries to their absolute locations.
RUN sh -c 'i=0; while read FILE; do \
      echo "$i: $FILE"; \
      DIR="$(dirname "$FILE")"; \
      mkdir -p "$DIR"; \
      mv /root/lib_abs/$i $FILE; \
      i=$((i+1)); \
    done < /root/lib_abs.list'

# Remove write utils used to construct image and tmp dir/file for lib copy.
RUN rm -rf ln rm mv mkdir dirname /root/lib_abs /root/lib_abs.list

# Install trusted CA certificates
COPY --from=busybox-min /etc/ssl/cert.pem /etc/ssl/cert.pem

# Install penumbra user
COPY --from=busybox-min /etc/passwd /etc/passwd
COPY --from=busybox-min --chown=1000:1000 /home/penumbra /home/penumbra

WORKDIR /home/penumbra
USER penumbra

ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL
ENV RUST_LOG=warn,pd=info,penumbra=info
CMD [ "/bin/pd" ]
