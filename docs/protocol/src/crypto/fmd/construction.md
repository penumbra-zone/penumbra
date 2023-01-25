# Constructing S-FMD

The original [FMD paper][fmd-paper] provides three constructions of R-FMD.  The
first two realize functionality for restricted false-positive probabilities of
the form $p = 2^{-n}$; the third supports arbitrary fractional probabilities
using a much more complex and expensive construction.  

The first two schemes, R-FMD1 and R-FMD2, are constructed similarly to a Bloom
filter: the `CreateClue` procedure encrypts a number of `1` bits, and the `Examine`
procedure uses information in a detection key to check whether some subset of
them are `1`, returning `true` if so and `false` otherwise.  The false positive
probability is controlled by extracting only a subset of the information in the
root key into the detection key, so that it can only check a subset of the bits
encoded in the clue.  We focus on R-FMD2, which provides more compact
ciphertexts and chosen-ciphertext security.

In this section, we:

* recall the construction of R-FMD2, changing notation from the paper;
* adapt R-FMD2 to S-FMD2, which provides sender FMD instead of receiver FMD;
* extend S-FMD2 to use deterministic derivation, allowing 32-byte flag and
  detection keys to support arbitrary-precision false positive probabilities;
* extend S-FMD2 to support diversified detection, allowing multiple, unlinkable
  flag keys to be detected by a single detection key (though, as this extension
  conflicts with the preceding one, we do not use it);
* summarize the resulting construction and describe how it can be integrated
  with a Sapling- or Orchard-style key hierarchy.

## The R-FMD2 construction

First, we recall the paper's construction of R-FMD2, changing from
multiplicative to additive notation and adjusting variable names to be
consistent with future extensions.

The construction supports restricted false positive values $p = 2^{-n}$ for $n
\leq \gamma$, a global parameter determining the minimum false positive rate.
$\mathbb G$ is a group of prime order $q$ and $H_1: \mathbb G^3 \rightarrow \{0,
1\}$ and $H_2: \mathbb G \times \{ 0, 1\}^\gamma \rightarrow \Z_q$ are hash
functions.

###### `R-FMD2/KeyGen`

Choose a generator $B \xleftarrow{\$} \mathbb G$.  For $i = 1,\ldots,\gamma$,
choose $x_i \xleftarrow{\$} \Z_q$ and compute $X_i = [x_i]B$.  Return the *root
key* $rk \gets (x_1, \ldots, x_\gamma)$ and the *clue key* $ck \gets (B, X_1,
\ldots, X_\gamma)$.

###### `R-FMD2/Extract`

On input $p = 2^{-n}$ and root key $rk$, parse $(x_1, \ldots, x_\gamma)
\leftarrow rk$ and return $(x_1, \ldots, x_n)$ as the *detection key*.

###### `R-FMD2/Flag`

On input $ck$, first parse $(B, X_1, \ldots, X_n) \gets ck$, then proceed as
follows:

1. Choose $r \xleftarrow{\$} \Z_q$ and compute $P \gets [r]B$.
2. Choose $z \xleftarrow{\$} \Z_q$ and compute $Q \gets [z]B$.
3. For each $i = 1,\ldots,\gamma$, compute 
    1. a key bit $k_i \gets H_1(P || [r]X_i || Q)$;
    2. a ciphertext bit $c_i \gets k_i \oplus 1$.
4. Compute $m \gets H_2(P || c_1 || \cdots || c_\gamma)$.
5. Compute $y \gets (z - m)r^{-1}$.

Return the *clue* $c \gets (P, y, c_1, \ldots, c_\gamma)$.

###### `R-FMD2/Examine`

On input $dk$, $c$, first parse $(x_1, \ldots, x_n) \gets dk$, $(P, y, c_1,
\ldots, c_\gamma) \gets c$, then proceed as follows:

1. Compute $m \gets H_2(P || c_1 || \cdots || c_\gamma)$.
2. Recompute $Q$ as $Q \gets [y]P + [m]B$.
3. For each $i = 1,\ldots,\gamma$, compute 
    1. a key bit $k_i \gets H_1(P || [x_i]P || Q)$;
    2. a plaintext bit $b_i \gets c_i \oplus k_i$.

If all plaintext bits $b_i = 1$, return $1$ (match); otherwise, return $0$.

