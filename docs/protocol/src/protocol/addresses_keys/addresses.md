# Addresses and Detection Keys

Rather than having a single address for each spending authority, Penumbra allows
the creation of many different publicly unlinkable *diversified addresses*.  An
incoming viewing key can scan transactions to every diversified address
simultaneously, so there is no per-address scanning cost.  In addition, Penumbra
attaches a *detection key* to each address, allowing a user to outsource
probabilistic transaction detection to a relatively untrusted third-party
scanning service.

## Diversifiers

Addresses are parameterized by *diversifiers*, 16-byte tags used to derive up to
$2^{128}$ distinct addresses for each spending authority.  The diversifier is
included in the address, so it should be uniformly random.  To ensure this,
diversifiers are indexed by a *address index* $i \in \{0, \ldots, 2^{128} -
1\}$; the $i$-th diversifier $d_i$ is the encryption of $i$ using [AES] with
the diversifier key $\mathsf{dk}$.[^1]

Each diversifier $d$ is used to generate a *diversified basepoint* $B_d$ as
$$B_d = H_{\mathbb G}^{\mathsf d}(d),$$
where 
$$H_{\mathbb G}^{\mathsf d} : \{0, 1\}^{128} \rightarrow \mathbb G$$
performs [hash-to-group] for `decaf377` as follows: first, apply BLAKE2b-512
with personalization `b"Penumbra_Divrsfy"` to the input, then, interpret the
64-byte output as an integer in little-endian byte order and reduce it modulo
$q$, and finally, use the resulting $\mathbb F_q$ element as input to the
`decaf377` CDH map-to-group method.

## Detection Keys

Each address has an associated *detection key*, allowing the creator of the
address to delegate a [probabilistic detection capability][fmd] to a third-party
scanning service.

The detection key consists of one component,

* $\mathsf{dtk_d}$, the detection key (component)[^2],

derived as follows.  Define `prf_expand(label, key, input)` as BLAKE2b-512 with
personalization `label`, key `key`, and input `input`.  Define
`from_le_bytes(bytes)` as the function that interprets its input bytes as an
integer in little-endian order, and `to_le_bytes` as the function that encodes
an integer to little-endian bytes.  Then
```
dtk_d = from_le_bytes(prf_expand(b"PenumbraExpndFMD", to_le_bytes(ivk), d))
```

## Addresses

Each payment address has three components:

* the *diversifier* $d$;
* the *transmission key* $\mathsf{pk_d}$, a `decaf377` element;
* the *clue key* $\mathsf{ck_d}$, a `decaf377` element.

The diversifier is derived from a address index as described above.  The
diversifier $d_0$ with index $0$ is the *default diversifier*, and corresponds
to the *default payment address*.

The transmission key $\mathsf{pk_d}$ is derived as $\mathsf{pk_d} =
[\mathsf{ivk}]B_d$, where $B_d = H_{\mathbb G}^{\mathsf d}(d)$ is the
diversified basepoint.

The clue key is $\mathsf{ck_d}$ is derived as $\mathsf{ck_d} =
[\mathsf{dtk_d}]B$, where $B$ is the conventional `decaf377` basepoint.

### Address Encodings

The raw binary encoding of a payment address is the 80-byte string `d || pk_d ||
ck_d`.  We then apply the [F4Jumble] algorithm to
this string. This mitigates attacks where an attacker replaces a valid
address with one derived from an attacker controlled key that encodes to an
address with a subset of characters that collide with the target valid address.
For example, an attacker may try to generate an address with the first
$n$ characters matching the target address. See [ZIP316] for more on this
scenario as well as [F4Jumble], which is a 4-round Feistel construction.

This jumbled string is then encoded with [Bech32m] with the following
human-readable prefixes:

* `penumbra` for mainnet, and
* `penumbra_tnXYZ_` for testnets, where XYZ is the current testnet number padded
  to three decimal places.

[^1]: This convention is not enforced by the protocol; client software could in
principle construct diversifiers in another way, although deviating from this
mechanism risks compromising privacy.

[^2]: As in the previous section, we use the modifier "component" to distinguish
between the internal key component and the external, opaque key.

[AES]: https://docs.rs/aes/latest/aes/
[Bech32m]: https://github.com/bitcoin/bips/blob/master/bip-0350.mediawiki
[hash-to-group]: ../../crypto/decaf377/group_hash.md
[F4Jumble]: https://zips.z.cash/zip-0316#jumbling
[fmd]: ../../crypto/fmd.md
[ZIP316]: https://zips.z.cash/zip-0316
