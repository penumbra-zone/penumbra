# Nullifiers

- [ ] What is a nullifier?
- [ ] Refactor to describe generic nullifiers

The nullifier $\mathsf {nf}$ (an $\mathbb F_q$ element) is derived as the following output of a rate-3 Poseidon hash:

```
nf = hash_3(ds, (nk, cm, pos))
```

where the `ds` is a domain separator as described below, $nk$ is the nullifier-deriving key, `cm` is the state commitment, and `pos` is the position of the state commitment in the state commitment tree.

Define
`from_le_bytes(bytes)` as the function that interprets its input bytes as an
integer in little-endian order. The domain separator `ds` used for nullifier derivation is computed as:

```
ds = from_le_bytes(BLAKE2b-512(b"penumbra.nullifier")) mod q
```
