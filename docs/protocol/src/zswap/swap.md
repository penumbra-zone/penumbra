# Sealed-Bid Batch Swaps

ZSwap's sealed-bid batch swaps conceptually decompose into two parts: the DEX
and AMM mechanism itself, and the batching procedure, which allows multiple
users' swaps to be batched into a single trade executed against the trading
function.  This section focuses on the batching procedure, leaving the mechanics
of the trading function for later.

A key challenge in the design of any private swap mechanism is that
zero-knowledge proofs only allow privacy for user-specific state, not for global
state, because [they don't let you prove statements about things that you don't
know][cant-build].  While users can prove that their user-specific state was
updated correctly without revealing it, they cannot do so for other users'
state.

Instead of solving this problem, ZSwap sidesteps the need for users to do so.
Rather than have users transact with each other, the chain permits them to
transmute one asset type to another, provably updating their private state
without interacting with any other users' private state.  To do this, they
privately burn their input assets and encrypt the amounts to the validators. The
validators aggregate the encrypted amounts and decrypt the batch total, then
compute the effective (inclusive of fees) clearing prices and commit them to the
chain state.  In any later block, users can privately mint output funds of the new type, proving consistency with the inputs they burned.

### `Swap` actions

First, users create transactions with `Swap` actions that privately burn their
input assets and encrypt the amounts to the validators.  This action identifies
the trading pair $(t_1, t_2)$ by asset id, consumes $\Delta = (\Delta_1, \Delta_2)$ of types
$(t_1, t_2)$ from the transaction balance, and contains an encryption of the
trade inputs
$$
\operatorname{Enc}_D(\Delta) = (\operatorname{Enc}_D(\Delta_1),
\operatorname{Enc}_D(\Delta_2)).
$$
Rational traders will choose either
$\Delta_1 = 0$ or $\Delta_2 = 0$, i.e., trade one asset type for the other type,
but the description provides two inputs so that different swap directions cannot
be distinguished.
The `Swap` action also consumes $f$ fee tokens from the transaction's value balance, which are saved for use as a prepaid transaction fee when claiming the swap output.

To record the user's contribution for later, the action mints a *swap NFT*.
Penumbra assets are recorded
as a pair of an amount (`u64`) and an asset id ($\mathbb F_q$).  Usually, the
asset id is the hash of a denomination string. For a swap NFT, however, the
asset id is computed as
$$
\mathsf a_{NFT} = \pi (t_1, t_2, f, \Delta_1, \Delta_2, B_d, \mathsf{pk}_d),
$$
where:
- $\pi$ is a Poseidon hash function;
- $(\Delta_1, \Delta_2)$ are the input amounts of types $t_1$ and $t_2$ respectively;
- $f$ is a prepaid fee amount that will be used for the swap claim;
- $B_d$ and $\mathsf{pk}_d$ are the diversified basepoint and diversified transmission key of one of the user's addresses, used to preauthorize the swap claim.

The swap NFT is recorded like any other asset in the shielded pool.  The `Swap`
action includes a `NotePayload` with an encryption of a new note with the swap
NFT, rather than using a separate `Output` action, in order to combine proof
statements and skip a fixed-size memo field.

### Batching and Execution

In this description, which focuses on the state model, we treat the execution
itself as a black box and focus only on the public data used for swap outputs.

Validators sum the encrypted amounts $\operatorname{Enc}_D(\Delta^{(i)})$ of all
swaps in the batch to obtain an encryption of the combined inputs
$\operatorname{Enc}_D(\sum_i \Delta^{(i)})$, then decrypt to obtain the batch
input $\Delta = \sum_i \Delta^{(i)}$ without revealing any individual
transaction's input $\Delta^{(i)}$.  Then they execute $\Delta$ [against the open liquidity positions](../dex/routing.md), updating the positions' state and obtaining the effective (inclusive of
fees) clearing prices $p_{t_1,t_2}$ ($t_1$ in terms of $t_2$) and $p_{t_2, t_1}$
($t_2$ in terms of $t_1$).

Since there may not be enough liquidity to perform the entirety of the swap, unfilled portions of the input assets are also returned as ${(u_1, u_2)}$.  The batch swap is always considered successful regardless of available liquidity; for a batch swap where no liquidity positions exist to execute against, $u_1 = \Delta_1$ and $u_2 = \Delta_2$.

Each user's inputs to the swap are indicated as $(\Delta_{1 i}, \Delta_{2 i})$ and their
share of the output indicated as ${\Lambda_{1 i}, \Lambda_{2 i}}$.  Their pro rata fractions of the total input are therefore ${(\Delta_{1 i} / \Delta_1, \Delta_{2 i} / \Delta_2)}$.