To see that the `Test` procedure detects true positives, first note that 
since $P = [r]B$ and $y = (z-m)r^{-1}$,
$$
\begin{align}
Q &= [y]P + [m]B\\
&= [(z - m)r^{-1}][r]B +  [m]B  \\
&= [z]B,
\end{align}
$$
so the detector will recompute the $Q$ used by the sender; then, since $[r]X_i =
[r][x_i]B = [x_i]P$, the detector's $k_i \gets H_1(P || [x_i]P || Q)$ is the
same as the sender's $k_i \gets H_1(P || [r]X_i || Q)$.

On the other hand, if the clue was created with a different clue key, the
resulting $k_i$ will be random bits, giving the desired $2^{-n}$ false positive
probability.

One way to understand the components of the construction is as follows: the
message consists of $\gamma$ bits of a Bloom filter, each encrypted using hashed
ElGamal with a common nonce $r$ and nonce commitment $P = [r]B$.  The points $P,
Q$ are included in the ElGamal hash, ensuring that any change to either will
result in a random bit on decryption.   The pair $(Q, y)$ act as a public key
and signature for a one-time signature on the ciphertext as follows: the sender
constructs the point $Q$ as the output of a chameleon hash with basis $(P, B)$
on input $(0, z)$, then uses the hash trapdoor (i.e., their knowledge of the
discrete log relation $P = [r]B$) to compute a collision $(y, m)$ involving $m$,
a hash of the rest of the ciphertext. The collision $y$ acts as a one-time
signature with public key $Q$.

The fact that the generator $B$ was selected at random in each key generation is
not used by the construction and doesn't seem to play a role in the security
analysis; in fact, the security proof in Appendix F has the security game
operate with a common generator.  In what follows, we instead assume that $B$ is
a global parameter.

## From R-FMD2 to S-FMD2

