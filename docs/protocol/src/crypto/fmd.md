# Fuzzy Message Detection

By design, privacy-preserving blockchains like Penumbra don't reveal metadata
about the sender or receiver of a transaction.  However, this means that users
must scan the entire chain to determine which transactions relate to their
addresses.  This imposes large bandwidth and latency costs on users who do not
maintain online replicas of the chain state, as they must "catch up" each time
they come online by scanning all transactions that have occurred since their
last activity.

Alternatively, users could delegate scanning to a third party, who would monitor
updates to the chain state on their behalf and forward only a subset of those
updates to the user.  This is possible using *viewing keys*, as in Zcash, but
viewing keys represent the capability to view all activity related to a
particular address, so they can only be delegated to trusted third parties.

Instead, it would be useful to be able to delegate only a probabilistic
*detection* capability.  Analogous to a Bloom filter, this would allow a
detector to identify all transactions related to a particular address (no false
negatives), while also identifying unrelated transactions with some false
positive probability.  Unlike viewing capability, detection capability would not
include the ability to view the details of a transaction, only a probabilistic
association with a particular address.  This is the problem of *fuzzy message
detection* (FMD), analyzed by Beck, Len, Miers, and Green in their paper [Fuzzy
Message Detection][fmd-paper], which proposes a cryptographic definition of
fuzzy message detection and three potential constructions.

This section explores how Penumbra could make use of fuzzy message detection:

* In [Sender and Receiver FMD](./fmd/sender-receiver.md), we propose a
generalization of the original definition where the false positive probability
is set by the sender instead of the receiver, and discusses why this is useful.

* In [Constructing S-FMD](./fmd/construction.md), we realize the new definition
using a variant of one of the original FMD constructions, and extend it in two ways:
    1. to support arbitrarily precise detection with compact, constant-size keys;
    2. to support *diversified detection*, allowing multiple, publicly
    unlinkable addresses to be scanned by a single detection key.

Unfortunately, these extensions are not mutually compatible, so we only use the
first one, and record the second for posterity.

* In [S-FMD Threat Model](./fmd/threat_model.md), we describe the threat model for S-FMD on Penumbra. 
* In [Parameter Considerations](./fmd/considerations.md), we discuss how the
false positive rates should be chosen.

## Acknowledgements

Thanks to George Tankersley and Sarah Jamie Lewis for discussions on this topic
(and each independently suggesting the modifications to realize S-FMD), to
Gabrielle Beck for discussions about the paper and ideas about statistical
attacks, and to Guillermo Angeris for pointers on analyzing information disclosure.

[fmd-paper]: https://eprint.iacr.org/2021/089
