# Liquidity Incentives for `PEN` Markets

Creating on-chain markets involving the staking token `PEN` poses challenges
related to [competition between staking and on-chain lending][competition].
As that paper highlights, if staking rewards and the expected yields of
lending to liquidity pools are not calibrated appropriately, on-chain lending
can cause stake holders to prefer to lend their tokens rather than bond them,
thus cannibalizing network security. On the other hand, if the staking reward
is too large, markets involving unbonded stake may have scarce liquidity,
impeding price discovery. While Penumbra has first-class staking derivatives
that allow bonded stake to be traded like any other asset, each validator's
bonded stake is a distinct asset, fragmenting liquidity. Finally, other
projects have demonstrated that liquidity incentives can be an effective way
to distribute stake in a protocol to the protocol's users.

For these reasons, it's desirable to be able to adjust incentives between
staking and lending, at least in `PEN` markets. This can be accomplished by
introducing a swap reward rate $s_e$, analogous to the base reward rate $r_e$
for staking, and having the chain inflate the `PEN` portion of liquidity
reserves by a factor $1 + s_e$ on every epoch. If $s_e \leq r_e$, the
chain retains fixed supply of bonded stake, since lent stake cannot
appreciate faster than bonded stake.

The reward rate $s_e$ can be chosen by governance.

[competition]: https://arxiv.org/pdf/2001.00919.pdf