Changing this construction to the S-FMD model is fairly straightforward: rather
than having the sender encrypt $\gamma$ bits of the Bloom filter and only allow
the detector to decrypt $n \leq \gamma$ of them, we have the sender only encrypt
$n \leq \gamma$ bits of the Bloom filter and allow the detector to decrypt all
potential bits.  As noted in the [previous
section](./sender-receiver.md#sender-fmd), this means that in the S-FMD context,
there is no separation of capability between the root key and the detection key.
For this reason, we skip the `Extract` algorithm and (conceptually) merge the
root key into the detection key.

The S-FMD2 construction then works as follows:

###### `S-FMD2/KeyGen`

For $i = 1,\ldots,\gamma$, choose $x_i \xleftarrow{\$} \Z_q$ and compute $X_i =
[x_i]B$.  Return the *detection key* $dk \gets (x_1, \ldots, x_\gamma)$ and the
*clue key* $ck \gets (X_1, \ldots, X_\gamma)$.

###### `S-FMD2/CreateClue`

On input clue key $ck$, first parse $(X_1, \ldots, X_n) \gets ck$, then proceed
as follows:

1. Choose $r \xleftarrow{\$} \Z_q$ and compute $P \gets [r]B$.
2. Choose $z \xleftarrow{\$} \Z_q$ and compute $Q \gets [z]B$.
3. For each $i = 1,\ldots,n$, compute 
    1. a key bit $k_i \gets H_1(P || [r]X_i || Q)$;
    2. a ciphertext bit $c_i \gets k_i \oplus 1$.
4. Compute $m \gets H_2(P || n || c_1 || \cdots || c_n)$.
5. Compute $y \gets (z - m)r^{-1}$.

Return the *clue* $c \gets (P, y, n, c_1, \ldots, c_n)$.  (We include $n$
explicitly rather than have it specified implicitly by $c_n$ to reduce the risk
of implementation confusion).

###### `S-FMD2/Examine`

On input detection key $dk$ and clue $c$, first parse $(x_1, \ldots, x_\gamma)
\gets dk$ and $(P, y, n, c_1, \ldots, c_n) \gets c$, then proceed as follows:

1. Compute $m \gets H_2(P || n || c_1 || \cdots || c_n)$.
2. Recompute $Q$ as $Q \gets [y]P + [m]B$.
3. For each $i = 1,\ldots,n$, compute 
    1. a key bit $k_i \gets H_1(P || [x_i]P || Q)$;
    2. a plaintext bit $b_i \gets c_i \oplus k_i$.

If all plaintext bits $b_i = 1$, return $1$ (match); otherwise, return $0$.

The value of $n$ is included in the ciphertext hash to ensure that the encoding
of the ciphertext bits is non-malleable.  Otherwise, the construction is identical.

## Compact clue and detection keys

One obstacle to FMD integration is the size of the clue keys.  Clue keys are
required to create clues, so senders who wish to create clues need to obtain the
receiver's clue key.  Ideally, the clue keys would be bundled with other address
data, so that clues can be included with all messages, rather than being an
opt-in mechanism with a much more limited anonymity set.

However, the size of clue keys makes this difficult.  A clue key supporting
false positive probabilities down to $2^{-\gamma}$ requires $\gamma$ group
elements; for instance, supporting probabilities down to $2^{-14} = 1/16384$
with a 256-bit group requires 448-byte flag keys.  It would be much more
convenient to have smaller keys.

One way to do this is to use a deterministic key derivation mechanism similar to
[BIP32] to derive a sequence of child keypairs from a single parent keypair, and
use that sequence as the components of a flag / detection keypair.  To do this,
we define a hash function $H_4 : \mathbb G \times \mathbb Z \rightarrow \mathbb
Z_q$.  Then given a parent keypair $(x, X = [x]B)$, child keys can be derived as 
$$
\begin{align}
x_i &\gets x + H_4(X || i) \\
X_i &\gets X + [H_4(X || i)] B
\end{align}
$$
with $X_i = [x_i]B$.  Unlike BIP32, there is only one sequence of keypairs, so
there is no need for a chain code to namespace different levels of derivation,
although of course the hash function $H_4$ should be domain-separated.

Applying this to S-FMD2, the scalar $x$ becomes the *detection key*, and the
point $X$ becomes the *clue key*.  The former detection key is renamed the
*expanded detection key* $(x_1, \ldots, x_\gamma)$, and the former clue key is
renamed the *expanded clue key* $(X_1, \ldots, X_\gamma)$; these are derived
from the detection and clue keys respectively.

In addition, it is no longer necessary to select the minimum false
positive probability as a global parameter $\gamma$ prior to key generation, as
the compact keys can be extended to arbitrary length (and thus arbitrary false
positive probabilities).

Deterministic derivation is only possible in the S-FMD setting, not the R-FMD
setting, because S-FMD does not require any separation of capability between the
component keys used for each bit of the Bloom filter.  Public deterministic
derivation does not allow separation of capability: given $X$ and any child
secret $x_i$, it's possible to recover the parent secret $x$ and therefore the
entire subtree of secret keys.  Thus, it cannot be used for R-FMD, where the
detection key is created by disclosing only some of the bits of the root key.

## Diversified detection: from S-FMD2 to S-FMD2-d

The Sapling design provides viewing keys that support *diversified addresses*: a
collection of publicly unlinkable addresses whose activity can be scanned by a
common viewing key.  For integration with Sapling, as well as other
applications, it would be useful to support *diversified detection*: a
collection of publicly unlinkable *diversified clue keys* that share a common
detection key.  This collection is indexed by data called a *diversifier*; each
choice of diversifier gives a different diversified clue key corresponding to
the same detection key.

In the Sapling design, diversified addresses are implemented by selecting the
basepoint of the key agreement protocol as the output of a group-valued hash
function whose input is the diversifier.  Because the key agreement mechanism
only makes use of the secret key (the incoming viewing key) and the
counterparty's ephemeral public key, decryption is independent of the basepoint,
and hence the choice of diversifier, allowing a common viewing key to decrypt
messages sent to any member of its family of diversified transmission (public)
keys.

A similar approach can be applied to S-FMD2, but some modifications to the tag
mechanism are required to avoid use of the basepoint in the `Test` procedure.
**Unfortunately, this extension is not compatible with compact clue and
detection keys, so although it is presented here for posterity, it is not used
in Penumbra**.

Let $\mathcal D$ be the set of diversifiers, and let $H_3 : \mathcal D
\rightarrow \mathbb G$ be a group-valued hash function.  At a high level, our
goal is to replace use of a common basepoint $B$ with a *diversified basepoint*
$B_d = H_3(d)$ for some $d \in \mathcal D$.  

Our goal is to have diversified clue keys, so we first replace $B$ by $B_d$ in
`KeyGen`, so that $X_i = [x_i]B_d$ and the components of the clue key are
parameterized by the diversifier $d$.  The sender will compute the key bits as
$$
k_i \gets H_1( P || [r]X_i || Q),
$$
and the receiver computes them as
$$
k_i \gets H_1( P || [x_i]P || Q).
$$
If $P = [r]B_d$ instead of $P = [r]B$, then $[x_i]P = [r]X_i$ and the middle
component will match.

But then the sender needs to construct $Q$ as $Q \gets [z]B_d$ in order to have
a known discrete log relation between $P$ and $Q$.  This is a problem, because
$Q$ is recomputed by the receiver, but if the receiver must recompute it as $Q
\gets [y]P + [m]B_d$, detection is bound to the choice of diversifier, and we
need detection to be independent of the choice of diversifier.

The root of this problem is that the chameleon hash uses basis $(P, B)$, so the
receiver's computation involves the basepoint $B$.  To avoid it, we can use a
basis $(P_1, P_2)$, where $P_i \gets [r_i]B$.  $P_1$ is used in place of $P$,
but $Q$ is computed as $[z]P_2$, i.e., as the chameleon hash $\langle (0, z) ,
(P_1, P_2) \rangle$.  The sender's collision then becomes $y \gets (z - m) \frac
{r_2} {r_1}$, allowing the detector to recompute $Q$ as
$$
\begin{align}
Q \gets & [y]P_1 + [m]P_2 \\
=& \left[(z - m)\frac {r_2} {r_1}\right][r_1]B +  [m]P_2  \\
=& \left[z - m\right]P_2 +  [m]P_2  \\
=& [z]P_2.
\end{align}
$$
The point $P_2$ must be included in the clue, increasing its size,
but detection no longer involves use of a basepoint (diversified or otherwise).

Concretely, this mechanism works as follows, splitting out generation of clue
keys (diversifier-dependent) from generation of detection keys:

###### `S-FMD2-d/KeyGen`

For $i = 1,\ldots,\gamma$, choose $x_i \xleftarrow{\$} \Z_q$, and
return the *detection key* $dk \gets (x_1, \ldots, x_\gamma)$.

###### `S-FMD2-d/Diversify`

On input detection key $dk$ and diversifier $d$, first parse $(x_1, \ldots,
x_\gamma) \gets dk$.  Then compute the diversified base $B_d \gets H_3(d)$, and
use it to compute $X_i = [x_i]B_d$.  Return the *diversified clue key* $ck_d
\gets (X_1, \ldots, X_\gamma)$.

###### `S-FMD2-d/CreateClue`

On input diversified clue key $ck_d$ and diversifier $d$, first parse $(X_1, \ldots, X_n)
\gets ck_d$, then proceed as follows:

1. Compute the diversified base $B_d \gets H_3(d)$.
1. Choose $r_1 \xleftarrow{\$} \Z_q$ and compute $P_1 \gets [r]B_d$.
1. Choose $r_2 \xleftarrow{\$} \Z_q$ and compute $P_2 \gets [r]B_d$.
2. Choose $z \xleftarrow{\$} \Z_q$ and compute $Q \gets [z]P_2$.
3. For each $i = 1,\ldots,n$, compute 
    1. a key bit $k_i \gets H_1(P_1 || P_2 || [r]X_i || Q)$;
    2. a ciphertext bit $c_i \gets k_i \oplus 1$.
4. Compute $m \gets H_2(P_1 || P_2 || n || c_1 || \cdots || c_n)$.
5. Compute $y \gets (z - m) \frac {r_2} {r_1}$.

Return the *clue* $c \gets (P_1, P_2, y, n, c_1, \ldots, c_n)$.

###### `S-FMD2-d/Examine`

On input detection key $dk$ and clue $c$, first parse $(x_1, \ldots,
x_\gamma) \gets dk$ and $(P_1, P_2, y, n, c_1, \ldots, c_n) \gets c$, then proceed as
follows:

1. Compute $m \gets H_2(P_1 || P_2 || n || c_1 || \cdots || c_n)$.
2. Recompute $Q$ as $Q \gets [y]P_1 + [m]P_2$.
3. For each $i = 1,\ldots,n$, compute 
    1. a key bit $k_i \gets H_1(P_1 || P_2 || [x_i]P || Q)$;
    2. a plaintext bit $b_i \gets c_i \oplus k_i$.

If all plaintext bits $b_i = 1$, return $1$ (match); otherwise, return $0$.

Unfortunately, this extension does not seem to be possible to combine with the
compact clue and detection keys extension described above.  The detection key
components $x_i$ must be derived from the parent secret and (the hash of) public
data, but diversified detection requires that different public data (clue keys)
have the same detection key components.

Of the two extensions, compact clue keys are much more important, because they
allow clue keys to be included in an address, and this allows message detection
to be a default behavior of the system, rather than an opt-in behavior of a
select few users.  

[fmd-paper]: https://eprint.iacr.org/2021/089
[BIP32]: https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki
[ZIP32]: https://zips.z.cash/zip-0032
