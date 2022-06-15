# ZSwap

Penumbra provides private, sealed-bid batch swaps using ZSwap.  ZSwap allows
users to privately swap between any pair of assets.  Individual swaps do not
reveal trade amounts.  Instead, all swaps in each block are executed in a single
batch.  Only the total amount in each batch is revealed, and only after the
batch has been finalized. This prevents front-running and provides better
execution, but also provides long-term privacy for individual swaps.  Users can
also provide liquidity by anonymously creating concentrated
liquidity positions.  These positions reveal the amount of liquidity and the
bounds in which it is concentrated, but are not otherwise linked to any
identity, so that (with some care) users can privately approximate arbitrary
trading functions without revealing their specific views about prices.

## Frequent batch swaps

[Budish, Cramton, and Shim (2015)][budish-cramton-shim] analyze trading in
traditional financial markets using the predominant continuous-time limit order
book market design, and find that high-frequency trading arises as a response to
mechanical arbitrage opportunities created by flawed market design:

> These findings suggest that while there is an arms race in speed, the arms
> race does not actually affect the size of the arbitrage prize; rather, it just
> continually raises the bar for how fast one has to be to capture a piece of the
> prize... Overall, our analysis suggests that the mechanical arbitrage
> opportunities and resulting arms race should be thought of as a constant of the
> market design, rather than as an inefficiency that is competed away over time.
>
> — Eric Budish, Peter Cramton, John Shim, _The High-Frequency Trading Arms
> Race: Frequent Batch Auctions as a Market Design Response_

Because these mechanical arbitrage opportunities arise from the market design
even in the presence of symmetrically observed public information, they do not
improve prices or produce value, but create arbitrage rents that increase the
cost of liquidity provision[^1].  Instead, they suggest changing from a
continuous-time model to a discrete-time model and performing frequent batch
auctions, executing all orders that arrive in the same discrete time step in a
single batch with a uniform price.

In the blockchain context, while projects like Uniswap have demonstrated the
power and success of CFMMs for decentralized liquidity provision, they have also
highlighted the mechanical arbitrage opportunities created by the mismatch
between continuous-time market designs and the state update model of the
underlying blockchain, which operates in discrete batches (blocks).  Each
prospective state update is broadcast to all participants to be queued in the
mempool, but only committed as part of the next block, and while it is queued
for inclusion in the consensus state, other participants can bid to manipulate
its ordering relative to other state updates (for instance, front-running a
trade).

This manipulation is enabled by two features:

- Although trades are always committed in a batch (a block), they are performed
individually, making them dependent on miners' choices of the ordering of trades
within a block;

- Because trades disclose the trade amount in advance of execution, all other
participants have the information necessary to manipulate them.

ZSwap addresses the first problem by executing all swaps in each block in a
single batch, first aggregating the amounts in each swap and then executing it
against the CFMM as a single trade.

ZSwap addresses the second problem by having users encrypt their swap amounts
using a [flow encryption][flow_enc] key controlled by the
validators, who aggregate the *encrypted* amounts and decrypt only the batch
trade.  This prevents front-running prior to block inclusion, and provides
privacy for individual trades (up to the size of the batch) afterwards.

Users do not experience additional trading latency from the batch swap
design, because the batch swaps occur in every block, which is already the
minimum latency for finalized state updates.  A longer batch latency could help
privacy for market-takers by increasing the number of swaps in each batch, but
would impair other trading and impose a worse user experience.

## Private, sealed-bid batch swaps

A key challenge in the design of any private swap mechanism is that
zero-knowledge proofs only allow privacy for user-specific state, not for global
state, because [they don't let you prove statements about things that you don't
know][bwh].  While users can prove that their user-specific state was updated
correctly without revealing it, they cannot do so for other users' state.

Instead of solving this problem, ZSwap sidesteps the need for users to do so.
At a high level, swaps work as follows: users privately burn funds of one kind
in a coordinated way that allows the chain to compute a per-block clearing
price, and mint or burn liquidity pool reserves.  Later, users privately mint
funds of the other kind, proving that they previously burned funds and that the
minted amount is consistent with the computed price and the burned amount.  No
interaction or transfer of funds between users or the liquidity pool reserves is
required.  Users do not transact with each other.  Instead, the chain permits
them to transmute one asset type to another, provably updating their private
state without interacting with any other users' private state.

This mechanism is described in more detail in the [Sealed-Bid Batch
Swaps](./zswap/swap.md) section.

## Concentrated Liquidity

ZSwap executes trades against *concentrated liquidity* positions.

Uniswap v3's insight was that that existing constant-product market makers like
Uniswap v2 allocate liquidity inefficiently, spreading it uniformly over the
entire range of possible prices for a trading pair.  Instead, allowing liquidity
providers (LPs) to restrict their liquidity to a price range of their choosing
provides a mechanism for market allocation of liquidity, concentrating it into
the range of prices that the assets in the pair actually trade. 

