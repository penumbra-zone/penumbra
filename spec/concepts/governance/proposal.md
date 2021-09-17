# Proposal Creation

Penumbra users can propose votes by escrowing a minimum amount of any flavor
of `PENb`. They do this by creating a transaction with a
`ProposalDescription`, which consumes some amount of `PENb` from the
transaction's balance, and creates a new escrow note with the same amount.
The escrow note is not included in the note commitment tree until voting
completes. The voting period begins in the next epoch and lasts for a fixed
number of epochs (e.g., 7).

## TODO

- [ ] work out details enabling joint escrow (proposal crowdfunding)
- [ ] need proposition selection (only one proposition can be voted on at a time)