Each user's output amounts can be computed as
$$
\Lambda_{1 i} = (\Delta_{1 i} / \Delta_1) * u_1 + (\Delta_{2 i} / \Delta_2) * \Lambda_1 \\
\Lambda_{2 i} = (\Delta_{1 i} / \Delta_1) * \Lambda_2 + (\Delta_{2 i} / \Delta_2) * u_2
$$

### Claiming Swap Outputs

In a future block, users who created transactions with `Swap` actions obtain
assets of the new types by creating a transaction with `SwapClaim` actions.
This action privately mints new tokens of the output type, and proves
consistency with the user's input contribution (via the swap NFT) and with the
effective clearing prices (which are part of the public chain state).  The
`SwapClaim` action is carefully designed so that it is *self-authenticating*,
and *does not require any spend authorization*.  Any entity in posession of the
full viewing key can create and submit a swap claim transaction, without further
authorization by the user.  This means that wallet software can automatically
claim swap outputs as soon as it sees confirmation that the swap occurred.

Like a `Spend` action, the `SwapClaim` action spends a shielded note, revealing its nullifier and witnessing an authentication path from it to a recent anchor.  However, it differs in several important respects:

- Rather than unlocking value from an arbitrary note, it proves that the spent note records $1$ unit of a swap NFT whose asset ID is $$ \mathsf a_{NFT} = \pi (t_1, t_2, f, \Delta_1, \Delta_2, B_d, \mathsf{pk}_d), $$
so that the input state is available to other proof statements;

- Rather than witnessing the full authentication path from the note commitment up to a recent anchor, it reveals the block height and only witnesses the authentication path up to the block-level root, proving that the note was included in a particular block[^2], and allowing reference to the effective
clearing prices $p_{t_1, t_2}$ and $p_{t_2, t_1}$;

- Rather than contributing to the transaction's value balance, it constructs two output notes itself, one for each of
    $$
    \Lambda_{1 i} = (\Delta_{1 i} / \Delta_1) * u_1 + (\Delta_{2 i} / \Delta_2) * \Lambda_1 \\
    \Lambda_{2 i} = (\Delta_{1 i} / \Delta_1) * \Lambda_2 + (\Delta_{2 i} / \Delta_2) * u_2
    $$
    and proves that the notes are sent to the address committed to by the
    $B_d$ and $\mathsf{pk}_d$ in the swap NFT;

The `SwapClaim` does not include a spend authorization signature, because it is only capable of consuming a swap NFT, not arbitrary notes, and only capable of sending the trade outputs to the address specified during construction of the original `Swap` action, which is signed.

Finally, the `SwapClaim` releases $f$ units of the fee token to the
transaction's value balance, allowing it to pay fees without an additional
`Spend` action.  The transaction claiming the swap outputs can therefore consist
of a single `SwapClaim` action, and that action can be prepared using only a
full viewing key.  This design means that wallet software can automatically
submit the swap claim without any explicit user intervention, even if the
user's custody setup (e.g., a hardware wallet) would otherwise require it.

Although sweep descriptions do not reveal the amounts, or which swap's
outputs they claim, they do reveal the block and trading pair, so their
anonymity set is considerably smaller than an ordinary shielded value
transfer. For this reason, client software should create and submit a
transaction with a sweep description immediately after observing that its
transaction with a swap description was included in a block, rather than
waiting for some future use of the new assets. This ensures that future
shielded transactions involving the new assets are not trivially linkable to
the swap.

## Privacy for Market-Takers

This design reveals only the net flow across a trading pair in each batch,
not the amounts of any individual swap. However, this provides no protection
if the batch contains a single swap, and limited protection when there are
only a few other swaps. This is likely to be an especially severe problem
until the protocol has a significant base of active users, so it is worth
examining the impact of amount disclosure and potential mitigations.

- [ ] TODO: on the client side, allow a "time preference" slider (immediate vs long duration), which spreads execution of randomized sub-amounts across multiple blocks at randomized intervals within some time horizon

- [ ] TODO: extract below into separate section about privacy on penumbra

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

[zcash_anon]: https://arxiv.org/pdf/1805.03180.pdf
[cant-build]: https://ethresear.ch/t/why-you-cant-build-a-private-uniswap-with-zkps/7754

[^1]: Thanks to Guillermo Angeris for this observation.

[^2]: Technically, this is not quite true: by itself, all that revealing the
block-level root on the authentication path proves is that the note was included
in *a* block with that root, not *the* block with that root, since the block
root binds all of the new note commitments produced in that block but does not 
explicitly bind the block height.  To fix this, we can have the chain insert a
dummy note whose note commitment is bound to the block height (e.g., by
computing the note's blinding factor as a hash of the height).  This prevents a
possible attack where an attacker who could control the exact set of note
commitments included in two different blocks at heights $h_1$ and $h_2$, both
with swaps, could submit the exact same input amounts in $h_1$ and $h_2$
(without change), and then claim both outputs at whichever executed with a
higher price.
