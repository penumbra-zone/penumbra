# Zero-Knowledge Proofs

## Parameter Setup

Penumbra's zero-knowledge proofs require circuit-specific parameters to be
generated in a preprocessing phase. There are two
keys generated for each circuit, the *Proving Key* and *Verifying Key* - used by the
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

### Adding a new Proof

To add a _new_ circuit to the parameter setup, you should modify
`tools/parameter-setup/src/main.rs` before running `cargo run`. 

Then edit `penumbra-proof-params` to reference the new parameters created in
`proof-params/src/gen`.

## Benchmarks

We have benchmarks for all proofs in the `penumbra-proof-params` crate. You can run them via:

```shell
cd crates/crypto/proof-params
cargo bench
```

Performance as of commit `60edb882587d8ae4d9088f2e6e5a58ddc9980f9a` benchmarked on a 2020 Macbook Pro M1 (8 core CPU) with 8 GB memory:

| Proof    | Number of constraints | Proving time |
| -------- | ------- | ----- |
| Spend  | 33,885    | 2.219s
| Output | 17,892    | 1.101s
| Delegator vote    | 35,978  | 2.236s
| Undelegate claim | 23,670 | 1.364s
| Swap | 37,655 | 2.255s
| SwapClaim | 35,245 | 1.829s
| Nullifier derivation | 953  | 0.126s
