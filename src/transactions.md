# IBC-enabled Shielded Transactions

Penumbra's transaction model is derived from the Sapling shielded transaction
design, with [modifications][multi_asset] to support multiple asset types,
and several extensions to support additional functionality.

Value is recorded in *notes*, which function similarly to UTXOs. However,
unlike UTXOs, notes are not recorded as part of the public chain state.
Instead, the chain contains a *note commitment tree*, an incremental Merkle
tree containing (public) commitments to (private) notes. Spending a note
involves proving that the spent note was previously included in the note
commitment tree, and revealing a *nullifier* which prevents spending the same
note twice.

Asset types are uniquely identified by an [ADR001] name for compatibility
with [IBC], and Penumbra supports IBC transfers
[into](./transactions/ibc-in.md) and [out of](./transactions/ibc-out.md) of
the zone, allowing Penumbra's shielded pool to privately record any asset in
the Cosmos ecosystem. ADR001 precisely specifies the entire pathway to the
origin of any token, but when this precision is not required, we denote the
Penumbra bearer tokens for asset `XYZ` as `pXYZ`; e.g., transferring `ATOM`
from the Hub to Penumbra results in `pATOM`.

Unlike the Cosmos Hub or many other chains built on the Cosmos SDK, Penumbra
has no notion of accounts. Only validators have any kind of long-term
identity, and this identity is only used (in the context of transactions) for
spending the validator's commission. Unlike Zcash, Penumbra has no notion of
transparent transactions or a transparent value pool. Instead, all value is
recorded in a single multi-asset shielded pool. Inbound IBC transfers shield
tokens analogously to `t2z` Zcash transactions, outbound IBC transfers
unshield tokens analogously to `z2t` Zcash transactions, and the entire
Cosmos ecosystem functions analogously to Zcash's transparent pool.

Transactions consist of a sequence of *descriptions* for various actions.
Each description adds or subtracts (typed) value from the transaction's value
balance, which must net to zero. Penumbra uses Sapling's `SpendDescription`,
which spends a note and adds to the transaction's value balance, and
`OutputDescription`, which creates a new note and subtracts from the
transaction's value balance, and adds many new descriptions to support
additional functionality:

- `TransferDescription`s transfer value out of Penumbra by IBC, consuming
value from the transaction's value balance, and producing an [ICS20]
[`FungibleTokenPacketData`][ftpd];

- [`DelegateDescription`s](./stake/delegation.md) convert unbonded stake to
bonded stake, consuming unbonded stake from the transaction's value balance
and producing new notes recording bonded stake;

- [`UndelegateDescription`s](./stake/undelegation.md) convert bonded stake to
unbonded stake, consuming bonded stake from the transaction's value balance
and producing new notes recording unbonded stake;

- [`ProposalDescription`s](./stake/governance.md) are used to propose
measures for on-chain governance and supply a deposit, consuming bonded stake
from the transaction's value balance and producing a new note that holds the
deposit in escrow;

- [`VoteDescription`s](./stake/governance.md) perform private voting for
on-chain governance and declare a vote, consuming bonded stake from the
transaction's value balance and producing a new note with the same amount of
bonded stake;

- [`SwapDescription`s](./dex/swaps.md) consume and burn tokens of one type
from a transaction's value balance and produce a swap commitment that permits
the user to later mint tokens of a different type;

- [`SweepDescription`s](./dex/swaps.md) allow a user who burned tokens of one
type to mint tokens of a different type at a chain-specified clearing price,
and adds the new tokens to a transaction's value balance.

- `DepositDescription`s deposit funds into the liquidity pool for a trading
pair, and produce liquidity pool shares, consuming value of the traded types
and producing value of the shares' type;

- `WithdrawDescription`s redeem liquidity pool shares and withdraw funds from
a liquidity pool, consuming the shares' type and producing value of the
traded types;

- [`FeeDescription`s](./stake/validator-rewards.md) declare and burn
transaction fees;

- [`CommissionDescription`s](./stake/validator-rewards.md) are used by
validators to sweep commission on staking rewards into shielded notes, adding
unbonded stake to the transaction's value balance;

## TODO

- [ ] specify the details of all of these, potentially in a new subsection


[multi_asset]: https://github.com/zcash/zips/blob/626ea6ed78863290371a4e8bc74ccf8e92292099/drafts/zip-user-defined-assets.rst
[ADR001]: https://docs.cosmos.network/master/architecture/adr-001-coin-source-tracing.html
[IBC]: https://docs.cosmos.network/master/ibc/overview.html
[ftpd]: https://github.com/cosmos/ibc/blob/master/spec/app/ics-020-fungible-token-transfer/README.md#data-structures
[ICS20]: https://github.com/cosmos/ibc/blob/master/spec/app/ics-020-fungible-token-transfer/README.md