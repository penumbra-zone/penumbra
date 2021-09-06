FROM rust:1.54.0 as build

WORKDIR /usr/src
COPY . .

# Fetch dependencies in a separate layer, so that they can be cached.
RUN cargo fetch --locked

RUN cargo build --bin pd --release --frozen && \
    mkdir -p /out && \
    mv target/release/pd /out/pd

# Install the penumbra daemon into the runtime image.

# TODO(eliza): it would be nice to be able to run the Penumbra daemon in a
# `scratch` image rather than Debian or Alpine. However, then we'd have to build
# with a statically linked libc (read: musl), and musl's malloc exhibits
# pathologically poor performance for Tokio applications...
FROM debian:buster-slim as runtime
WORKDIR /penumbra
COPY --from=build /out/pd /usr/bin/pd
ENV RUST_LOG=warn,pd=info,penumbra=info
CMD [ "/usr/bin/pd" ]