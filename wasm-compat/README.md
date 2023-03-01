# penumbra-wasm

This is dummy package, intended to track support for WASM
in the Penumbra dependencies. There's no project code,
just an empty skeleton app. The `Cargo.toml` declares
relative imports for the Penumbra crates within the workspace
at the top-level of the Penumbra git repository.

## Building

```
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```
