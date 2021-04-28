# Counting Votes

At the end of each epoch, validators collect the encrypted votes from each
delegation pool, aggregate the encrypted votes into encrypted tallies and
decrypt the tallies. At the end of the voting period, these per-epoch,
per-validator tallies are combined as follows. For each validator $v$, the
per-epoch tallies from that validator's delegation pool are summed to obtain
voting power subtotals for each vote option. Then, these subtotals are summed
to determine the amount of the delegation pool that voted. The validator's
vote acts as a default vote for the rest of the delegation pool. Finally,
these per-validator subtotals are multiplied by the voting power adjustment
function $\theta_v(e)$ to obtain the final vote totals.

If the vote was not vetoed, the escrowed note from the `ProposalDescription`
is included in the note commitment tree, so that it can be spent by the
proposer.  Otherwise, it is not, and the funds are burned.
