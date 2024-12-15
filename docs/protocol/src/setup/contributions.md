# Contributions

In this section, we describe the contributions that make up a setup ceremony
in more detail.
We describe:
- the high level idea behind the ceremony,
- what contributions look like, and how to check their correctness,
- how to check the correctness of the setup as a whole.

## High Level Overview

We break the CRS described [previously](./groth16_recap.md) into two parts:

First, we have:

- $[\alpha]_1, [\beta]_1, [\beta]_2$

- $\displaystyle \left[x^i\right]_1\quad (i \in [0, \ldots, 2d - 2])$

- $\displaystyle \left[x^i\right]_2\quad (i \in [0, \ldots, d - 1])$

- $\displaystyle \left[\alpha x^i\right]_1\quad (i \in [0, \ldots, d - 1])$

- $\displaystyle \left[\beta x^i\right]_1\quad (i \in [0, \ldots, d - 1])$


Second, we have:

- $[\delta]_1, [\delta]_2$

- $\displaystyle \left[\frac{1}{\delta} p^{\alpha, \beta}_i(x)\right]_1\quad (i \geq s)$

- $\displaystyle \left[\frac{t(x)}{\delta} x^i \right]_1\quad (i \in [0, \ldots, d - 2])$

We split the ceremony into two phases, to calculate the first and second
part of the CRS, respectively.
The general idea in each ceremony is that the secret values of interest
(e.g. $\alpha, x$ etc.) are shared multiplicatively, as
$\alpha_1 \cdot \alpha_2 \ldots$, with each party having
one of the shares.
Because of this structure, given the current value of the CRS elements
in a given phase, it's possible for a new party to add their contribution.
For example, in the first phase, one can multiply each element
by some combination $\alpha^{d_1} \cdot \beta^{d_2} \cdot x^{d_3}$,
depending on the element,
to get a new CRS element.

Each contribution will come with a proof of knowledge for the new secret
values contributed, which can also partially attest to how these secret
values were used.
However, this is not enough to guarantee that the resulting elements
are a valid CRS: for this, we have a consistency check
allowing us to check that the elements in a given phase
have the correct internal structure.

Each party can thus contribute one after the other, until
enough contributions have been gathered through that phase.

In order to link phase 1 and phase 2,
we use the fact that with $\delta = 1$, the CRS elements of phase 2
are linear combinations of those in phase 1.
If we consider $t(x)x^i$, with $i$ up to $d - 2$,
the largest monomial we'll find is $2d - 2$, since $t$ has degree at most $d$.
In the first phase, we calculated these powers of $x$, and so can
calculate these values by linear combination.
We can do the same for:
$$
p_i^{\alpha, \beta}(x) = \alpha u_i(x) + \beta v_i(x) + w_i(x)
$$
since we have access to $\alpha x^i$ and $\beta x^i$ for sufficiently
high degrees.

## Phase 1

Assuming we have the CRS elements of phase 1, a contribution involves
fresh random scalars $\hat{\alpha}, \hat{\beta}, \hat{x}$, and produces
the following elements:

- $\hat{\alpha} \cdot [\alpha]_1, \hat{\beta} \cdot [\beta]_1, \hat{\beta} \cdot [\beta]_2$

- $\hat{x}^i \cdot [x^i]_1\quad (i \in [0, \ldots, 2d - 2])$
- $\hat{x}^i \cdot [x^i]_2\quad (i \in [0, \ldots, d - 1])$
- $\hat{\alpha}\hat{x}^i \cdot [\alpha x^i]_1\quad (i \in [0, \ldots, d - 1])$
- $\hat{\beta}\hat{x}^i \cdot [\beta x^i]_1\quad (i \in [0, \ldots, d - 1])$

Additionally, a contribution includes three proofs:

1. $\pi_1 \gets P_{\text{DL}}(\text{ctx}, \hat{\alpha} \cdot [\alpha]_1, [\alpha]_1; \hat{\alpha})$
2. $\pi_2 \gets P_{\text{DL}}(\text{ctx}, \hat{\beta} \cdot [\beta]_1, [\beta]_1; \hat{\beta})$
3. $\pi_3 \gets P_{\text{DL}}(\text{ctx}, \hat{x} \cdot [x]_1, [x]_1; \hat{x})$

### Checking Correctness

Given purported CRS elements:

- $G_{\alpha}, G_{\beta}, H_{\beta}$

- $G_{x^i}\quad (i \in [0, \ldots, 2d - 2])$

- $H_{x^i}\quad (i \in [0, \ldots, d - 1])$

- $\displaystyle G_{\alpha x^i}\quad (i \in [0, \ldots, d - 1])$

- $\displaystyle G_{\beta x^i}\quad (i \in [0, \ldots, d - 1])$

