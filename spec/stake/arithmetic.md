# Fixed-Point Arithmetic for Rate Computation

To compute base reward rate, base exchange rate, validator-specific exchange
rates, and total validator voting power, we need to carefully perform
arithmetic to avoid issues with precision and rounding. We use explicitly
specified fixed-precision arithmetic for this, with a precision of 8 digits.
This allows outputs to fit in a u64, with all products fitting in the output
and plenty of headroom for additions.

All integer values should be interpreted as unsigned 64-bit integers, with the
exception of the validator's commission rate, which is a `u16` specified in
terms of basis points. All integer values, with the exception of the
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


## Table of Contents

* $\mathtt r_{e}$: the fixed-point representation of the base rate at epoch $e$.
* $\mathtt \psi(e)$: the fixed-point representation of the base exchange rate at epoch $e$.
* $s_{i, e}$: the $10^4$ denominated fixed-point representation of the funding rate of the validator's funding stream at index $i$ and epoch $e$.
* $c_{v,e}$: the sum of the validator's commission rates, which are specified as a fixed point integer with an implicit $10^4$ denominator.
* $\mathtt r_{v,e}$: the fixed-point representation of the validator-specific reward rate for validator $v$ at epoch $e$.
* $\mathtt \psi_v(e)$: the fixed-point representation of the validator-specific exchange rate for validator $v$ at epoch $e$.
* $\mathtt \theta_v(e)$: the fixed-point representation of the validator's voting power adjustment function for validator $v$ at epoch $e$.


## Base Reward Rate

The base reward is an input to the protocol, and the exact details of how this
base rate $r_{e}$ is determined is not yet decided. For now, we can assume it is
derived from the block header.


## Base Exchange Rate

The base exchange rate, $\psi(e)$, can be safely computed as follows:

$$\mathtt \psi(e) = \left\lfloor \frac {\mathtt \psi(e-1) (10^8 + \mathtt r_e)} {10^8} \right\rfloor$$

## Commission Rate from Funding Streams

To compute the validator's commission rate from its set of funding streams, the following calculation can be performed:

$$c_{v,e} = \sum_{0 \leq i < n} (s_{i,e})$$

Where $s$ is the set of rates defined by the funding streams, $s_{i,e}$ being
the ith rate at epoch $e$.


## Validator Reward Rate

To compute the validator's base reward rate, we compute the following:

$$\mathtt r_{v,e} = \left\lfloor \frac{(10^8 - (c_{v,e} \cdot 10^4))\mathtt r_e}{10^8} \right\rfloor$$

## Validator Exchange Rate

To compute the validator's exchange rate, we adapt the formula from the staking
specification for our fixed-point arithmetic scheme like so:

$$\mathtt \psi_v(e) = \left\lfloor \frac {\psi_v(e-1) (10^8 + \mathtt r_{v,e})}{10^8} \right\rfloor$$

## Validator Voting Power Adjustment

Finally, to compute the validator's voting power adjustment function, simply take:

$$\theta_v(e) = \lfloor \frac{\psi_v{e}*10^8}{\psi{e}} \rfloor$$