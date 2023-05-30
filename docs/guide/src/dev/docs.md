# Building documentation

The [protocol docs] and the [guide] (this document) are built using
[mdBook] and auto-deployed on pushes to `main`.  To build locally:

1. Install the requirements: `cargo install mdbook mdbook-katex mdbook-mermaid`
2. Run `mdbook serve` from `docs/protocol` (for the protocol spec) or from `docs/guide` (for this document).

The [Rust API docs][rustdoc] can be built with `./deployments/scripts/rust-docs`.
The landing page, the top-level `index.html`, is handled as a special case.
If you added new crates by appending a `-p <crate_name>` to the `rust-docs` script,
then you must rebuild the index page via:

You'll need to use the `nightly` toolchain for Rust to build the docs. In some cases, you'll need
a specific version. To configure locally:

```
rustup toolchain install nightly-2023-05-15
```

CI will automatically rebuild all our docs on merges into main.

[protocol docs]: https://protocol.penumbra.zone
[rustdoc]: https://rustdoc.penumbra.zone
[guide]: https://guide.penumbra.zone
[mdBook]: https://rust-lang.github.io/mdBook/
