# Undelegation

The undelegation process unbonds stake from a validator, converting delegation
tokens `dPEN` to stake `PEN`. Undelegations may be performed in any block, but
only settle after the undelegation has exited the unbonding queue.

The unbonding queue is a FIFO queue allowing only a limited amount of stake to
be unbonded in each epoch, according to an unbonding rate selected by
governance. Undelegations are inserted into the unbonding queue in FIFO order.
Unlike delegations, where only the total amount of newly bonded stake is
revealed, undelegations reveal the precise amount of newly unbonded stake,
allowing the unbonding queue to function.

Undelegations are accomplished by creating a transaction with a
`Undelegate` description. This description has different behaviour
depending on whether or not the validator was slashed.

In the unslashed case, the undelegate description spends a note with value
$y$ `dPEN`, reveals $y$, and produces $y \psi_v(e)$ `PEN` for the transaction's
balance, where $e$ is the index of the current epoch.  However, the nullifiers
revealed by undelegate descriptions are not immediately included in the
nullifier set, and new notes created by a transaction containing an undelegate
description are not immediately included in the state commitment tree. Instead,
the transaction is placed into the unbonding queue to be applied later. In the
first block of each epoch, transactions are applied if the corresponding
validator remains unslashed, until the unbonding limit is reached.

If a validator is slashed, any undelegate transactions currently in the
unbonding queue are discarded. Because the nullifiers for the notes those
transactions spent were not included in the nullifier set, the notes remain
spendable, allowing a user to create a new undelegation description.

Undelegations from a slashed validator are settled immediately. The
undelegate description spends a note with value $y$ `dPEN` and produces
$sy \psi_v(e_s)$ `PEN`, where $1-s$ is the slashing penalty and
$e_s$ is the epoch at which the validator was slashed. The remaining value,
$(1-s)y\psi_v(e_s)$, is burned.

Because pending undelegations from a slashed validator are discarded without
applying their nullifiers, those notes can be spent again in a post-slashing
undelegation description. This causes linkability between the discarded
undelegations and the post-slashing undelegations, but this is not a concern
because slashing is a rare and unplanned event which already imposes worse
losses on delegators.
