# Building documentation

The [protocol spec][protocol] and the guide (this document) is built using
[mdBook] and auto-deployed on pushes to `main`.  To build it locally:

1. Install the requirements: `cargo install mdbook mdbook-katex mdbook-mermaid`
2. Run `mdbook serve` from `docs/protocol` (for the protocol spec) or from `docs/guide` (for this document).

The Rust API docs can be built with `cargo doc`.
