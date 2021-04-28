# Private Voting

Votes are the same as on the Cosmos Hub: `Yes`, `No`, `NoWithVeto`, and
`Abstain`. `NoWithVeto` is the same as `No` but also votes that the proposer
should lose their deposit. The intended cultural norm is that `No` should be
used to indicate disagreement with good-faith proposals and `NoWithVeto`
should be used to deter spam proposals.

- [ ] this isn't exactly the same as on the hub, where there's a distinct
  veto mechanic; is that mechanic good to copy?

By default, stakeholders' votes are delegated to the validator their stake is
bonded to.  Validators' votes are public, signed by the validator's key, and
act as a default vote for their entire delegation pool.

Stakeholders vote by proving ownership of some amount of bonded stake (their
voting power) prior to the beginning of the voting period.

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

## TODO

- [ ] vote acceptance thresholds / veto behavior
- [ ] encrypt voting power for each vote choice so that the vote does not reveal it