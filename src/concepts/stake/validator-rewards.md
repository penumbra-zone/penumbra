# Validator Rewards and Fees

Validators declare a commission percentage $c_{v,e} \in [0, 1]$, which determines the spread between the base reward rate $r_e$ and the reward rate for their delegators $r_{v,e} = (1 - c_{v,e})r_e$, or equivalently $r_e = r_{v,e} + c_{v,e}r_e$.

Validator rewards are distributed in the first block of each epoch.  In epoch $e$, a validator $v$ whose delegation pool has size $y_v$ `PENb` receives a commission of size $$y_v c_{v,e} r_e \psi(e-1)$$ `PEN`, issued to the validator's address.

To see why this is the reward amount, suppose validator $v$ has a delegation pool of size $y_v$ `PENb`. In epoch $e-1$, the value of the pool is $y_v \psi_v(e-1)$ `PEN`.  In epoch $e$, the base reward rate $r_{e}$ causes the value of the pool to increase to
$$
(1 + r_e)y_v \psi_v(e-1).
$$
Splitting $r_e$ as $r_e = r_{v,e} + c_{v,e}r_e$, this becomes
$$ y_v (1 + r_{v,e}) \psi_v(e-1) + c_{v,e}r_e y_v \psi_v(e-1). $$  

The value in the first term, $y_v (1 + r_{v,e}) \psi_v(e-1) $,
corresponds to the $r_{v,e}$ portion, and accrues to the delegators. Since $(1 + r_{v,e})\psi_v(e-1) = \psi_v(e)$, this is exactly $y_v \psi_v(e)$, the new `PEN`-denominated value of the delegation pool.

The value in the second term, $c_{v,e}r_e y_v \psi_v(e-1)$, corresponds to the $c_{v,e}r_e$ portion, and accrues to the validator as commission.  Validators can self-delegate the resulting `PEN` or use it to fund their operating expenses.

Transaction fees are denominated in `PEN` and are burned, so that the value of the fees accrues equally to all stake holders.

## TODO

- [ ] allow transaction fees in `PENb` with appropriate discounting, but only in transactions (e.g., undelegations, voting) that otherwise reveal the flavor of `PENb`.
