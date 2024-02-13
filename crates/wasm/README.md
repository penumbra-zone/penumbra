# penumbra-wasm

The Penumbra core repo has a ton of utilities and functions that are critical to
developing an app that interacts with the Penumbra chain. However, it is written in Rust.
This package exists to bridge the gap between the Rust environment and others (namely web).
This is done via Web Assembly, the universal binary format that runs almost anywhere.

Further, [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/) & [wasm-pack](https://rustwasm.github.io/docs/wasm-pack/) help us compile our Rust code and create an NPM package
that is easily imported for `web`, `nodejs`, and `bundler` environments (javascript/typescript).

## Building Rust crate

```
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

## Testing

The WASM crate incorporates a unit testing suite that leverages `wasm-bindgen-test` to simulate a sample spend transaction.
For testing purposes, the suite mocks `indexDB` database calls in an interactive browser accessible at `http://127.0.0.1:8000`.

```bash
# Install wasm-pack
cargo install wasm-pack

wasm-pack test --chrome -- --test test_build --target wasm32-unknown-unknown --release --features "mock-database"
```

The transaction outputs are accessible in the browser's developer console.

The wasm browser tests are also run headlessly in Penumbra CI. Review the relevant GHA workflow file for specifics.

## Publishing to Npm

The npm org [@penumbra-zone](https://www.npmjs.com/search?q=%40penumbra-zone) governs the public packages.

Upon a tagged release of this repo, `web`, `nodejs`, and `bundler` versions of the wasm packages are published publicly.
See [github action](../../.github/workflows/npm.yml) for more details. Npm secrets are uploaded to github to allow pipeline to publish.

To test compiling wasm packages locally:

```bash
# Install node
# https://nodejs.org/

# Install wasm-pack
cargo install wasm-pack

cd ./publish
npm install
npm run compile-wasm
```

### Manually publishing wasm package to npm

Likely you are wanting to do this to test new versions of crate on a testet-preview version of a web app.

1. Go to [Cargo.toml](Cargo.toml) and change version. Follow naming convention `<next chain version>-pre.<preview version>`.

```diff
- version = "0.64.1"
+ version = "0.65.0-pre.4"
```

2. Push your branch to remote
3. Go to [Github Actions](https://github.com/penumbra-zone/penumbra/actions) and run `wasm-npm-publish` against your branch.
4. Use preview package in your `package.json`
