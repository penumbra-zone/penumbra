# Transactions

Transactions describe an atomic collection of changes to the ledger state.
Each transaction consist of a sequence of *descriptions* for various actions.
Each description adds or subtracts (typed) value from the transaction's value
balance, which must net to zero.  Penumbra uses Sapling's *spend description*,
which spends a note and adds to the transaction's value balance, and
*output description*, which creates a new note and subtracts from the
transaction's value balance, and adds many new descriptions to support
additional functionality:

- [`FeeDescription`s](./concepts/stake/validator-rewards.md) declare and burn
transaction fees;

- `TransferDescription`s transfer value out of Penumbra by IBC, consuming
value from the transaction's value balance, and producing an [ICS20]
[`FungibleTokenPacketData`][ftpd];

- [`DelegateDescription`s](./concepts/stake/delegation.md) convert unbonded stake to
bonded stake, consuming unbonded stake from the transaction's value balance
and producing new notes recording bonded stake;

- [`UndelegateDescription`s](./concepts/stake/undelegation.md) convert bonded stake to
unbonded stake, consuming bonded stake from the transaction's value balance
and producing new notes recording unbonded stake;

- [`CommissionDescription`s](./concepts/stake/validator-rewards.md) are used by
validators to sweep commission on staking rewards into shielded notes, adding
unbonded stake to the transaction's value balance;

- [`ProposalDescription`s](./concepts/governance/proposal.md) are used to propose
measures for on-chain governance and supply a deposit, consuming bonded stake
from the transaction's value balance and producing a new note that holds the
deposit in escrow;

- [`VoteDescription`s](./concepts/governance/voting.md) perform private voting for
on-chain governance and declare a vote, consuming bonded stake from the
transaction's value balance and producing a new note with the same amount of
bonded stake;

- [`SwapDescription`s](./concepts/dex/swaps.md) consume and burn tokens of one type
from a transaction's value balance and produce a swap commitment that permits
the user to later mint tokens of a different type;

- [`SweepDescription`s](./concepts/dex/swaps.md) allow a user who burned tokens of one
type to mint tokens of a different type at a chain-specified clearing price,
and adds the new tokens to a transaction's value balance.

- `DepositDescription`s deposit funds into the liquidity pool for a trading
pair, and produce liquidity pool shares, consuming value of the traded types
and producing value of the shares' type;

- `WithdrawDescription`s redeem liquidity pool shares and withdraw funds from
a liquidity pool, consuming the shares' type and producing value of the
traded types;

## TODO

- [ ] add links to specifications in the Cryptography section


[multi_asset]: https://github.com/zcash/zips/blob/626ea6ed78863290371a4e8bc74ccf8e92292099/drafts/zip-user-defined-assets.rst
[ADR001]: https://docs.cosmos.network/master/architecture/adr-001-coin-source-tracing.html
[IBC]: https://docs.cosmos.network/master/ibc/overview.html
[ftpd]: https://github.com/cosmos/ibc/blob/master/spec/app/ics-020-fungible-token-transfer/README.md#data-structures
[ICS20]: https://github.com/cosmos/ibc/blob/master/spec/app/ics-020-fungible-token-transfer/README.md