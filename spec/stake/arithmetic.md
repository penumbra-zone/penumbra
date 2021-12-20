# Fixed-Point Arithmetic for Rate Computation

To compute base reward rate, base exchange rate, validator-specific exchange
rates, and total validator voting power, we need to carefully perform
arithmetic to avoid issues with precision and rounding. We use explicitly
specified fixed-precision arithmetic for this, with a precision of 8 digits.
This allows outputs to fit in a u64, with all products fitting in the output
and plenty of headroom for additions.

All integer values should be interpreted as unsigned 64-bit integers, with the
exception of the validator's commission rate, which is a `u16` specified in
terms of basis points (one one-hundredth of a percent, or in other words, an
implicit denominator of $10^{4}$). All integer values, with the exception of the
validator's commission rate, have an implicit denominator of $10^8$. 

Throughout this spec *representations* are referred to as $\mathtt x$, where
$\mathtt x = x \cdot 10^{8}$, and the *value* represented by representations is
$x = \mathtt x \cdot 10^{-8}$.

As an example, let's take a starting value $\mathtt x$ represented in our scheme (so,
$x \cdot 10^8$) and compute its product with $y$, also represented by our fixed point
scheme, so $y \cdot 10^8$. The product $\mathtt x \mathtt y$ is computed as 

$$\left\lfloor \frac{(x 10^8)(y 10^8)}{10^{8}} \right\rfloor = \left\lfloor \frac{\mathtt x \mathtt y}{10^{8}} \right\rfloor$$

Since both $x$ and $y$ are both representations and both have a factor of
$10^8$, computing their product includes an extra factor of 10^8 which we
divide out. All representations are `u64` in our scheme, and any fixed-point
number or product of fixed point numbers with 8 digits fits well within 64
bits.


### Summary of Notation

* $\mathtt r_{e}$: the fixed-point representation of the base rate at epoch $e$.
* $\mathtt {psi}(e)$: the fixed-point representation of the base exchange rate at epoch $e$.
* $s_{i, e}$: the funding rate of the validator's funding stream at index $i$ and epoch $e$, in basis points.
* $c_{v,e}$: the sum of the validator's commission rates, in basis points.
* $\mathtt r_{v,e}$: the fixed-point representation of the validator-specific reward rate for validator $v$ at epoch $e$.
* $\mathtt {psi}_v(e)$: the fixed-point representation of the validator-specific exchange rate for validator $v$ at epoch $e$.
* $\mathtt {theta}_v(e)$: the fixed-point representation of the validator's voting power adjustment function for validator $v$ at epoch $e$.


## Base Reward Rate

The base reward is an input to the protocol, and the exact details of how this
base rate $r_{e}$ is determined is not yet decided. For now, we can assume it is
derived from the block header.


## Base Exchange Rate

The base exchange rate, $\psi(e)$, can be safely computed as follows:

$$\mathtt {psi}(e) = \left\lfloor \frac {\mathtt {psi}(e-1) \cdot (10^8 + \mathtt r_e)} {10^8} \right\rfloor$$

## Commission Rate from Funding Streams

To compute the validator's commission rate from its set of funding streams, compute
$$c_{v,e} = \sum_{i} s_{i,e}$$
where $s_{i, e}$ is the rate of validator $v$'s $i$-th funding stream at epoch $e$.

## Validator Reward Rate

To compute the validator's base reward rate, we compute the following:

$$\mathtt r_{v,e} = \left\lfloor \frac{(10^8 - (c_{v,e} \cdot 10^4))\mathtt r_e}{10^8} \right\rfloor$$

## Validator Exchange Rate

To compute the validator's exchange rate, we use the same formula as for the
base exchange rate, but substitute the validator-specific reward rate in place
of the base reward rate:

$$\mathtt {psi}_v(e) = \left\lfloor \frac {\mathtt {psi}_v(e-1) \cdot (10^8 + \mathtt r_{v,e})}{10^8} \right\rfloor$$

## Validator Voting Power Adjustment

Finally, to compute the validator's voting power adjustment function, compute:

$$\mathtt {theta}_v(e) = \left\lfloor \frac{\mathtt {psi}_v(e) 10^8}{\mathtt {psi}(e)} \right\rfloor$$
