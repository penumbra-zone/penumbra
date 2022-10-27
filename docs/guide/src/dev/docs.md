# Building documentation

The [protocol docs] and the [guide] (this document) are built using
[mdBook] and auto-deployed on pushes to `main`.  To build locally:

1. Install the requirements: `cargo install mdbook mdbook-katex mdbook-mermaid`
2. Run `mdbook serve` from `docs/protocol` (for the protocol spec) or from `docs/guide` (for this document).

The Rust API docs can be built with `cargo doc`.

[protocol docs]: https://protocol.penumbra.zone
[rustdoc]: https://rustdoc.penumbra.zone
[guide]: https://guide.penumbra.zone
[mdBook]: https://rust-lang.github.io/mdBook/