Liquidity providers create *positions* that tie a quantity of liquidity to a
specific price range.  Within that price range, the position acts as a
constant-product market maker with larger "virtual" reserves.  At each price,
the pool aggregates liquidity from all positions that contain that price, and
tracks which positions remain (or become) active as the price moves.  By
creating multiple positions, LPs can approximate arbitrary trading functions.

However, the concentrated liquidity mechanism in Uniswap v3 has a number of
limitations:

- Approximating an arbitrary trading function using a set of concentrated
liquidity positions is cumbersome and difficult, because each position is a
scaled and translated copy of the (non-linear) constant-product trading function;

- Liquidity providers cannot compete on trading fees, because positions must be
created with one of a limited number of fee tiers, and users cannot natively
route trades across different fee tiers;

- All liquidity positions are publicly linked to the account that created them,
so that any successful LP strategy can be immediately cloned by other parties.

Zswap solves all of these problems:

- Using linear (constant-price) concentrated liquidity positions makes
approximating an arbitrary trading function much easier, and makes the DEX
implementation much more efficient. Users can recover a constant-product (or any
other) trading function by creating multiple positions, pushing the choice of
curve out of consensus and into client software.

- Each position includes its own fee setting.  This fragments liquidity into
many (potentially tens of thousands) of distinct AMMs, each with a specific fee,
but this is not a problem, because Zswap can optimally route batched swaps
across all of them.

- While positions themselves are public, they are opened and closed anonymously.
This means that while the aggregate of all LPs' trading functions is public, the
trading function of each individual LP is not, so LPs can approximate their
desired trading functions without revealing their specific views about prices,
as long as they take care to avoid linking their positions (e.g., by timing or
amount).

Handling of concentrated liquidity is described in more detail in the
[Concentrated Liquidity](./zswap/concentrated_liquidity.md) section.

## Liquidity Compensation

Systems that support both staking and liquidity provision need a mechanism to
manage the [competitive equilibrium][staking-lending-competition] between these
two uses of the staking token.  If staking rewards are too high relative to LP
returns, LPs will choose to stake instead, causing liquidity to dry up.  On the
other hand, if LP returns are too high relative to staking rewards, stakers will
choose to provide liquidity instead, weakening the chain's economic security.

Penumbra's staking design, which provides native delegation tokens for each
validator, poses additional challenges.  LPs are disincentivized to provide
liquidity in the staking token, if they could otherwise provide liquidity for a
delegation token and get staking rewards as well as LP returns.  This is
undesirable for a number of reasons: it fragments liquidity across many
different delegation tokens, it undermines the security model to some extent (by
making it easier to unload delegation tokens before misbehavior is detected),
and it drives staking centralization (since larger validators would presumably
have deeper liquidity in their delegation token, making it a more attractive
asset).  

To address these challenges, and allow the protocol to stabilize competition
between staking and liquidity provision, ZSwap uses a mechanism called
*liquidity compensation*, so named because it compensates LPs for the
opportunity cost of not staking.  Instead of issuing staking rewards only to
delegators, the total issuance in each epoch is split between delegators and
liquidity providers.  To determine the split, the chain sets a target ratio of
bonded stake to stake used for liquidity provision, and compares the actual
ratio to the target ratio, similarly to the way that other systems adjust
staking rewards based on the proportion of tokens staked.

At the end of each epoch, the share of issuance used for liquidity compensation
is allocated to each eligible liquidity position active during that epoch, pro
rata to liquidity provided.  Eligible liquidity positions are any positions in
trading pairs where one asset is the staking token, and the other asset is not a
delegation token.

This mechanism has several nice properties:

- Although the amount of liquidity compensation is determined on the basis of
value locked, it's allocated on the basis of liquidity provided.  This
incentivizes LPs to efficiently deploy their capital, since LPs who can create
finer liquidity positions will receive disproportionate liquidity compensation
rewards.

- Because markets in delegation tokens are not eligible for liquidity
compensation, marketmakers providing liquidity between bonded and unbonded forms
of the staking token must take on the opportunity cost of not staking the
unbonded side.  Therefore, fees in those markets must be sufficient to cover
that opportunity cost, so that there's no free, instant withdrawal from a
delegation pool.

- The mechanism is credibly neutral, applying to any trading pair involving the
staking token, rather than artificially subsidizing certain tokens via
governance proposals.

This mechanism is described in more detail in the [Liquidity
Compensation](./zswap/liquidity_compensation.md) section.




[bwh]: https://ethresear.ch/t/why-you-cant-build-a-private-uniswap-with-zkps/7754
[flow_enc]: ./crypto/flow.md
[budish-cramton-shim]: https://faculty.chicagobooth.edu/eric.budish/research/HFT-FrequentBatchAuctions.pdf
[staking-lending-competition]: https://arxiv.org/abs/2001.00919

[^1]: on HFT, N.B. their footnote 5: 
> A point of clarification: our claim is *not* that markets are less liquid today than before the rise of electronic trading and HFT; the empirical record is clear that trading costs are lower today than in the pre-HFT era, though most of the benefits appear to have been realized in the late 1990s and early 2000s... Rather, our claim is that markets are less liquid today than they would be under an alternative market design that eliminated sniping.
