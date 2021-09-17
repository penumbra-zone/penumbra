# Primitives

Penumbra uses the following cryptographic primitives, described in the following sections:

- The [Proof System](./primitives/proofs.md) section describes the choice of proving curve (BLS12-377) and proof system (Groth16, and potentially PLONK in the future);

- The [`decaf377`](./primitives/decaf377.md) section describes `decaf377`, a parameterization of the [Decaf] construction defined over the BLS12-377 scalar field, providing a prime-order group that can be used inside or outside of a circuit;

- The [Poseidon for BLS12-377](./primitives/poseidon.md) section describes parameter selection for an instantiation of [Poseidon], a SNARK-friendly sponge construction, over the BLS12-377 scalar field;

- The [`zk555`](./primitives/strobe.md) section describes `zk555`, an instantiation of the [STROBE protocol framework][strobe] for use inside (and outside) of circuits.

[Decaf]: https://www.shiftleft.org/papers/decaf/
[Poseidon]: https://www.poseidon-hash.info/home
[strobe]: https://strobe.sourceforge.io/