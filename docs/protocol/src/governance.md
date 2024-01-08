# Governance

Penumbra features on-chain governance similar to Cosmos Hub, with the simplification that there are
only 3 kinds of vote: yes, no, and abstain.

## Voting On A Proposal

Validators and delegators may both vote on proposals. Validator votes are public and attributable to
that validator; delegator votes are anonymous, revealing only the voting power used in the vote, and
the validator which the voting delegator had delegated to. Neither validators nor delegators can
change their votes after they have voted.

### Eligibility And Voting Power

Only validators who were active at the time the proposal started voting may vote on proposals. Only
delegators who had staked delegation tokens to active validators at the time the proposal started
voting may vote on proposals.

A validator's voting power is equal to their voting power at the time a proposal started voting, and
a delegator's voting power is equal to the unbonded staking token value (i.e. the value in
`penumbra`) of the delegation tokens they had staked to an active validator at the time the proposal
started voting. When a delegator votes, their voting power is subtracted from the voting power of
the validator(s) to whom they had staked delegation notes at the time of the proposal start, and
their stake-weighted vote is added to the total of the votes: in other words, validators vote on
behalf of their delegators, but delegators may override their portion of their validator's vote.

## Authoring A Proposal

Anyone can submit a new governance proposal for voting by escrowing a _proposal deposit_, which will
be held until the end of the proposal's voting period. Penumbra's governance system discourages
proposal spam with a _slashing_ mechanism: proposals which receive more than a high threshold of no
votes have their deposit burned. At present, the slashing threshold is 80%. If the proposal is not
slashed (but regardless of whether it passes or fails), the deposit will then be returned to the
proposer at the end of voting.

From the proposer's point of view, the lifecycle of a proposal begins when it is
_submitted_ and ends when it the deposit is _claimed_. During the voting period, the proposer may
also optionally _withdraw_ the proposal, which prevents it from passing, but does not prevent it
from being slashed. This is usually used when a proposal has been superseded by a revised
alternative.

<picture>
  <source srcset="governance/governance-dark.png" media="(prefers-color-scheme: dark)" />
  <img src="governance/governance-light.png" />
</picture>

In the above, rounded gray boxes are actions submitted by the proposal author, rectangular colored
boxes are the state of the proposal on chain, and colored circles are outcomes of voting.

## Proposal NFTs

Control over a proposal, including the power to withdraw it before voting concludes, and control
over the escrowed proposal deposit, is mediated by a per-proposal family of bearer NFTs. For
proposal `N`, the NFT of denomination `proposal_N_submitted` is returned to the author. If
withdrawn, this NFT is consumed and a `proposal_N_withdrawn` NFT is returned. When finally the
escrowed deposit is claimed, a `proposal_N_claimed` NFT is returned to the author, as well as the
proposal deposit itself, provided the deposit has not been slashed. This is the final state.

### Kinds Of Proposal

There are 4 kinds of governance proposals on Penumbra: **signaling**, **emergency**, **parameter
change**, and **community pool spend**.

#### Signaling Proposals

Signaling proposals are meant to signal community consensus about something. They do not have a
mechanized effect on the chain when passed; they merely indicate that the community agrees about
something.

This kind of proposal is often used to agree on code changes; as such, an optional `commit` field
may be included to specify these changes.

#### Emergency Proposals

Emergency proposals are meant for when immediate action is required to address a crisis, and
conclude early as soon as a 2/3 majority of all active voting power votes yes.

Emergency proposals have the power to optionally halt the chain when passed. If this occurs,
off-chain coordination between validators will be required to restart the chain.

#### Parameter Change Proposals

Parameter change proposals alter the chain parameters when they are passed. Chain parameters specify
things like the base staking reward rate, the amount of penalty applied when slashing, and other
properties that determine how the chain behaves. Many of these can be changed by parameter change
proposals, but some cannot, and instead would require a chain halt and upgrade.

A parameter change proposal specifies both the _old_ and the _new_ parameters. If the current set of
parameters at the time the proposal passes are an _exact match_ for the old parameters specified in
the proposal, the entire set of parameters is immediately set to the new parameters; otherwise,
nothing happens. This is to prevent two simultaneous parameter change proposals from overwriting
each others' changes or merging with one another into an undesired state. Almost always, the set of
old parameters should be the current parameters at the time the proposal is submitted.

#### Community Pool Spend Proposals

Community Pool spend proposals submit a _transaction plan_ which may spend funds from the Community Pool if passed.

Community Pool spend transactions have exclusive capability to use two special actions which are not allowed in
directly submitted user transactions: `CommunityPoolSpend` and `CommunityPoolOutput`. These actions, respectively, spend
funds from the Community Pool, and mint funds _transparently_ to an output address (unlike regular output
actions, which are shielded). Community Pool spend transactions are unable to use regular shielded outputs,
spend funds from any source other than the Community Pool itself, perform swaps, or submit, withdraw, or claim
governance proposals.

## Validator Voting

A validator vote is a transparent action, signed by and attributable to the specific validator who
cast that vote on the proposal.

## Delegator Voting

A delegator vote consists of a spend proof for a given delegation note to some validator, coupled
with an inclusion proof showing that the creation block height of that delegation note was strictly
before voting started on the proposal. Additionally, the delegator vote reveals the nullifier for
the note which was used to justify the vote, to prevent double-votes on the same proposal (the node
keeps track of per-proposal nullifier sets for voting, entirely distinct from the main nullifier
set). This means that you can spend a delegation note, and then still subsequently use it to justify
a vote, so long as the time it was created was before when the proposal started voting.

This scheme means that clients _should_ "roll over" delegation notes upon voting to prevent their
votes on different proposals from being linkable by correlating nullifiers. If two proposals are
submitted concurrently, it is not possible for the delegator to prevent their votes on the two
proposals from being linked; this is considered an acceptable sacrifice.

## Contributing To The Community Pool

Anyone can contribute any amount of any denomination to the Penumbra Community Pool. Funds contributed to the
Community Pool cannot be withdrawn except by a successful Community Pool spend governance proposal. Community Pool spend proposals
are only accepted for voting if they would not overdraw the current funds in the Community Pool at the time the
proposal is submitted.

A validator may non-custodially send funds to the Community Pool, similarly to any other funding stream. To do
this, a validator declares in their validator declaration that one of their funding streams has the
Community Pool as a recipient rather than a penumbra address.
