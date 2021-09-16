# Expanded Spending Keys

The *expanded spending key* has three components derived from the spending key *sk*:

These components are:

* $ask$, the *spend authorization key* which is a scalar value
* $nsk$, the *nullifier private key* which is a scalar value
* $ovk$, the *outgoing viewing key* which is a 32 byte number

The scalars are derived by hashing $sk$ along with a modifier value using `blake2b_512`, then mapping to a scalar for the decaf377 curve (here called `to_scalar`). In pseudocode this becomes:

```
ask = to_scalar( BLAKE2b_512("Penumbra_ExpandSeed", encode(sk) || 0 ) )
nsk = to_scalar( BLAKE2b_512("Penumbra_ExpandSeed", encode(sk) || 1 ) )
```

If $ask$ is 0, then $sk$ and all derived key material is discarded and a new $sk$ is generated.

The outgoing viewing key is derived by taking the first 32 bytes of the hash:

```
ovk = truncate_32_bytes( BLAKE2b_512("Penumbra_ExpandSeed", encode(sk) || 2 )
```

## TODOs

- [ ] Define encoding of sk, LEBS2OSP in Zcash spec
- [ ] Define $to_scalar$ function.
- [ ] Replace `blake2b_512` with `poseidon` - followup issue, requires #55
