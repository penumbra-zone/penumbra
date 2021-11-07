# Sealed-Bid Batch Auctions

ZSwap's sealed-bid batch auctions conceptually decompose into two parts: the
market-making function itself, whose design is inspired by Uniswap v3, and the
batching procedure, which allows multiple users' swaps to be batched into a
single trade executed against the trading function.  This section focuses on the
batching procedure, leaving the mechanics of the trading function for later.

A key challenge in the design of any private swap mechanism is that
zero-knowledge proofs only allow privacy for user-specific state, not for global
state, because [they don't let you prove statements about things that you don't know][barrywhitehat-private-uniswap].  While users can prove that their
user-specific state was updated correctly without revealing it, they cannot
do so for other users' state.

Instead of solving this problem, ZSwap sidesteps the need for users to do so.
Rather than have users transact with each other, the chain permits them to
transmute one asset type to another, provably updating their private state
without interacting with any other users' private state.  This works as follows.

1. Users create transactions with `Swap` descriptions that privately burn their
input assets and encrypt the amounts to the validators.  This description
identifies the trading pair $(t_1, t_2)$, consumes $\Delta = (\Delta_1,
\Delta_2)$ of types $(t_1, t_2)$ from the transaction balance, and contains an
encryption of the trade inputs $$\operatorname{Enc}_D(\Delta) =
(\operatorname{Enc}_D(\Delta_1), \operatorname{Enc}_D(\Delta_2))$$ and a swap
commitment (analogous to a note commitment) that binds to $\Delta$ and a
nullifier. Rational traders will choose either $\Delta_1 = 0$ or $\Delta_2 = 0$,
i.e., trade one asset type for the other type, but the description provides two
inputs so that different swap directions cannot be distinguished.

2. Validators sum the encrypted amounts $\operatorname{Enc}_D(\Delta^{(i)})$ of
all swaps in the batch to obtain an encryption of the combined inputs
$\operatorname{Enc}_D(\sum_i \Delta^{(i)})$, then decrypt to obtain the batch
input $\Delta = \sum_i \Delta^{(i)}$ without revealing any individual
transaction's input $\Delta^{(i)}$.  Then they execute $\Delta$ against the
trading pool, updating the pool state and obtaining the effective (inclusive of
fees) clearing prices $p_{t_1,t_2}$ and $p_{t_2, t_1}$.

3. In a future block, users who created `Swap` descriptions obtain assets of the
new types by creating a `Sweep` descriptions. This description reveals the block
height $h$ containing the `Swap` description, the trading pair $(t_1, t_2)$, and
the nullifier $n$ of a swap commitment. Then, it proves inclusion of a swap
commitment with nullifier $n$ and trade inputs $\Delta$ in the note commitment
tree for that block, and proves that $\Lambda_1 = \Delta_2 p_{t_1,t_2}$ and
$\Lambda_2 = \Delta_1 p_{t_2, t_1}$.  Finally, it adds $(\Lambda_1, \Lambda_2)$
of types $(t_1, t_2)$ to the transaction's balance (e.g., to be consumed by an
`Output` description that creates a new note).

Although sweep descriptions do not reveal the amounts, or which swap's
outputs they claim, they do reveal the block and trading pair, so their
anonymity set is considerably smaller than an ordinary shielded value
transfer. For this reason, client software should create and submit a
transaction with a sweep description immediately after observing that its
transaction with a swap description was included in a block, rather than
waiting for some future use of the new assets. This ensures that future
shielded transactions involving the new assets are not trivially linkable to
the swap.

This design reveals only the net flow across a trading pair in each batch,
not the amounts of any individual swap. However, this provides no protection
if the batch contains a single swap, and limited protection when there are
only a few other swaps. This is likely to be an especially severe problem
until the protocol has a significant base of active users, so it is worth
examining the impact of amount disclosure and potential mitigations.

Assuming that all amounts are disclosed, an attacker could attempt to
deanonymize parts of the transaction graph by tracing amounts, using
strategies similar to those in [An Empirical Analysis of Anonymity in
Zcash][zcash_anon]. That research attempted to deanonymize transactions by
analysing movement between Zcash's transparent and shielded pools, with some
notable successes (e.g., identifying transactions associated to the sale of
stolen NSA documents). Unlike Zcash, where opt-in privacy means the bulk of
the transaction graph is exposed, Penumbra does not have a transparent pool,
and the bulk of the transaction graph is hidden, but there are several
potential places to try to correlate amounts:

- IBC transfers into Penumbra are analogous to `t2z` transactions and disclose
  types and amounts and accounts on the source chain;
- IBC transfers out of Penumbra are analogous to `z2t` transactions and disclose
  types and amounts and accounts on the destination chain;
- Each unbonding discloses the precise amount of newly unbonded stake and the
  validator;
- Each epoch discloses the net amount of newly bonded stake for each validator;
- Liquidity pool deposits disclose the precise type and amount of newly
  deposited reserves;
- Liquidity pool deposits disclose the precise type and amount of newly
  withdrawn reserves;

The existence of the swap mechanism potentially makes correlation by amount more
difficult, by expanding the search space from all amounts of one type to all
combinations of all amounts of any tradeable type and all historic clearing
prices.  However, assuming rational trades may cut this search space
considerably.[^1]

## TODO

- [ ] consider the effect of multi-asset support on correlations more deeply.

[zcash_anon]: https://arxiv.org/pdf/1805.03180.pdf

[^1]: Thanks to Guillermo Angeris for this observation.
