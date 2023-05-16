# ZK Proofs Parameter Setup

Penumbra's zero-knowledge proofs require circuit-specific parameters to be
generated in a preprocessing phase. There are two
keys generated for each circuit, the *Proving Key* and *Verifying Key* used by the
prover and verifier respectively.

For development purposes *only*, we have a crate in `tools/parameter-setup`
that lets one generate the proving and verifying keys:

```shell
cd tools/parameter-setup
cargo run
```

The verifying and proving keys for each circuit will be created in a serialized
form in the `proof-params/src/gen` folder. Note that the keys will be generated
for all circuits, so you should commit only the keys for the circuits that have
changed.

The proving keys are tracked using Git-LFS. The verifying keys are stored
directly in git since they are small (around ~1 KB each).

## Adding a new Proof

To add a _new_ circuit to the parameter setup, you should modify
`tools/parameter-setup/src/main.rs` before running `cargo run`. 

Then edit `penumbra-proof-params` to reference the new parameters created in
`proof-params/src/gen`.
