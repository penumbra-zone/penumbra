# Viewing Keys

Each spending key has a corresponding collection of *viewing keys*, which
represent different subsets of capability around creating and viewing
transactions:

* the *full viewing key* represents all capability except for spend
  authorization, including viewing all incoming and outgoing transactions and
  creating proofs;
* the *incoming viewing key* represents the capability to scan the blockchain for incoming transactions;
* the *outgoing viewing key* represents the capability to recover information about sent transactions.

## Full Viewing Keys

The full viewing key consists of two components:

* $\mathsf{ak} \in \mathbb G$, the *spend verification key*, a `decaf377-rdsa` verification key;
* $\mathsf{nk} \in \mathbb F_q$, the *nullifier key*.

Their derivation is described in the previous section.

The spend verification key is used (indirectly) to verify spend authorizations.
Using it directly would allow spends signed by the same key to be linkable to
each other, so instead, each spend is signed by a randomization of the spend
authorization key, and the proof statement includes a proof that the
verification key provided in the spend description is a randomization of the
(secret) spend verification key.

The nullifier key is used to derive the nullifier of a note sent to a particular
user. Nullifiers are used for double-spend protection and so must be bound to
the note contents, but it's important that nullifiers cannot be derived solely
from the contents of a note, so that the sender of a note cannot learn when the
recipient spent it.

The full viewing key can be used to create transaction proofs, as well as to
view all activity related to its account. The hash of a full viewing key is referred
to as an *Account ID* and is derived as follows.

Define
`poseidon_hash_2(label, x1, x2)` to be rate-2 Poseidon hashing of inputs `x1`,
`x2` with the capacity initialized to the domain separator `label`.  Define
`from_le_bytes(bytes)` as the function that interprets its input bytes as an
integer in little-endian order. Define
`element.to_le_bytes()` as the function that encodes an input element in
little-endian byte order. Define `decaf377_s(element)` as the function
that produces the $s$-value used to encode the provided `decaf377` element.
Then

```
hash_output = poseidon_hash_2(from_le_bytes(b"Penumbra_HashFVK"), nk, decaf377_s(ak))
account_group_id = hash_output.to_le_bytes()[0:32]
```

i.e. we take the 32-bytes of the hash output as the account ID.

## Incoming and Outgoing Viewing Keys

The incoming viewing key consists of two components:

* $\mathsf {dk}$, the *diversifier key*, and
* $\mathsf {ivk}$, the *incoming viewing key* (component)[^1].

The diversifier key is used to derive *diversifiers*, which allow a single spend
authority to have multiple, publicly unlinkable *diversified addresses*, as
described in the next section.  The incoming viewing key is used to scan the
chain for transactions sent to every diversified address simultaneously.

The outgoing viewing key consists of one component:

* $\mathsf {ovk}$, the *outgoing viewing key* (component).

These components are derived as follows.

The $\mathsf {dk}$ and $\mathsf {ovk}$ are not intended to be derived in a
circuit.  Define `prf_expand(label, key, input)` as BLAKE2b-512 with
personalization `label`, key `key`, and input `input`.  Define
`to_le_bytes(input)` as the function that encodes an input integer in
little-endian byte order.  Define `decaf377_encode(element)` as the function
that produces the canonical encoding of a `decaf377` element. Then

```
ovk  = prf_expand(b"Penumbra_DeriOVK", to_le_bytes(nk), decaf377_encode(ak))[0..32]
dk = prf_expand(b"Penumbra_DerivDK", to_le_bytes(nk), decaf377_encode(ak))[0..16]
```

The $\mathsf {ivk}$ is intended to be derived in a circuit.  Then
```
ivk = poseidon_hash_2(from_le_bytes(b"penumbra.derive.ivk"), nk, decaf377_s(ak)) mod r
```
i.e., we treat the Poseidon output (an $\mathbb F_q$ element) as an integer, and
reduce it modulo $r$ to get an element of $\mathbb F_r$.

[^1]: In general, we refer to the combination of $\mathsf {dk}$ and $\mathsf
{ivk}$ as the "incoming viewing key" and the component just as $\mathsf {ivk}$,
using the modifier "component" in case of ambiguity.
