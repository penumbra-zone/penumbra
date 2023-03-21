# penumbra-web-assembly

Builds WASM targets for using Penumbra code in a web context,
such as a web extension. Originally written by the Zpoken team.
See #1990 for historical info.

## Building

```
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
cargo install wasm-pack
wasm-pack build
```

Artifacts can be found in `pkg/`.
