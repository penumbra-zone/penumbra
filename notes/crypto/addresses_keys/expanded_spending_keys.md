# Expanded Spending Keys

The *expanded spending key* has three components:

* $ask$, the *spend authorization key* which is a scalar value
* $nsk$, the *nullifier private key* which is a scalar value
* $ovk$, the *outgoing viewing key* which is a 32 byte number

The scalars are derived by hashing $sk$ along with a value $t$ ($t=0$ for $ask$, $t=1$ for $nsk$, $t=2$ for $ovk$), then mapping to a scalar for the decaf377 curve.

TODO: Define $ToScalar$ function for decaf377
TODO: Confirm $PRF^{expand}_{sk}$ is unchanged from Zcash sapling (using Blake2b)

TK: FMD flag key goes in here derived from $sk$?
