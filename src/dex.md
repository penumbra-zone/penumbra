# Private DEX

Penumbra supports private, sealed-bid batch swaps, using a constant-product market maker similar to Uniswap.

At a high level, these swaps work as follows: users privately burn funds of
one kind in a coordinated way that allows the chain to compute a per-block
clearing price, and mint or burn liquidity pool reserves. Later, users prove
they previously burned funds of one kind and privately mint funds of the
other kind, proving that the minted amount is consistent with the computed
price and the amount they burned. No interaction or transfer of funds between
users or the liquidity pool reserves is required.

Following the notation in [Improved Price Oracles: Constant Function Market
Makers][ipocfmm], we write the reserves of the liquidity pool as $R = (R_1,
R_2) \in \mathbb R^2_+$, the inputs to a trade as $\Delta = (\Delta_1,
\Delta_2) \in \mathbb R^2_+$, the outputs from a trade as $\Lambda =
(\Lambda_1, \Lambda_2) \in \mathbb R^2_+$, and the trading function of the
CFMM as $\varphi(R, \Delta, \Lambda)$. Trades are accepted when $$\varphi(R,
\Delta, \Lambda) = \varphi(R, 0,0),$$ i.e., when the trading function is kept
constant, and the reserves are updated as $R \gets R + \Delta - \Lambda$.

Penumbra's trading function is $$\varphi(R, \Delta, \Lambda) = (R_1 + \gamma
\Delta_1 - \Lambda_1) (R_2 + \gamma \Delta_2 - \Lambda_2),$$ i.e., a
constant-product market with percentage fee $1 - \gamma$.

[ipocfmm]: https://web.stanford.edu/~guillean/papers/constant_function_amms.pdf