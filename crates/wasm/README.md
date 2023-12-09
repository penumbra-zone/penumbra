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

## Testing
The WASM crate incorporates a unit testing suite that leverages `wasm-bindgen-test` to simulate a sample spend transaction.
For testing purposes, the suite mocks `indexDB` database calls in an interactive browser accessible at `http://127.0.0.1:8000`.

```
wasm-pack test --chrome -- --test test_build --target wasm32-unknown-unknown --release --features "mock-database"
```

The transaction outputs are accessible in the browser's developer console.

The wasm browser tests are also run headlessly in Penumbra CI. Review the relevant GHA workflow file for specifics.
