
# Proof Authorization Keys

The *proof authorization key* has two components:

* $ak$, the *authorization key*, a point on the `decaf377` curve,
* $nsk$, the *nullifier private key*, as defined in the [*expanded spending key*](./expanded_spending_keys.md) section,

The authorization key is derived by multiplying the scalar $ask$ (from the expanded spending key) by a fixed generator point [$B$](../primitives/decaf377/test_vectors.md) on the `decaf377` curve (this is analogous to `SpendAuthSig.DerivePublic` in the Zcash specification):

$ak = [ask]B$

To spend notes, one will prove knowledge of $ak$, $nsk$, and $ask$.
