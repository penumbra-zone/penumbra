# Discrete Logarithm Proofs

One gadget we'll need is a way to have ZK proofs for the following relation:
$$
\{(W, X; w) \mid W = w \cdot X\}
$$
(with $w$ kept secret).

In other words, one needs to prove knowledge of the discrete logarithm
of $W$ with regards to $X$.

The notation we'll use here is
$$
\pi \gets P_{\text{DL}}(\text{ctx}, W, X; w)
$$
for generating a proof (with some arbitrary context string $\text{ctx}$), using the public statement $(W, X)$ and the witness $w$,
as well as:
$$
V_{\text{DL}}(\text{ctx}, W, X, \pi)
$$
for verifying that proof, using the same context and statement.

The proof should fail to verify if the context or statement
don't match, or if the proof wasn't produced correctly, of course.

## How They Work

(You can safely skip this part, if you don't actually
need to know how they work).

These are standard Maurer / Schnorr-esque proofs, making use of
a hash function
$$
H : \{0, 1\}^* \times \mathbb{G}^3 \to \mathbb{F}
$$
modelled as a random oracle.

**Proving**

$$
\begin{aligned}
&P_{\text{DL}}(\text{ctx}, X, Y; w) :=\cr
&\quad k \xleftarrow{\$} \mathbb{F}\cr
&\quad K \gets k \cdot Y\cr
&\quad e \gets H(\text{ctx}, (X, Y, K))\cr
&\quad (K, k + e \cdot x)\cr
\end{aligned}
$$

**Verification**

$$
V_{\text{DL}}(\text{ctx}, X, Y, \pi = (K, s)) := s \cdot Y \overset{?}{=} K + H(\text{ctx}, (X, Y, K)) \cdot X
$$
