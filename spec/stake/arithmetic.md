# Fixed-Point Arithmetic for Rate Computation

To compute base reward rate, base exchange rate, validator-specific exchange
rates, and total validator voting power, we need to carefully perform
arithmetic to avoid issues with precision and rounding. We use explicitly
specified fixed-precision arithmetic for this, with a precision of 8 digits.
This allows outputs to fit in a u64, with all products fitting in the output
and plenty of headroom for additions.

All integer values should be interpreted as unsigned 64-bit integers, with the
exception of the validator's commission rate, which is a `u16` specified in
terms of basis points.

## Base Reward Rate

The base reward is an input to the protocol, and the exact details of how this
base rate $r_{e}$ is determined is not yet decided. For now, we can assume it is
derived from the block header. $r_{e}$ is a fixed-precision `u64` integer with
8 digits of precision.


## Base Exchange Rate

The base exchange rate, $\psi(e)$, can be safely computed as follows:

$$\psi(e) = \prod_{0 \leq i < e} \lfloor \frac{(1e8 + r_i)}{1e8} \rfloor $$


## Commission Rate from Funding Streams

To compute the validator's commission rate from its set of funding streams, the following calculation can be performed:

$$c_{v,e} = \sum_{0 \leq i < n} (s_{i,e})$$

Where $s$ is the set of rates defined by the funding streams, $s_{i,e}$ being
the ith rate at epoch $e$.  $s_i$ should be a fixed-precision `u16` integer
type, specified in terms of [basis
points](https://en.wikipedia.org/wiki/Basis_point). The resulting $c_{v,e}$ is
defined in terms of basis points (i.e., four digits of precision). 


## Validator Reward Rate

To compute the validator's base reward rate, we compute the following:

$$r_{v,e} = \lfloor \frac{(1e8 - (c_{v,e}*1e4))r_e}{1e8} \rfloor$$

## Validator Exchange Rate

To compute the validator's exchange rate, we adapt the formula from the staking
specification for our fixed-point arithmetic scheme like so:

$$\psi_v(e) = \prod_{0 \leq i < e} \lfloor \frac{(1e8 + r_{v,i})}{1e8} \rfloor$$

## Validator Voting Power Adjustment

Finally, to compute the validator's voting power adjustment function, simply take:

$$\theta_v(e) = \lfloor \frac{\psi_v{e}*1e8}{\psi{e}} \rfloor$$