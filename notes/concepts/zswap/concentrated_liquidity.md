# Concentrated Liquidity

ZSwap uses the *concentrated liquidity* mechanism introduced in [Uniswap
v3][uv3] to make liquidity provision more capital-efficient.  Because ZSwap is
integrated into the consensus software, the implementation is different, but
mechanism itself is unchanged, with a few exceptions that can be thought of as
switching from an identity-based account model to a capability-based UTXO model:

[uv3]: https://uniswap.org/whitepaper-v3.pdf

- Positions are immutable, and cannot be modified after the position is opened;
- Fees accrue to the position, and are claimed when the position is closed;
- Positions are not tied to an account, but instead contain a (randomized)
spending key that authorizes closing the position.

Otherwise, this description of concentrated liquidity follows the
Uniswap v3 design, adapted for the ZSwap context.

## Constant-product markets

Following the notation in [Angeris-Chitra 2020][ipocfmm], a constant function
market maker (CFMM) is a type of automated market maker whose behavior is
determined by a *trading function* $\varphi(R, \Delta, \Lambda)$ whose inputs
are the *trading reserves* $R \in \R^n_+$, the
*trade inputs* $\Delta \in \R^n_+$, and the *trade outputs* $\Delta \in
\R^n_+$.  A proposed trade $(\Delta, \Lambda)$ is accepted whenever
$$
\varphi(R, \Delta, \Lambda) = \varphi(R, 0, 0),
$$
i.e., whenever the trading function is held constant, in which case the reserves
are updated as
$$
R \gets R + \Delta - \Lambda.
$$

One instance of a CFMM is a constant product market maker for a pair of assets,
as in Uniswap v2, where the reserves lie on a constant-product curve $$R_1 R_2 =
k,$$ where $R_1$ is the quantity of the first asset in the liquidity pool and
$R_2$ is the quantity of the second.  In this case, the trading function is
$$
\varphi(R, \Delta, \Lambda) = (R_1 + \Delta_1 - \Lambda_1)(R_2 + \Delta_2 - \Lambda_2),
$$
or, applying a percentage fee $(1 - \gamma)$ to trades,
$$
\varphi(R, \Delta, \Lambda) = (R_1 + \gamma \Delta_1 - \Lambda_1)(R_2 + \gamma \Delta_2 - \Lambda_2).
$$

As noted in [Angeris-Chitra 2020][ipocfmm], in the presence of fees, no
individual rational trader will create a trade where both $\Delta_1$ and
$\Delta_2$ are nonzero, because this involves trading some amount of one asset
for a smaller amount of the same asset.  For this reason, the trading function
is often put into a form that treats the $\Delta_1 \neq 0$ (trading asset 1 for
asset 2) and $\Delta_2 \neq 0$ (trading asset 2 for asset 1) separately.
However, in ZSwap, all swaps in each batch are first aggregated into a single
trade, and only the aggregate is decrypted and executed against the CFMM.  This
means that the formulation of the trading function needs to handle the case
where both $\Delta_1$ and $\Delta_2$ are nonzero, as different traders may
trade in different directions in the same batch.

## Concentrated Liquidity

In Uniswap v2, liquidity providers (LPs) deposit both types of asset into the
trading pool's reserves pool and receive liquidity shares representing
fractional ownership of the pool.  The trading fees $(1-\gamma)\Delta$ accrue to
the pool and therefore to the LPs' liquidity shares.  However, this structure
means that liquidity is evenly distributed across the entire range of
theoretically possible prices, rather than the range of prices that the assets
in the pair actually trade at.

Uniswap v3 adopts *concentrated liquidity*, allowing liquidity providers to
restrict the range of prices their liquidity will be used for trading by
specifying a *position*: liquidity bounded to a specified price range.  When the
pool's price is inside of the range, the liquidity is active, contributes to
trading, and earns fees.  The position only needs to maintain enough reserves of
each asset to cover price movement up to the specified bounds.  The trading
function for each position is the translation of the standard $R_1 R_2 = k$
curve so that it is solvent exactly within the specified price range.  As the
price moves towards one of the bounds, the composition of the position's
reserves shifts towards one of the assets, becoming composed entirely of one
asset as the price crosses the bound.  At this point, the position becomes
inactive until the price reenters the position's range.

This design has a few notable implications:

1.  Traders can 

1.  The narrower a position's range, the less assets are required to provide the
same liquidity.  This means that active traders can compete to provide liquidity
in the most capital-efficient way.

## Positions

###### Ticks.

As in Uniswap v3, the space of prices is discretized into *ticks*. Ticks are
indexed by integers (*tick indices*) and geometrically spaced to express
relative price differences: tick index $i$ corresponds to price $1.0001^i$, so
that each tick is one basis point away from its neighbors.  For technical
reasons, it's convenient for pools to track square roots of prices using
half-integer indices $1.0001^{i/2}$.





[ipocfmm]: https://web.stanford.edu/~guillean/papers/constant_function_amms.pdf
