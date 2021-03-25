# Swaps

This section assumes the existence of a liquidity pool for a particular
trading pair and describes the mechanics of the swap system from a users'
perspective. Creation of liquidity pools and liquidity provision are
discussed below.

Users wishing to convert assets of type $t_1$ to type $t_2$ create a
transaction with a `SwapDescription`. This description identifies the trading
pair $(t_1, t_2)$, consumes $\Delta = (\Delta_1, \Delta_2)$ of types $(t_1,
t_2)$ from the transaction balance, and contains an encryption of the trade
inputs $$\operatorname{Enc}_D(\Delta) = (\operatorname{Enc}_D(\Delta_1),
\operatorname{Enc}_D(\Delta_2))$$ and a swap commitment (analogous to a note
commitment) that binds to $\Delta$ and a nullifier. Rational traders will
choose either $\Delta_1 = 0$ or $\Delta_2 = 0$, i.e., trade one asset type
for the other type, but the description provides two inputs so that different
swap directions cannot be distinguished.

For each trading pair appearing in a block, validators sum the encrypted
trade inputs $\operatorname{Enc}_D(\Delta^{(i)})$ from all swap descriptions
for that trading pair to obtain an encryption of the combined inputs
$\operatorname{Enc}_D(\sum_i \Delta^{(i)})$, then decrypt to obtain the batch
input $\Delta = \sum_i \Delta^{(i)}$ without revealing any individual
transaction's input $\Delta^{(i)}$.

Given the batch input $\Delta$, the batch outputs are $$\Lambda_1 = \frac {R_1
+ \gamma \Delta_1} {R_2 + \gamma \Delta_2} \gamma \Delta_2$$ and $$\Lambda_2 =
\frac {R_2 + \gamma \Delta_2} {R_1 + \gamma \Delta_1} \gamma \Delta_1.$$ The
clearing price for the batch is $$p_{t_1, t_2} = \frac {R_1 + \gamma\Delta_1}
{R_2 + \gamma\Delta_2} = \frac {\Lambda_1} {\gamma \Delta_2} = \frac {\gamma
\Delta_1} {\Lambda_2}.$$

The validators use the batch input $\Delta$ to compute the new reserves $R' =
R + \Delta - \Lambda$. The new reserves $R'$ and the clearing price $p_{t_1,
t_2}$ are recorded in the block.

In a future block, users who created swap descriptions obtain assets of the
new types by creating a `SweepDescription`. This description reveals the
block height $h$ containing the swap description, the trading pair $(t_1,
t_2)$, and the nullifier $n$ of a swap commitment. Then, it proves inclusion
of a swap commitment with nullifier $n$ and trade inputs $\Delta$ in the swap
commitment tree for that block and trading pair, and proves that $\Lambda_1 =
\gamma \Delta_2 p_{t_1,t_2}$ and $\Lambda_2 = \gamma \Delta_1 / p_{t_1,
t_2}$. Finally, it adds $(\Lambda_1, \Lambda_2)$ of types $(t_1, t_2)$ to the
transaction's balance (e.g., to be consumed by an `OutputDescription`).

Although sweep descriptions do not reveal the amounts, or which swap's
outputs they claim, they do reveal the block and trading pair, so their
anonymity set is considerably smaller than an ordinary shielded value
transfer. For this reason, client software should create and submit a
transaction with a sweep description immediately after observing that its
transaction with a swap description was included in a block, rather than
waiting for some future use of the new assets. This ensures that future
transactions involving the new assets are indistinguishable from any other
shielded transactions.
