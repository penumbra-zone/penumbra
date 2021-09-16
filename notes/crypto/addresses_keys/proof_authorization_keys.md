
# Proof Authorization Keys

The *proof authorization key* has two components:

* $ak$, the *authorization key*, a point on the `decaf377` curve, derived from multiplying $ask$ (from the expanded spending key) by a fixed generator point on `decaf377`
* $nsk$, the *nullifier private key*, as defined in the [*expanded spending key*](./expanded_spending_keys.md) section,

The authorization key is derived by multiplying $ask$ by a generator point on the `decaf377` curve (this is analogous to `SpendAuthSig.DerivePublic` in the Zcash specification):

$ak = G_{a} * ask$

To spend notes, one must prove knowledge of $ak$, $nsk$, and $ask$.

## TODOs

- [ ] Define $G_{a}$ as this point: https://github.com/penumbra-zone/decaf377/blob/main/ristretto.sage#L689
