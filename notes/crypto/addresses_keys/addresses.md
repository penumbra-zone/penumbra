# Addresses

Addresses in Penumbra are diversified payment addresses as in Zcash Sapling. This means that for each *spending key*, there are many possible diversified payment addresses. Each address consists of:

* a *diversifier* $d$, an 11 byte random number
* a *transmission key* $pk_d$, a point on the `decaf377` curve
* a *compact flag key* $cfk_d$, a point on the `decaf377` curve

The *compact flag key* is derived from a *compact detection key* $ctk_d$. 
