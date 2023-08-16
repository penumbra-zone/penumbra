# Zero-Knowledge Proofs

## Test Parameter Setup

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

## Circuit Benchmarks

We have benchmarks for all proofs in the `penumbra-proof-params` crate. You can run them via:

```shell
cd crates/crypto/proof-params
cargo bench
```

Performance as of commit `98590ef2dd92ea75fa3bd6f09b3c24fec3fe36ff` benchmarked on a 2023 Macbook Pro M2 (12 core CPU) with 32 GB memory:

| Proof    | Number of constraints | Proving time |
| -------- | ------- | ----- |
| Spend  | 34,630    | 2.06s
| Output | 13,875    | 667ms
| Delegator vote    | 36,723  | 2.05s
| Undelegate claim | 14,776 | 722ms
| Swap | 25,700 | 1.21s
| SwapClaim | 37,061 | 1.83s
| Nullifier derivation | 394  | 59ms

## zk-SNARK Ceremony Benchmarks

Run benchmarks for the zk-SNARK ceremony via:

```shell
cd crates/crypto/proof-setup
cargo bench
```

Performance as of commit `eb5c4a26b925ae7c8fd3e3d6c3eb28f8fc49ddee` benchmarked on a 2023 Macbook Pro M2 (12 core CPU) with 32 GB memory using 37,061 constraints (SwapClaim circuit):

| Task    | Time |
| -------- | ------- |
| Phase 1 prove  | 40.3s    |
| Phase 1 verify | 84.2s    |
