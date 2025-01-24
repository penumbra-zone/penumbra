# Addresses and Detection Keys

Rather than having a single address for each spending authority, Penumbra allows
the creation of many different publicly unlinkable *diversified addresses*.  An
incoming viewing key can scan transactions to every diversified address
simultaneously, so there is no per-address scanning cost.  In addition, Penumbra
attaches a *detection key* to each address, allowing a user to outsource
probabilistic transaction detection to a relatively untrusted third-party
scanning service.

## Privacy Implications

While diversified addresses are described as *publicly unlinkable*, a detection entity given multiple detection keys can empirically link the corresponding diversified addresses via the clue keys $\mathsf{ck_d}$ contained within them. This is because the detection entity, by reporting detected transactions to the same user, empirically knows that the detection keys $\mathsf{dtk_d}$ are linked. If the detection entity observes two addresses belonging to the user, they can link them because the clue key appears in each address and is derived solely from the detection key. 

In a simplified scenario, a user with diversified addresses ${addr_1}$ and ${addr_2}$ gives the associated detection keys ${dtk_{d_1}}$ and ${dtk_{d_2}}$ to the detection entity. The detection entity detects relevant transactions using the clue keys ${ck_{d_1}}$ and ${ck_{d_2}}$, and reports the detected transactions for ${dtk_{d_1}}$ and ${dtk_{d_2}}$ back to the user. The detection entity can naively observe that transactions related to ${ck_{d_1}}$ and ${ck_{d_2}}$ are reported back to the same user, and therefore the addresses linked to these detection keys belong to the same user. There's a notion of linkability here since the diversified addresses can be linked by the detection entity through the detection keys, from which the clue keys are derived. 

To mitigate the linkability of diversified addresses when using detection keys, a user should consider using multiple third parties: distribute detection keys to different detection entities instead of a single one, reducing the risk that any single entity has enough keys to link diversified addresses.

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
performs [hash-to-group](../crypto/decaf377/group_hash.md) for `decaf377` as follows: first, apply BLAKE2b-512
with personalization `b"Penumbra_Divrsfy"` to the input, then, interpret the
64-byte output as an integer in little-endian byte order and reduce it modulo
$q$, and finally, use the resulting $\mathbb F_q$ element as input to the
`decaf377` CDH map-to-group method.

## Detection Keys

Each address has an associated *detection key*, allowing the creator of the
address to delegate a [probabilistic detection capability](../crypto/fmd.md) to a third-party
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

The diversifier is derived from an address index as described above.  The
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

### Short Address Form

Addresses can be displayed in a short form. A short form of length $120$ bits (excludes the human-readable
prefix) is recommended to mitigate address replacement attacks. The attacker's goal is to find a partially
colliding prefix for any of $2^N$ addresses they have targeted. 

#### Untargeted attack

In an untargeted attack, the attacker wants to find two different addresses that have a colliding short form of length $M$ bits.

A collision is found in $\sqrt(2^M)$ steps due to the birthday bound, where $M$ is the number of bits of the prefix.

Thus we'd need to double the length $2M$ to provide a desired security level of $M$ bits.

This is equivalent to $2M/5$ characters of the Bech32m address, excluding the human-readable prefix. Thus for a targeted security
level of 80 bits, we'd need a prefix length of 160 bits, which corresponds to 32 characters of the Bech32m address, excluding the
human-readable prefix.

The short form is not intended to mitigate this attack.

#### Single-target attack

In a targeted attack, the attacker's goal is to find one address that collides with a target prefix of length $M$ bits.

Here the birthday bound does not apply. To find a colliding prefix of the first M bits, they need to search $2^M$ addresses.

The short form is intended to mitigate this attack.

#### Multi-target attack

In a multi-target attack, the attacker's goal is to be able to generate one address that collides with the short form of 1 of $2^N$ different addresses.

They are searching for a collision between the following two sets:
* set of the short forms of the targeted addresses, which has size $2^N$ elements, and each element has length $2^M$ bits.
* set of the short forms of all addresses, which has size $2^M$ elements.

If the attacker has a target set of $2^{N}$ addresses, she can find a collision
after $2^{M - N}$ steps. Thus for a prefix length of $M$ bits, we get $M-N$ bits
of security. For a targeted security level of 80 bits and a target set of size $2^{40}$,
we need a prefix length of 120 bits, which corresponds to 24 characters of the Bech32m
address, excluding the human-readable prefix and separator.

The short form is intended to mitigate this attack.

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
