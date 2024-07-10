# Privacy

This document describes the privacy features of the Penumbra protocol, as well as conditions that must be true for privacy to be preserved.

## Transfers

Transfers do not reveal the asset, amount, sender or recipient identity. Transfers are also unlinkable to each other.

## Swaps

Initiating a swap does not reveal the identity of the swapper or the pre-paid claim fee, but it does reveal the assets and amounts in the swap.

Claiming a swap does not reveal the amounts, asset types or the identity of the claimant. 

An observer of the chain will see that an anonymous account minted shielded outputs of a swap in a trading pair, but those outputs can't be linked to the claimant.

## Governance Voting

During a vote, the voting power (amount and asset type of the staked note that's used for voting),
the vote itself, the identity of the validator (equivalent to the asset type), as well as the proposal being voted on are **revealed** during a delegator vote.
However, the address of the voter is hidden, which renders the information revealed anonymous.

## Staking

TODO

## IBC

TODO