# Viewing Keys

A *full viewing key* enables one to identify incoming and outgoing notes only. It consists of three components:

* $ak$, the *authorization key*, a point on the `decaf377` curve, derived as described in the [*proof authorization keys*](./proof_authorization_keys.md) section,
* $nk$, the *nullifier deriving key*, used for deriving nullifiers for notes, derived from multiplying $nsk$ by a fixed generator point on `decaf377`,
* $ovk$, the *outgoing viewing key*, derived as described in the [*expanded spending keys*](./expanded_spending_keys.md) section,

The nullifier deriving key is generating by multiplying the scalar $nsk$ (from the spending key) by a fixed generator point [$B$](../primitives/decaf377/test_vectors.md) on the `decaf377` curve:

$nk = [nsk]B$

The *incoming viewing key* $ivk$ is derived from hashing together $ak$ and $nk$. 
