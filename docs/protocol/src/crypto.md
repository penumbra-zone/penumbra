# Primitives

Penumbra uses the following cryptographic primitives, described in the following sections:

- The [Proof System](./crypto/proofs.md) section describes the choice of proving
curve (BLS12-377) and proof system (Groth16, and potentially PLONK in the
future).

- The [`decaf377`](./crypto/decaf377.md) section describes `decaf377`, a
parameterization of the [Decaf] construction defined over the BLS12-377 scalar
field, providing a prime-order group that can be used inside or outside of a
circuit.

- The [Poseidon for BLS12-377](./crypto/poseidon.md) section describes parameter
selection for an instantiation of [Poseidon], a SNARK-friendly sponge
construction, over the BLS12-377 scalar field.

- The [Tiered Commitment Tree](./crypto/tct.md) section describes a sparse
incremental three-tiered Merkle quad-tree which uses the above
[Poseidon for BLS12-377](./crypto/poseidon.md) construction as its hash function,
allowing zero knowledge proofs of inclusion in the tree.

- The [Fuzzy Message Detection](./crypto/fmd.md) section describes a
construction that allows users to outsource a probabalistic detection
capability, allowing a third party to scan and filter the chain on their behalf,
without revealing precisely which transactions are theirs.

- The [Homomorphic Threshold Decryption](./crypto/threshold.md) section
describes the construction used to [batch flows of
value](./concepts/batching_flows.md) across transactions.

- The [Randomizable Signatures](./crypto/decaf377-rdsa.md) section describes
`decaf377-rdsa`, a variant of the Zcash RedDSA construction instantiated over
`decaf377`, used for binding and spend authorization signatures.

- The [Key Agreement](./crypto/decaf377-ka.md) section describes `decaf377-ka`,
an instantiation of Diffie-Hellman key agreement over `decaf377`.




[Decaf]: https://www.shiftleft.org/papers/decaf/
[Poseidon]: https://www.poseidon-hash.info/home
[strobe]: https://strobe.sourceforge.io/
