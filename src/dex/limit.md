# Limit Swaps

The swap system described in the [previous section](./swaps.md) provides
sealed-bid batch swaps. Because the swap amounts are encrypted, front-running
is not possible, and because the swaps are batched, users get better prices
and improved privacy. However, that system has a serious weakness: it makes
it very difficult for an arbitrageur to make trades that move the price of a
trading pair to match an external market, removing the price oracle
mechanism.

The arbitrageur's problem is that the clearing price for the batch depends on
the combined inputs to the batch, which they cannot observe or guarantee at
the time they submit their orders. For instance, if a price of some trading
pair on an external market moves by 1%, an arbitrageur can compute exactly
the trade size that they would need to make to correct the price on Penumbra.
But other arbitrageurs might sumbit the same trade, causing the total batch
input to be too large, and overshoot.

To address this problem, we'd like to have a way to specify a limit price for
a swap, so that a swap would be included in the batch only if the batch
clearing price (with that order included) would be within the specified limit.

More precisely, instead of having a swap consist of
$$
\sigma^{(i)} = \operatorname{Enc}_D(\Delta^{(i)}) 
= (\operatorname{Enc}_D(\Delta_1^{(i)}), \operatorname{Enc}_D(\Delta_2^{(i)})),
$$
we have a swap consist of
$$
\sigma^{(i)} = 
(\operatorname{Enc}_D(\Delta^{(i)}), \ell^{(i)}),
$$
where $\ell = (\ell_{1,2}, \ell_{2,1})$ is a pair of limits, so that
$\sigma^{(i)}$ is included in the batch only when the resulting clearing prices
satisfy
$$
\ell_{1,2}^{(i)} < p_{1,2}  \\\\
\ell_{2,1}^{(i)} < p_{2,1}
$$
Since the prices $p_{1, 2}$ and $p_{2, 1}$ are defined so that
$$
\Lambda_1 = p_{1,2} \gamma \Delta_2 \\\\
\Lambda_2 = p_{2,1} \gamma \Delta_1
$$
the limit conditions bound from below the outputs that the swap $\sigma^{(i)}$ recieves as
$$
\ell_{1,2}^{(i)} \gamma \Delta_2^{(i)} < \Lambda_1^{(i)} \\\\
\ell_{2,1}^{(i)} \gamma \Delta_1^{(i)} < \Lambda_2^{(i)}  
$$

Intuitively, larger values provide more restrictive limits; the limit $\ell =
(0,0)$ will always be satisfied and provides a universal default bound for
people who want privacy and are happy to be price-takers. This suggests that
we can order swaps by the size of their limits. Since rational traders only
choose one of $\Delta_1^{(i)}$ and $\Delta_2^{(i)}$ to be nonzero, they also
only choose one of $\ell_{1,2}$ and $\ell_{2,1}$ to be nonzero. This suggests
defining
$$
|\ell| = \frac {\ell_{1,2}} {p_{1,2}^{start}} + \frac {\ell_{2,1}} {p_{2,1}^{start}},
$$
scaling the limits to the starting prices of the liquidity pool so that the
sizes of $\ell_{1,2}$ and $\ell_{2,1}$ are comparable.

Now we could try to assemble a batch as follows: sort the candidate swaps by
limit size, smallest to largest, and sequentially attempt to add them into
the batch, checking that the clearing price for the batch remains below the
swap's limit. Swaps for which the clearing price exceeds the limits are
rejected from the batch. Sweep descriptions handle rejection by checking
whether the swap was included or rejected and mint either the opposite asset
type to sweep the output (as before) or mint the original asset type to
recover the inputs.

However, the validators can't actually do this, because computing the
clearing price for the batch requires knowledge of the swap inputs, but the
validators actually have only the encrypted swap inputs. There are two
potential solutions:

1. The validators aggregate the encrypted inputs of all of the swaps with
limit $\ell^{(i)} = (0,0)$, i.e., the price-taking swaps, and jointly decrypt
the sum as the initial batch input. They also jointly decrypt all of the
swaps with limits individually, then do the computation above, ordering the
remaining swaps from smallest to largest limit and sequentially adding swaps
to the batch or rejecting them.

2. The validators aggregate the encrypted inputs of all of the swaps, and
jointly decrypt the sum as the maximal batch input. Then, they sort the swaps
by limit, largest to smallest, and peel rejected swaps out of the batch by
checking whether the current clearing price for the batch exceeds the limit.
If so, the validators jointly decrypt the input amount of the rejected swap,
and subtract it from the batch input.

In approach (1), the process reveals the exact amounts of all swaps with
limits, and provides batching only between users who are price-takers. In
contrast, approach (2) only reveals the amounts of rejected swaps, so that
limit swaps placed by arbitrageurs provide cover traffic for price-takers.
Unfortunately, it requires that the validators perform strictly sequential
joint decryptions to process the batch, which could cause latency problems.

More design progress requires fleshing out more details on how the threshold
decryption should work (and work with ABCI++).
