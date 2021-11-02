# Addresses

Addresses in Penumbra are diversified payment addresses as in Zcash Sapling but with an additional key for [FMD](../../crypto/fmd.md) support. For each *spend key*, there are many possible diversified payment addresses, all of which share viewing keys. Each address consists of three components:

* a *diversifier* $d$, an 11 byte random number
* a *transmission key* $pk_d$, a point on the `decaf377` curve
* a *compact flag key* $cfk_d$, a point on the `decaf377` curve

The diversifier $d$ is an 11 byte random number. From $d$ we compute a *diversified basepoint* $B_d$:

$B_d = GH(d)$

The $GH$ function is [Group Hash](../../crypto/decaf377/group_hash.md) which produces a point on the `decaf377` curve that we use as a generator. Every diversifier will produce a valid generator.

Next we derive the *transmission key* $pk_d$ by multiplying the diversified basepoint by the incoming viewing key $ivk$:

$pk_d = [ivk]B_d$

The *compact flag key* is derived from a (diversified) *compact detection key* $cdtk_d$ using a non-diversified `decaf377` basepoint [$B$](../primitives/decaf377/test_vectors.md):

$cfk_d = [cdtk_d]B$

The diversifier, transmission key, and compact flag key are [Bech32m](https://github.com/bitcoin/bips/blob/master/bip-0350.mediawiki) encoded with a human readable prefix that is:

* `penumbra` for mainnet, and
* `penumbra_tn00X_` for testnets, where X is the current testnet number.
