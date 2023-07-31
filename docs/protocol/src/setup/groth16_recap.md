# Groth16 Recap

In this chapter, we'll be looking at the internals of how Groth16's CRS works,
so it might be useful to very briefly describe how the system works.
For another succinct resource on Groth16, see [Kurt Pan's Notes](https://site.kurtpan.pro/notes/groth16.html).

## Constraint System

We work over a circuit taking $n$ inputs, which we write as $[z_i \in \mathbb{F} \mid i \in [n]]$.
There is an index $s \in [n]$ such that the $[z_i \mid i < s]$ are the public inputs, and the $[z_i \mid i \geq s]$
are the private inputs.

The constraints of our circuit are encoded as a list of polynomials $[u_i, v_i, w_i \in \mathbb{F}[X] \mid i \in [n]]$ of degree $d - 1$,
along with a polynomial $t(X)$, of degree $d$.
The inputs satisfy the circuit when the following equation holds:
$$
\left(\sum_i z_i u_i(X)\right)\left(\sum_i z_i v_i(X)\right) \equiv \sum_i z_i w_i(X) \mod t(X)
$$
(Note that saying that $f(X) \equiv g(X) \mod t(X)$ is equivalent
to saying that there exists an $h(X)$, of degree at most $d - 2$,
such that $f(X) - g(X) = h(X)t(X)$).

The goal of a proof is to prove knowledge of the $[z_i \mid i \geq s]$
satisfying these constraints, without revealing information about what
their values might be.

## CRS

The CRS involves generating private parameters, and then performing
some calculations to derive public elements, which are then used
for creating and verifying proofs.
It's important that these private parameters are destroyed
after being used to derive the public parameters.

As a shorthand, we define the following polynomial:
$$
p^{\alpha, \beta}_i(X) := \beta u_i(X) + \alpha v_i(X) + w_i(X)
$$
which gets used many times in the CRS itself.

The private parameters consist of randomly sampled scalars:
$$
\alpha, \beta, \gamma, \delta, x
$$

The public parameters are then derived from these private ones,
producing the following list of elements:

- $[\alpha]_1, [\beta]_1, [\delta]_1$

- $\displaystyle \left[x^i\right]_1\quad (i \in [0, \ldots, d - 1])$

- $\displaystyle \left[\frac{1}{\gamma} p^{\alpha, \beta}_i(x)\right]_1\quad (i < s)$

- $\displaystyle \left[\frac{1}{\delta} p^{\alpha, \beta}_i(x)\right]_1\quad (i \geq s)$

- $\displaystyle \left[\frac{t(x)}{\delta} x^i \right]_1\quad (i \in [0, \ldots, d - 2])$

- $[\beta]_2, [\gamma]_2, [\delta]_2$

- $\displaystyle \left[x^i\right]_2\quad (i \in [0, \ldots, d - 1])$

(Note that given $[\rho \cdot x^i]_\sigma$ for $i$ up to a given degree $d$,
we can then compute $[\rho f(x)]_\sigma$, for any polynomial
$f$ of degree up to $d$, since this element is a linear combination
of these monomial elements.
We write $\rho f([x]_\sigma)$ to denote this process.)

## Proving and Verifying

Next we describe the proving and verification equations:

**Proving**

A proof $\pi$ consists of three group elements: $A, C \in \mathbb{G}_1$,
and $B \in \mathbb{G}_2$.

The proof requires the generation of two fresh random scalars $r$ and $s$,
which are then used to compute the following elements:

- $\displaystyle A := [\alpha]_1 + \sum_i z_i \cdot u_i([x]_1) + r \cdot [\delta]_1$

- $\displaystyle B := [\beta]_2 + \sum_i z_i \cdot v_i([x]_2) + s \cdot [\delta]_2$
- $\displaystyle \hat{B} := [\beta]_1 + \sum_i z_i \cdot v_i([x]_1) + s \cdot [\delta]_1$

- $\displaystyle C := \sum_{i \geq s} z_i \cdot \left[\frac{1}{\delta} p_i^{\alpha, \beta}(x)\right]_1 + \frac{t(x)}{\delta}h([x]_1) + s \cdot A + r \cdot \hat{B} - rs\delta$

Finally, the proof is returned as $(A, B, C)$.

**Verification**

Given a proof $\pi = (A, B, C)$, verification checks:
$$
A \odot B \overset{?}{=} [\alpha]_1 \odot [\beta]_2 + \sum_{i < s} z_i \cdot \left[\frac{1}{\gamma}p^{\alpha, \beta}_i(x)\right]_1 \odot [\gamma]_2 + C \odot [\delta]_2
$$

## Modified CRS

[BGM17](https://eprint.iacr.org/2017/1050) (Section 6) proposed a slightly modified
CRS, adding extra elements, in order to simplify the setup ceremony.
They also proved that adding these elements did not affect the security
(specifically, *knowledge soundness*) of the scheme.

The CRS becomes:

- $[\alpha]_1, [\beta]_1, [\delta]_1$

- $\displaystyle \left[x^i\right]_1\quad \textcolor{DodgerBlue}{(i \in [0, \ldots, 2d - 2])}$

- $\textcolor{DodgerBlue}{\displaystyle \left[\alpha x^i\right]_1\quad (i \in [0, \ldots, d - 1])}$

- $\textcolor{DodgerBlue}{\displaystyle \left[\beta x^i\right]_1\quad (i \in [0, \ldots, d - 1])}$

- $\displaystyle \left[\frac{1}{\delta} p^{\alpha, \beta}_i(x)\right]_1\quad (i \geq s)$

- $\displaystyle \left[\frac{t(x)}{\delta} x^i \right]_1\quad (i \in [0, \ldots, d - 2])$

- $[\beta]_2, [\delta]_2$

- $\displaystyle \left[x^i\right]_2\quad (i \in [0, \ldots, d - 1])$

The main change is that $\gamma$ has been removed,
and that we now have access to higher degrees of $x^i$ in $\mathbb{G}_1$,
along with direct access to $\alpha x^i$ and $\beta x^i$.
