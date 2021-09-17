# Expanded Spending Keys

The *expanded spending key* has three components derived from the spending key *sk*:

* $ask$, the *spend authorization key*, a scalar value
* $nsk$, the *nullifier private key*, a scalar value
* $ovk$, the *outgoing viewing key*, a 32 byte number

The scalars are derived by hashing $sk$ along with a modifier value using `BLAKE2b_512`, then mapping to a scalar for the decaf377 curve (here called `to_scalar`). In pseudocode this becomes:

```
ask = to_scalar( BLAKE2b_512("Penumbra_ExpandSeed", sk || 0 ) )
nsk = to_scalar( BLAKE2b_512("Penumbra_ExpandSeed", sk || 1 ) )
```

where the function `to_scalar` returns an integer in little-endian order, modulo the order of the `decaf377` subgroup.

If $ask$ or $nsk$ is 0, then $sk$ and all derived key material is discarded and a new $sk$ is generated.

The outgoing viewing key is derived by taking the first 32 bytes of the hash of $sk$ and its modifier:

```
ovk = truncate_32_bytes( BLAKE2b_512("Penumbra_ExpandSeed", sk || 2 )
```
