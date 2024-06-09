# Nullifiers

Each positioned state commitment has an associated _nullifier_, an $\mathbb F_q$
element.  We say that a state commitment is _positioned_ when it has been
included in the state commitment tree and its position is fixed.  In other
words, a nullifier is not just a property of the content of a state fragment,
but of a specific _instance_ of a state fragment included in the chain state.

Nullifiers are not publicly derivable from the state fragment.  Instead, each
state fragment defines an associated _nullifier key_, also an $\mathbb F_q$
element.  The nullifier key represents the capability to derive the nullifier
associated to a state fragment.  When state fragments are created, their state
commitment is revealed on-chain, and when they are consumed, their nullifier is
revealed.  These actions are unlinkable without knowledge of the nullifier key.

## Nullifier Derivation

The nullifier $\mathsf {nf}$ (an $\mathbb F_q$ element) is derived as the
following output of a rate-3 Poseidon hash:

```
nf = hash_3(ds, (nk, cm, pos))
```

where the `ds` is a domain separator as described below, `nk` is the
nullifier key, `cm` is the state commitment, and `pos` is the position
of the state commitment in the state commitment tree.

Define `from_le_bytes(bytes)` as the function that interprets its input bytes as
an integer in little-endian order. The domain separator `ds` used for nullifier
derivation is computed as:

```
ds = from_le_bytes(BLAKE2b-512(b"penumbra.nullifier")) mod q
```

## Nullifier Keys

Nullifiers are derived only semi-generically. While the nullifier derivation
itself is generic, depending only on the nullifier key and the positioned
commitment, each type of state fragment is responsible for defining how the
nullifier key is linked to the content of that state fragment.  Because
nullifiers prevent double-spends, this linking is critical for security.

This section describes the abstract properties required for nullifier key
linking, and how each type of state fragment achieves those properties.

- **Uniqueness**: each state fragment must have at most one nullifier key.

Uniqueness ensures that each state fragment can be nullified at most once.  A
state fragment without a nullifier key is unspendable; this is likely
undesirable, like sending funds to a burn address, but is not prohibited by the
system.  If a state fragment had more than one nullifier key, it could be
consumed multiple times, causing a double-spend vulnerability.

Each action that reveals nullifiers is responsible for certifying two distinct
properties as part of its proof statements:

- Correct derivation of the nullifier given the nullifier key and positioned state commitment (generic across state fragment types);
- Correct linking of that nullifier key with the content of the nullified state fragment (specific to that state fragment type).

## State Fragments

These are the current types of state fragment recorded by Penumbra:

### Notes

The nullifier key for a note is the nullifier key component of the full viewing
key for the address controlling the note.

**Uniqueness**: Nullifiers are unique, to prevent faerie gold attacks, as they are derived from the position in the state commitment tree which is unique. 

### Swaps

The nullifier key for a swap is the nullifier key component of the full viewing key for the claim address that controls the swap outputs.

**Uniqueness**: TODO
