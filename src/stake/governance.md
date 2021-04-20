# Governance

Like the Cosmos Hub, Penumbra supports on-chain governance with delegated voting. Unlike the Cosmos Hub, Penumbra's governance mechanism supports secret ballots.

Penumbra users can propose votes by escrowing a minimum amount of any flavor
of `PENb`. They do this by creating a transaction with a
`ProposalDescription`, which consumes some amount of `PENb` from the
transaction's balance, and creates a new escrow note with the same amount.
The escrow note is not included in the note commitment tree until voting
completes. The voting period begins in the next epoch and lasts for a fixed
number of epochs (e.g., 7).

Votes are the same as on the Cosmos Hub: `Yes`, `No`, `NoWithVeto`, and
`Abstain`. `NoWithVeto` is the same as `No` but also votes that the proposer
should lose their deposit. The intended cultural norm is that `No` should be
used to indicate disagreement with good-faith proposals and `NoWithVeto`
should be used to deter spam proposals.

- [ ] this isn't exactly the same as on the hub, where there's a distinct veto mechanic; is that mechanic good to copy?

By default, stakeholders' votes are delegated to the validator their stake is bonded to. Validators' votes are public, signed by the validator's key, and act as a default vote for their entire delegation pool.

Stakeholders vote by proving ownership of some amount of bonded stake (their voting power) prior to the beginning of the voting period.

To do this, they create a transaction with a `VoteDescription`. This
description reveals the validator $v$ and one of the four votes above,
consumes $y$ `PENb(v)` from the transaction's balance, produces a new note
with $y$ `PENb`, and includes $\operatorname{Enc}_D(y)$, an encryption of the
voting power to the validators' decryption key.

Descriptions that spend notes contain a proof-of-inclusion for the note they
spend. This establishes that a commitment to the spent note was previously
included in the note commitment tree, an incremental Merkle tree whose
current root ("anchor") is included in each block. Normally, these proofs are
created and validated with respect to a recent anchor. However, transactions
containing `VoteDescription`s are always validated with respect to the anchor
in the last block before the voting period begins. This prevents
double-voting, by ensuring that only notes created before voting began can be
used to vote.

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

If the vote was not vetoed, the escrowed note from the `ProposalDescription` is included in the note commitment tree, so that it can be spent by the proposer.  Otherwise, it is not, and the funds are burned.

## TODO

- [ ] work out details enabling joint escrow (proposal crowdfunding)
- [ ] need proposition selection (only one proposition can be voted on at a time)
- [ ] vote acceptance thresholds / veto behavior
- [ ] emergency vs normal proposals, where emergency proposals are approved as soon as 2/3 validators vote yes
- [ ] is "liquid democracy"-style voting possible? currently the only supported delegation is to validators, but it could be useful to support delegation to third parties.  one way to do this would be to allow the proxy voter to create votes on behalf of their voting delegators, but this would require that the proxy spend and create their notes, which is a huge power.  this could be minimized by creating a special "voting key" capability, but nothing ensures that the proxy voter actually creates valid notes when spending them, so they could still destroy their voting delegators' power.  another 