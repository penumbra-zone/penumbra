# Voting Power

The size of each validator's delegation pool (i.e., the amount of `PENb` of that validator's flavor) is public information, and determines the validator's voting power.  However, these sizes cannot be used directly, because they are based on validator-specific conversion rates $\psi_v$ and are therefore incommensurate.

Voting power is calculated using the adjustment function $\theta_v(e) = \psi_v(e) / \psi(e)$, so that a validator $v$ whose delegation pool has $y_v$ `PENb` in epoch $e$ has voting power $y_v \theta_v(e)$.

The validator-specific reward rate $$r_{v,e} = r_e - c_{v,i} r_e$$ adjusts the base reward rate to account for the validator's commission.  Since $$\psi_v(e) = \prod_{0 \leq i < e} (1 + r_{v,i}),$$ and $$\psi(e) = \prod_{0 \leq i < e} (1 + r_i),$$ the adjustment function $$\theta_v(e) = \frac {\psi_v(e)}{\psi(e)} = \prod_{0 \leq i < e} \frac{ 1 + r_{v,i}}{1 + r_i}$$ accounts for the compounded effect of the validator's commission on the size of the delegation pool.
