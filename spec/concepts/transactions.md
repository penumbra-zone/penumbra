# Transactions

Transactions describe an atomic collection of changes to the ledger state.  Each
transaction consist of a sequence of *descriptions* for various actions[^1].
Each description adds or subtracts (typed) value from the transaction's value
balance, which must net to zero.  Penumbra adapts the *Spend* and *Output* actions from Sapling, and adds many new descriptions to support additional functionality:

Penumbra adapts Sapling's *Spend*, which
spends a note and adds to the transaction's value balance, and
*Output*, which creates a new note and subtracts from the
transaction's value balance, and adds many new descriptions to support
additional functionality:


- **Spend** descriptions spend an existing note, adding its value to the
transaction's value balance;

- **Output** descriptions create a new note, subtracting its value from the
transaction's value balance;

- **Transfer** descriptions transfer value out of Penumbra by IBC, consuming value
from the transaction's value balance, and producing an [ICS20]
[`FungibleTokenPacketData`][ftpd] for the counterparty chain;

- **Delegate** descriptions [deposit unbonded stake into a validator's delegation
pool](./stake/delegation.md), consuming unbonded stake from the
transaction's value balance and producing new notes recording delegation
tokens representing the appropriate share of the validator's delegation pool;

- **Undelegate** descriptions [withdraw from a validator's delegation
pool](./stake/undelegation.md), consuming delegation tokens from the
transaction's value balance and producing new notes recording the appropriate
amount of unbonded stake;

- **Commission** descriptions are used by validators to [sweep commission on
staking rewards](./stake/validator-rewards.md) into shielded notes,
adding unbonded stake to the transaction's value balance;

- **Proposal** descriptions  are used to [propose measures for on-chain
governance](./concepts/governance/proposal.md) and supply a deposit, consuming
bonded stake from the transaction's value balance and producing a new note that
holds the deposit in escrow;

- **Vote** descriptions perform [private voting for on-chain
governance](./concepts/governance/voting.md) and declare a vote, consuming
bonded stake from the transaction's value balance and producing a new note with
the same amount of bonded stake;

- **Swap** descriptions perform the first phase of
[ZSwap](./zswap/auction.md), consuming tokens of one type from a
transaction's value balance, burning them, and producing a swap commitment for
use in the second stage;

- **Sweep** descriptions perform the second phase of
[ZSwap](./zswap/auction.md), allowing a user who burned tokens of one
type to mint tokens of the other type at the chain-specified clearing price, and
adding the new tokens to a transaction's value balance;

- **OpenPosition** descriptions open [concentrated liquidity
positions](./zswap.md), consuming value of the traded types from the
transaction's value balance and adding the specified position to the AMM state;

- **ClosePosition** descriptions close [concentrated liquidity
positions](./zswap.md), removing the specified position to the AMM
state and adding the value of the position, plus any accumulated fees or
liquidity rewards, to the transaction's value balance.

Each transaction also contains a fee specification, which is always
transparently encoded. The value balance of all of a transactions actions,
together with the fees, must net to zero.

[^1]: Note that like Zcash Orchard, we use the term "action" to refer to one of
a number of possible state updates; unlike Orchard, we do not attempt to conceal
which types of state updates are performed, so our Action is an enum.

[multi_asset]: https://github.com/zcash/zips/blob/626ea6ed78863290371a4e8bc74ccf8e92292099/drafts/zip-user-defined-assets.rst
[ADR001]: https://docs.cosmos.network/master/architecture/adr-001-coin-source-tracing.html
[IBC]: https://docs.cosmos.network/master/ibc/overview.html
[ftpd]: https://github.com/cosmos/ibc/blob/master/spec/app/ics-020-fungible-token-transfer/README.md#data-structures
[ICS20]: https://github.com/cosmos/ibc/blob/master/spec/app/ics-020-fungible-token-transfer/README.md