We can check their validity by ensuring the following checks hold:

1. Check that $G_\alpha, G_\beta, H_{\beta}, G_x, H_x \neq 0$ (the identity element in the respective groups).
2. Check that $G_\beta \odot [1]_2 = [1]_1 \odot H_\beta$.
3. Check that $G_{x^i} \odot [1]_2 = [1]_1 \odot H_{x^i} \quad (\forall i \in [0, \ldots, d - 1])$.
4. Check that $G_{\alpha} \odot H_{x^i} = G_{\alpha x^i} \odot [1]_2 \quad (\forall i \in [0, \ldots, d- 1])$.
5. Check that $G_{\beta} \odot H_{x^i} = G_{\beta x^i} \odot [1]_2 \quad (\forall i \in [0, \ldots, d- 1])$.
6. Check that $G_{x^i} \odot H_x = G_{x^{i + 1}} \odot [1]_2 \quad (\forall i \in [0, \ldots, 2d - 3])$.

### Checking Linkedness

To check that CRS elements $G'_{\ldots}$ build off a prior CRS $G_{\ldots}$,
one checks the included discrete logarithm proofs $\pi_1, \pi_2, \pi_3$, via:

1. $V_{\text{DL}}(\text{ctx}, G'_\alpha, G_\alpha, \pi_1)$
2. $V_{\text{DL}}(\text{ctx}, G'_\beta, G_\beta, \pi_2)$
3. $V_{\text{DL}}(\text{ctx}, G'_x, G_x, \pi_3)$

## Phase 2

Assuming we have the CRS elements of phase 2, a contribution involves
a fresh random scalar $\hat{\delta}$, and produces
the following elements:

- $\hat{\delta} \cdot [\delta]_1, \hat{\delta} \cdot [\delta]_2$

- $\displaystyle \frac{1}{\hat{\delta}} \cdot \left[\frac{1}{\delta}p_i^{\alpha, \beta}(x)\right]_1\quad (i \geq s)$

- $\displaystyle \frac{1}{\hat{\delta}} \cdot \left[\frac{1}{\delta}t(x)x^i\right]_1\quad (i \in [0, \ldots, d - 2])$

Additionally, a contribution includes a proof:

$$
\pi \gets P_{\text{DL}}(\text{ctx}, \hat{\delta} \cdot [\delta]_1, [\delta]_1; \hat{\delta})
$$

### Checking Correctness 

Assume that the elements $[p_i^{\alpha, \beta}(x)]_1\ (i \geq s)$ and $[t(x) x^i]_1\ (i \in [0, \ldots, d - 2])$ are known.

Then, given purported CRS elements:

- $G_\delta, H_\delta$
- $G_{\frac{1}{\delta}p_i}\quad(i \geq s)$
- $G_{\frac{1}{\delta}t_i}\quad(i \in [0, \ldots, d - 2])$

We can check their validity by ensuring the following checks hold:

1. Check that $G_\delta, H_\delta \neq 0$ (the identity element in the respective groups).
2. Check that $G_\delta \odot [1]_2 = [1]_1 \odot H_\delta$.
3. Check that $G_{\frac{1}{\delta}p_i} \odot H_\delta = [p_i^{\alpha, \beta}]_1 \odot [1]_2\quad (\forall i \geq s)$.
4. Check that $G_{\frac{1}{\delta}t_i} \odot H_\delta = [t(x) x^i]_1 \odot [1]_2\quad (\forall i \in [0, \ldots, d - 2])$.

### Checking Linkedness

To check that CRS elements $G'_{\ldots}$ build off a prior CRS $G_{\ldots}$,
one checks the included discrete logarithm proof $\pi$, via:

$$
V_{\text{DL}}(\text{ctx}, G'_\delta, G_\delta, \pi)
$$

## Batched Pairing Checks

Very often, we need to check equations of the form:
$$
\forall i.\ A_i \odot B = C \odot D_i
$$
(this would also work if the right-hand side is of the form $D_i \odot C$, and vice versa).

This equation is equivalent to checking:
$$
\forall i.\ A_i \odot B - C \odot D_i = 0
$$
If you pick random scalars $r_i$ from a set $S$, then except with probability
$|S|^{-1}$, this is equivalent to checking:
$$
\sum_i r_i \cdot (A_i \odot B - C \odot D_i) = 0
$$
By the homomorphic properties of a pairing, this is the same as:
$$
\left(\sum_i r_i \cdot A_i\right) \odot B - C \odot \left(\sum_i r_i \cdot D_i\right)
$$

Instead of checking $2N$ pairings, we can instead perform $2$ MSMs
of size $N$, and then $2$ pairings, which is more performant.
