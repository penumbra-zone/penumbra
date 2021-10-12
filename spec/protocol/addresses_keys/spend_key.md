# Spend Keys

The *spend key* has three components derived from the spend seed $ss$:

* $ask$, the *spend authorization key*, a scalar value
* $nsk$, the *nullifier private key*, a scalar value
* $ovk$, the *outgoing viewing key*, a 32 byte number

The scalars are derived by using the spend seed as the key for a PRF,
expanding different labels, and converting the output to a `decaf377` scalar.
In pseudocode this becomes:

```
prf_expand(x) = BLAKE2b_512("Penumbra_ExpandSeed", ss || x )
ask = to_scalar(prf_expand(0))
ask = to_scalar(prf_expand(1))
```
where the function `to_scalar` interprets the input byte sequence as an integer
in little-endian order and reduces it modulo the order of the `decaf377` group[^1].

The outgoing viewing key is derived by truncating the PRF output:
```
ovk = truncate_32_bytes(prf_expand(2))
```

[^1]: Note that it is technically possible for the derived $ask$ or $nsk$ to be
$0$, but this happens with probability approximately $2^{-252}$, so we ignore
this case, as, borrowing phrasing from [Adam Langley][agl_elligator], it happens
significantly less often than malfunctions in the CPU instructions we'd use to
check it.

[agl_elligator]: https://www.imperialviolet.org/2013/12/25/elligator.html