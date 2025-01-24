Penumbra is a fully private proof-of-stake network and decentralized exchange
for the Cosmos ecosystem.

Penumbra integrates privacy with proof-of-stake through a novel private
delegation mechanism that provides staking derivatives, tax-efficient staking,
and on-chain governance with private voting. Penumbra connects to the Cosmos
ecosystem via IBC, acting as an ecosystem-wide shielded pool and allowing
private transactions in any IBC-compatible asset.  Users can also swap these
assets using ZSwap, a private decentralized exchange supporting sealed-bid batch
auctions and Uniswap-v3-style concentrated liquidity.  Sealed-bid batch auctions
prevent frontrunning, provide better execution, and reveal only the net flow
across a pair of assets in each block, and liquidity positions are created
anonymously, allowing traders to approximate their desired trading function
without revealing their individual beliefs about prices.

This website renders the work-in-progress protocol specification for Penumbra.

Press `s` or use the magnifying glass icon for full-text search.

If you're interested in technical discussion about the project, why not

- join [the discord](https://discord.gg/hKvkrqa3zC),
- check out [the repo and issue tracker](https://github.com/penumbra-zone/penumbra),
- view the [roadmap goals](https://github.com/orgs/penumbra-zone/projects),
- or [follow the project on Twitter](https://twitter.com/penumbrazone) for updates.

## Private Transactions

Penumbra records all value in a single multi-asset shielded pool based on the
Zcash Sapling design, but allows private transactions in any kind of IBC
asset.  Inbound IBC transfers shield value as it moves into the zone, while
outbound IBC transfers unshield value.

Unlike Zcash, Penumbra has no notion of transparent transactions or a
transparent value pool; instead, inbound IBC transfers are analogous to `t2z`
Zcash transactions, outbound IBC transfers are analogous to `z2t` Zcash
transactions, and the entire Cosmos ecosystem functions analogously to
Zcash's transparent pool.

Unlike the Cosmos Hub or many other chains built on the Cosmos SDK, Penumbra
has no notion of accounts.  Only validators have any kind of long-term
identity, and this identity is only used (in the context of transactions) for
spending the validator's commission.

## Private Staking

In a proof-of-stake system like the Cosmos Hub, stakeholders delegate staking
tokens by bonding them to validators.  Validators participate in Tendermint
consensus with voting power determined by their delegation size, and
delegators receive staking rewards in exchange for taking on the risk of
being penalized for validator misbehavior (slashing).

Integrating privacy and proof of stake poses significant challenges.  If
delegations are public, holders of the staking token must choose between
privacy on the one hand and staking rewards and participation in consensus on
the other hand.  Because the majority of the stake will be bonded to
validators, privacy becomes an uncommon, opt-in case.  But if delegations are
private, issuing staking rewards becomes very difficult, because the chain no
longer knows the amount and duration of each address' delegations.

Penumbra sidesteps this problem using a new mechanism that eliminates staking
rewards entirely, treating unbonded and bonded stake as separate assets, with an
epoch-varying exchange rate that prices in what would be a staking reward in
other systems.  This mechanism ensures that all delegations to a particular
validator are fungible, and can be represented by a single *delegation token*
representing a share of that validator's delegation pool, in effect a
first-class staking derivative.  Although delegation fungibility is key to
enabling privacy, as a side effect, delegators do not realize any income while
their stake is bonded, only a capital gain (or loss) on unbonding.

The total amount of stake bonded to each validator is part of the public chain
state and determines consensus weight, but the delegation tokens themselves are
each just another token to be recorded in a multi-asset shielded pool.  This
provides accountability for validators and privacy and flexibility for
delegators, who can trade and transact with their delegation tokens just like
they can with any other token.

It also provides an alternate perspective on the debate between fixed-supply and
inflation-based rewards.  Choosing the unbonded token as the numéraire,
delegators are rewarded by inflation for taking on the risk of validator
misbehavior, and the token supply grows over time.  Choosing (a basket of)
delegation tokens as the numéraire, non-delegators are punished by depreciation
for not taking on any risk of misbehavior, and the token supply is fixed.

## Private Governance

Like the Cosmos Hub, Penumbra supports on-chain governance with delegated
voting.  Unlike the Cosmos Hub, Penumbra's governance mechanism supports
secret ballots.  Penumbra users can anonymously propose votes by escrowing a
deposit of bonded stake.  Stakeholders vote by proving ownership of their
bonded stake prior to the beginning of the voting period and encrypting their
votes to a threshold key controlled by the validator set.  Validators sum
encrypted votes and decrypt only the per-epoch totals.

## Private DEX

Penumbra provides private batch swaps using [ZSwap](./dex.md).
ZSwap allows users to privately swap between any pair of assets.  Individual
swaps publicly burn their input notes and privately mint their output notes.
All swaps in each block are executed in a single batch.  Users can also provide
liquidity by anonymously creating concentrated liquidity positions.  These
positions reveal the amount of liquidity and the bounds in which it is
concentrated, but are not otherwise linked to any identity, so that (with some care) users can privately approximate arbitrary
trading functions without revealing their specific views about prices.
