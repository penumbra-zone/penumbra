# Groth16 Setup

The proving system we use, [Groth16](https://eprint.iacr.org/2016/260),
requires a per-circuit trusted setup: each circuit requires some public
parameters, called a *CRS* (common reference string), and generating
these public parameters involves the creation of *private* parameters.
Knowing these private parameters would allow for forging proofs;
ensuring their destruction is paramount.

To that end, systems don't simply generate these parameters,
but instead go through a *setup ceremony*, involving many participants,
such that the setup is secure so long as *at least one* participant
destroys the private parameters they've used to contribute to the ceremony.

This chapter describes the technical aspects of a ceremony
setting up these parameters, based off of
[KMSV21](https://eprint.iacr.org/2021/219) (Snarky Ceremonies),
itself based off of [BGM17](https://eprint.iacr.org/2017/1050).
We organize the information herein as follows:
- The [Groth16 Recap](./setup/groth16_recap.md) section provides a brief recap of how the formulas and CRS structure for Groth16 work.
- The [Discrete Logarithm Proofs](./setup/dlog_proofs.md) section describes a simple discrete logarithm proof we need for setup contributions.
- The [Contributions](./setup/contributions.md) section describes
the crux of the ceremony: how users make contributions to the parameters.

## Notation

We work with a triplet of groups $\mathbb{G}_1, \mathbb{G}_2, \mathbb{G}_T$, with an associated field of scalars $\mathbb{F}$, equipped with a pairing operation:
$$
\odot : \mathbb{G}_1 \times \mathbb{G}_2 \to \mathbb{G}_T
$$
We also have designated generator elements $G_1, G_2, G_T$
for each of the respective groups, with $G_T = G_1 \odot G_2$.
In the case of Penumbra, the concrete groups used are from [BLS12-377](https://neuromancer.sk/std/bls/BLS12-377).

We take the convention that lowercase letters (e.g. $x, a$)
are taken to be scalars in $\mathbb{F}$,
and uppercase letters (e.g. $X, A$) are taken to be elements
of $\mathbb{G}_1$, $\mathbb{G}_2$, or $\mathbb{G}_T$.

For $i \in \{1, 2, T\}$, we use the shorthand:
$$
[x]_i := x \cdot G_i
$$
for scalar multiplication using one of the designated
generators.

All of the groups we work with being commutative, we use
additive notation consistently.

As an example of this use of additive notation,
consider the following equation:
$$
([a]_1 + [b]_1) \odot [c]_2 = [ac + bc]_T
$$

As a somewhat unfortunate conflict of notation, we use $[n]$ to denote
the set $\{1, \ldots, n\}$, and ${[s_i \mid i \in S]}$ to denote
a list of elements, with $i$ ranging over a set $S$.
