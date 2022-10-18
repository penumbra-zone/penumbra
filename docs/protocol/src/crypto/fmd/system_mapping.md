# S-FMD in Penumbra

In the secions that follow we describe how S-FMD clue keys, detection keys, and
clues are integrated into the Penumbra system.

## Clue Keys

Each Penumbra diversified address includes as part of the encoded address an
S-FMD *clue key*. This key can be used to derive a *clue* for a given output.

See the
[Addresses](../../protocol/addresses_keys/addresses.md) section for more details.

## Detection Keys

Each Penumbra address has an associated S-FMD *detection key*. This key is what the
user can optionally disclose to a third-party service for detection. Recall the
detection key is used to examine each clue and return if there is a detection or
not.

## Clues

Each Penumbra transaction can have multiple outputs, to the same or different
recipients. If a transaction contains $n$ outputs for the same recipient,
then the FMD false positive rate $p$ will be $p^{n} << p$ if the detector uses
the detection key that does not correspond to that recipient. To ensure that
transactions are detected with false positive rate $p$, we attach clues to
transactions such that there is a single clue per recipient
clue key per transaction.

In order not to leak the number of distinct recipients to a passive observer
through the number of clues in a transaction, we
add dummy clues to the transaction until there are an equal number of clues and
outputs. A consensus rule verifies that all transactions have an equal number
of clues and outputs.

A consensus rule verifies that clues in transactions have been generated using
the appropriate precision, within a grace period of 10 blocks[^1]. Transactions
with clues generated using the incorrect precision are rejected by the network.

[^1]: This imposes an bound on the latency of signing workflows.
