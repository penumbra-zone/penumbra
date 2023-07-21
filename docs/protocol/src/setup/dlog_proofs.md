# Discrete Logarithm Proofs

One gadget we'll need is a way to have ZK proofs for the following relation:
$$
\{(X, Y; w) \mid X = w \cdot G_1 \land Y = w \cdot G_2\}
$$
(with $w$ kept secret).

In other words, one needs to prove that $X$ and $Y$
have the same discrete logarithm,
relative to the generators of $\mathbb{G}_1$ and $\mathbb{G}_2$,
respectively.

The notation we'll use here is
$$
\pi \gets P_{\text{DL}}(\text{ctx}, X, Y; w)
$$
for generating a proof (with some arbitrary context string $\text{ctx}$), using the public statement $(X, Y)$ and the witness $w$,
as well as:
$$
V_{\text{DL}}(\text{ctx}, X, Y, \pi)
$$
for verifying that proof, using the same context and statement.

The proof should fail to verify if the context or statement
don't match, or if the proof wasn't produced correctly, of course.

## How They Work

(You can safely skip this part, if you don't actually
need to know how they work).

These proofs make use of the pairing operation,
as well as a hash function
$$
H : \{0, 1\}^* \times \mathbb{G}_1 \times \mathbb{G}_2 \to \mathbb{G}_1
$$
modelled as a random oracle,
for which one does *not* learn the discrete logarithm
of the output.

**Proving**

$$
P_{\text{DL}}(\text{ctx}, X, Y; w) := w \cdot H(\text{ctx}, X, Y)
$$

**Verification**

$$
V_{\text{DL}}(\text{ctx}, X, Y, \pi) :=
\begin{aligned}
X \odot G_2 &\overset{?}{=} G_1 \odot Y \quad \land\cr
\pi \odot G_2 &\overset{?}{=} 
H(\text{ctx}, X, Y) \odot Y 
\end{aligned}
$$