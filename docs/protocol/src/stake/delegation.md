# Delegation

The delegation process bonds stake to a validator, converting stake `PEN` to
delegation tokens `dPEN`. Delegations may be performed in any block, but only
take effect in the next epoch.

Delegations are accomplished by creating a transaction with a
`Delegate` description.  This specifies a validator $v$,
consumes $x$ `PEN` from the transaction's balance, produces a new shielded note
with $$y = x / \psi_v(e)$$ `dPEN` associated to that validator, and includes an
encryption $\operatorname{Enc}_D(y)$ of the delegation amount to the validators'
shared decryption key $D$.  Here $e$ is the index of the next epoch, when the
delegation will take effect. 

In the last block of epoch $e-1$, the validators sum the encrypted bonding
amounts $\operatorname{Enc}_D(y_v^{i})$ from all delegate descriptions for
validator $v$ in the entire epoch to obtain an encryption of the total
delegation amount $\operatorname{Enc}_D(\sum_i y_v^{(i)})$ and decrypt to obtain
the total delegation amount $y_v = \sum_i y_v^{(i)}$ without revealing any
individual transaction's delegation amount $y_v^{i}$.  These total amounts are
used to update the size of each validator's delegation pool for the next epoch.

Revealing only the total inflows to the delegation pool in each epoch helps
avoid linkability.  For instance, if the size of each individual transaction's
delegation were revealed, a delegation of size $a$ followed by an undelegation
of size $b$ could be correlated if an observer notices that there are some
epochs $e_1, e_2$ so that $$a \frac {\psi_v(e_2)}{ \psi_v(e_1)} = b.$$

This risk is still present when only the total amount -- the minimum disclosure
required for consensus -- is revealed, because there may be few (or no) other
delegations to the same validator in the same epoch. Some care should be taken
in client implementations and user behavior to mitigate the effects of this
information disclosure, e.g., by splitting delegations into multiple
transactions in different epochs involving randomized sub-portions of the stake.
However, the best mitigation would simply be to have many users.
