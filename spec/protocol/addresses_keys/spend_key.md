# Spending Keys

The root key material for a particular spend authority is a 32-byte random
`seed`.  The `seed` value is used to derive

* $\mathsf{ask} \in \mathbb F_r$, the *spend authorization key*, and
* $\mathsf{nk} \in \mathbb F_q$, the *nullifier key*,

as follows.  Define `prf_expand(label, key, input)` as BLAKE2b-512 with
personalization `label`, key `key`, and input `input`.  Define
`from_le_bytes(bytes)` as the function that interprets its input bytes as an
integer in little-endian order.  Then
```
ask = from_le_bytes(prf_expand("Penumbra_ExpndSd", seed, 0)) mod r
nk  = from_le_bytes(prf_expand("Penumbra_ExpndSd", seed, 0)) mod q
```

The *spending key* consists of `seed` and `ask`.  When using a hardware wallet
or similar custody mechanism, the spending key remains on the device.

The spend authorization key $\mathsf{ask}$ is used as a `decaf377-rdsa` signing
key.[^1] The corresponding verification key is the *spend verification key*
$\mathsf{ak} = [\mathsf{ask}]B$.  The spend verification key $\mathsf{ak}$ and
the nullifier key $\mathsf{nk}$ are used to create the *full viewing key*
described in the next section.

[^1]: Note that it is technically possible for the derived $ask$ or $nsk$ to be
$0$, but this happens with probability approximately $2^{-252}$, so we ignore
this case, as, borrowing phrasing from [Adam Langley][agl_elligator], it happens
significantly less often than malfunctions in the CPU instructions we'd use to
check it.

[agl_elligator]: https://www.imperialviolet.org/2013/12/25/elligator.html
