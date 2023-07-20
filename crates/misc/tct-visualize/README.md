# Visualizing the Tiered Commitment Tree

Penumbra's [Tiered Commitment Tree](https://rustdoc.penumbra.zone/main/penumbra_tct/index.html)
is a merkle quadtree used to store cryptographic commitments to shielded state. This crate provides
interactive visualizations for understanding how the tree works. To see a talk using these visualizations
to demonstrate the functionality of the tree, see: https://www.youtube.com/watch?v=mHoe7lQMcxU.

## Interactive Visualization

To run the interactive visualization in your browser, run:

```bash
cargo run --release --bin tct-live-edit
```

Then visit the page http://0.0.0.0:8080 in your browser (the animations are smoothest and fastest in Chromium
based browsers).

Various options can be configured; for details, see:

```bash
cargo run --release --bin tct-live-edit -- --help
```

## Randomized Step-By-Step Non-Interactive Visualization

You can also simulate a sequence of operations on the tree and render each step as an SVG using:

```bash
cargo run --release --bin tct-visualize
```

See its `--help` options for details about how to configure the simulation.
