# Zero-Knowledge Proofs

## Test Parameter Setup

Penumbra's zero-knowledge proofs require circuit-specific parameters to be
generated in a preprocessing phase. There are two
keys generated for each circuit, the *Proving Key* and *Verifying Key* - used by the
prover and verifier respectively.

For development purposes *only*, we have a crate in `tools/parameter-setup`
that lets one generate the proving and verifying keys:

```shell
cargo run --release --bin penumbra-parameter-setup
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

## Circuit Benchmarks

We have benchmarks for all proofs in the `penumbra-bench` crate. You can run them via:

```shell
cargo bench
```

Performance as of commit `ce2d319bd5534fd28600227b28506e32b8504493` benchmarked on a 2023 Macbook Pro M2 (12 core CPU) with 32 GB memory and the `parallel` feature enabled:

| Proof    | Number of constraints | Proving time |
| -------- | ------- | ----- |
| Spend  | 34,630    | 388ms
| Output | 13,875    | 130ms
| Delegator vote    | 36,723  | 389ms
| Undelegate claim | 14,776 | 139ms
| Swap | 25,700 | 237ms
| SwapClaim | 37,061 | 379ms
| Nullifier derivation | 394  | 15ms

## zk-SNARK Ceremony Benchmarks

Run benchmarks for the zk-SNARK ceremony via:

```shell
cd crates/crypto/proof-setup
cargo bench
```

Performance as of commit `1ed963657c16e49c65a8e9ecf998d57fcce8f200` benchmarked on a 2023 Macbook Pro M2 (12 core CPU) with 32 GB memory using 37,061 constraints (SwapClaim circuit) (note that in practice the performance will be based on the next power of two, for the most part):

| Task             |    Time |
| ---------------- | ------- |
| Phase 1 run      |  71.58s |
| Phase 1 check    | 147.41s |
| Phase transition | 131.72s |
| Phase 2 run      |  14.76s |
| Phase 2 check    |   0.21s |
