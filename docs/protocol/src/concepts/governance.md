# Governance

Penumbra supports on-chain governance with delegated voting.  Validators' votes
are public and act as default votes for their entire delegation pool, while
delegators' votes are private, and override the default vote provided by their
validator.

Votes are the same as on the Cosmos Hub: `Yes`, `No`, `NoWithVeto`, and
`Abstain`. `NoWithVeto` is the same as `No` but also votes that the proposer
should lose their deposit. The intended cultural norm is that `No` should be
used to indicate disagreement with good-faith proposals and `NoWithVeto`
should be used to deter spam proposals.

## Proposals

Penumbra users can propose votes by escrowing a minimum amount of `PEN`.  They
do this by creating a transaction with a `CreateProposal` description, which
consumes some amount of `PEN` from the transaction's balance, and creates a new
escrow note with the same amount.  The note is escrowed in the sense that it is
recorded seperately and is not included in the state commitment tree until voting
completes. 

Proposals can either be normal or emergency proposals.  In either case, the
voting period begins immediately, in the next block after the proposal has been
committed to the chain.  Normal proposals have a fixed-length voting period,
while emergency proposals are accepted as soon as a 2/3 majority of the stake is
reached.

Because validators provide default votes for their delegation pool, an emergency
proposal can in principle be accepted immediately, without any input from
delegators. This allows time-critical resolution of emergencies (e.g., deploying
an 0day hotfix); the 2/3 majority of the stake required is already sufficient to
arbitrarily rewrite the chain state.

Proposals can also be withdrawn by their proposer prior to the end of the voting
period.  This is done by creating a transaction with a `WithdrawProposal`
description, and allows the community to iterate on proposals as the (social)
governance process occurs.  For instance, a chain upgrade proposal can be
withdrawn and re-proposed with a different source hash if a bug is discovered
while upgrade voting is underway.  Withdrawn proposals cannot be accepted, even
if the vote would have passed, but they can be vetoed.[^1]

## Voting

Stakeholder votes are of the form $(x_y, x_n, x_a, x_v)$, representing the
weights for yes, no, abstain, and veto respectively.  Most stakeholders would
presumably set all but one weight to $0$.  Stakeholders vote by proving
ownership of some amount of bonded stake (their voting power) prior to the
beginning of the voting period.

To do this, they create a transaction with a `Vote` description.  This
description identifies the validator $v$ and the proposal, proves spend
authority over a note recording $y$ `dPEN(v)`, and reveals the note's nullifier.
Finally, it proves vote consistency $y = x_y + x_n + x_a + x_v$, produces a new
note with $y$ `dPEN(v)`, and includes $\operatorname{Enc}_D(x_i)$, an encryption
of the vote weights to the validators' decryption key.

The proof statements in a `Vote` description establishing spend authority over
the note are almost identical to those in a `Spend` description.  However, there
are two key differences.  First, rather than proving that the note was included
in a recent state commitment tree state, it always uses the root of the note
commitment tree at the time that voting began, establishing that the note was
not created after voting began.  Second, rather than checking the note's
nullifier against the global nullifier set and marking it as spent, the
nullifier is checked against a snapshot of the nullifier set at the time that
voting began (establishing that it was unspent then), as well as against a
per-proposal nullifier set (establishing that it has not already been used for
voting).  In other words, instead of marking that the note has been spent in
general, we only mark it as having been spent in the context of voting on a
specific proposal.

This change allows multiple proposals to be voted on concurrently, at the cost
of linkability.  While the same note can be used to vote on multiple proposals,
those votes, as well as the subsequent spend of the note, will have the same
nullifier and thus be linkable to each other.  However, the `Vote` descriptions
are shielded, so an observer only learns that two opaque votes were related to
each other.

We suggest that wallets roll over the note the first time it is used for voting
by creating a transaction with `Vote`, `Spend`, and `Output` descriptions.  This
mitigates linkability between `Vote` and `Spend` descriptions, and means that
votes on any proposals created after the first vote are unlinkable from prior
votes.

# Counting Votes

At the end of each epoch, validators collect the encrypted votes from each
delegation pool, aggregate the encrypted votes into encrypted tallies and
decrypt the tallies.  These intermediate tallies are revealed, because it is not
possible to batch value flows over time intervals longer than one epoch.  In
practice, this provides a similar dynamic as existing (transparent) on-chain
governance schemes, where tallies are public while voting is ongoing.

At the end of the voting period, the per-epoch tallies are summed.  For each
validator $v$, the votes for each option are summed to determine the portion of
the delegation pool that voted; the validator's vote acts as the default vote
for the rest of the delegation pool.  Finally, these per-validator subtotals are
multiplied by the [voting power adjustment function](../stake/voting-power.md)
$\theta_v(e)$ to obtain the final vote totals.

If the vote was not vetoed, the escrowed note from the `Proposal` description
is included in the state commitment tree, so that it can be spent by the
proposer.  Otherwise, it is not, and the funds are burned.

[^1]: If withdrawing a proposal halted on-chain voting immediately, the escrow
mechanism would not be effective at deterring spam, since the proposer could
yank their proposal at the last minute prior to losing their deposit.  However,
at the UX level, withdrawn proposals can be presented as though voting were
closed, since validators' default votes are probably sufficient for spam
deterrence.
