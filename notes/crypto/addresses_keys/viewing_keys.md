# Viewing Keys

A *full viewing key* enables one to identify incoming and outgoing notes only. It consists of three components:

* $ak$, the *authorization key*, a point on the `decaf377` curve, derived from multiplying $ask$ by a fixed generator point on `decaf377`
* $nk$, the *nullifier deriving key*, used for deriving nullifiers for notes, derived from multiplying $nsk$ by a fixed generator point on `decaf377`
* $ovk$, the *outgoing viewing key*, defined as above in `Expanded Spending Keys`

An *incoming viewing key* $ivk$ is derived from hashing $ak$ and $nk$.

TODO: Confirm $CRH^{ivk}$ unchanged from Sapling