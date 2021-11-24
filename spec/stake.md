# Staking and Delegation

As described in the [overview](./penumbra.md), integrating privacy with
proof-of-stake poses significant challenges.  Penumbra sidesteps these
challenges using a novel mechanism that eliminates staking rewards entirely,
treating unbonded and bonded stake as separate assets, with an epoch-varying
exchange rate that prices in what would be a staking reward in other systems.
This mechanism ensures that all delegations to a particular validator are
fungible, and can be represented by a single *delegation token* representing a
share of that validator's delegation pool, in effect a first-class staking
derivative.

This section describes the staking and delegation mechanism in detail:

- [Staking Tokens](./stake/tokens.md) describes the different staking tokens;
- [Validator Rewards and Fees](./stake/validator-rewards.md) describes mechanics around validator commissions and transaction fees;
- [Voting Power](./stake/voting-power.md) describes how validators' voting power is calculated;
- [Delegation](./stake/delegation.md) describes how users bond stake to validators;
- [Undelegation](./stake/undelegation.md) describes how users unbond stake from validators;
- [Example Staking Dynamics](./stake/example.md) contains a worked example illustrating the mechanism design.
