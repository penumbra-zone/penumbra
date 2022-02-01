# Inverse Square Roots

As in the [internet-draft], the `decaf377` functions are defined in terms of the
following function, which computes the square root of a ratio of field elements,
with the special behavior that if the input is nonsquare, it returns the square
root of a related field element, to allow reuse of the computation in the
hash-to-group setting.

Define `sqrt_ratio_zeta(N,D)` as:

- (True, $\sqrt{\frac N D}$) if $N$ and $D$ are nonzero, and $\frac N D$ is square;
- (True, $0$) if $N$ is zero;
- (False, $0$) if $D$ is zero and $N$ is non-zero;
- (False, $\sqrt{ \zeta \frac N D}$) if $N$ and $D$ are nonzero, and $\frac N D$ is nonsquare.

Since $\zeta$ is nonsquare, if $\frac N D$ is nonsquare, $\zeta \frac N D$ is
square.  Note that **unlike** the similar function in the
`ristretto255`/`decaf448` [internet-draft], this function does not make any
claims about the sign of its output.

Define `isqrt(x)` as a function that internally invokes `sqrt_ratio_zeta` and computes the inverse square root of a field element $x$:

- (True, $0$) if $x$ is zero;
- (True, $1/\sqrt{x}$) if $x$ is nonzero, and $x$ is square;
- (False, $1/\sqrt{ \zeta x}$) if $x$ is nonzero, and $x$ is nonsquare.

To compute `sqrt_ratio_zeta` we use a table-based method adapted from [Sarkar 2020] and [zcash-pasta], which is described in the remainder of this section.

## Constants

We set $n \gt 1$ (the 2-adicity of the field) and $m$ odd such that $p-1 = 2^n m$. For the BLS12-377 scalar field, $n=47$ and $m=60001509534603559531609739528203892656505753216962260608619555$.

We define $g = \zeta^m$ where $\zeta$ is a non-square root of unity. We fix $\zeta$ as
2841681278031794617739547238867782961338435681360110683443920362658525667816.

We then define $\ell_0, ..., \ell_{k-1} > 0$ such that $\ell_0 + ... + \ell_{k-1} = n - 1$. Following Section 2.2 of [Sarkar 2020], for decaf377 we choose:
- $k=6$
- $\ell_{0,1} = 7$
- $\ell_{2, 3, 4, 5} = 8$

## Precomputation

Lookup tables[^note] are needed which can be precomputed as they depend only on the 2-adicity $n$ and the choice of $\ell_i$ above. This section follows the alternative table lookup method described in Section 3 of [Sarkar 2020].

We define $w = \max(\ell_0,...,\ell_{k-1}) = 8$. 

### Table 1: $g^{\nu 2 ^ {n - iw}}$

We compute $g^{\nu 2 ^ {n - iw}}$ for $\nu \isin {1, ..., 2^w - 1}$ and $i \isin {2, 1, 0}$. This can be stored as a matrix where the rows represent distinct $i$ values and the columns represent distinct $\nu$ values:

$
\begin{pmatrix}
g^{2^{31}} & g^{2^{32}} & ... & (g^{2^{31}})^{2^8 - 1}\\
g^{2^{39}} & g^{2^{40}} & ... & (g^{2^{39}})^{2^8 - 1}\\
g^{2^{47}} & g^{2^{48}} & ... & (g^{2^{47}})^{2^8 - 1}
\end{pmatrix}
$

For example, the entry for $\nu=1$ and $i=2$ is the upper left entry $g^{2^{31}}$.

### Table 2: $g^{\nu 2 ^ {iw}}$

We compute $g^{\nu 2 ^ {iw}}$ for $\nu \isin {1, ..., 2^w - 1}$ and $i \isin {0, 1, ..., 5}$. This can be stored as a matrix where the rows represent distinct $i$ values and the columns represent distinct $\nu$ values:

$
\begin{pmatrix}
g & g^{2} & ... & g^{2^8 - 1}\\
g^{2^{28}} & g^{2^{29}} & ... & (g^{2^8})^{2^8 - 1}\\
\vdots & \vdots & \ddots & \vdots  \\
g^{2^{40}} & g^{2^{41}} & ... & (g^{2^{40}})^{2^8 - 1}
\end{pmatrix}
$

### Table 3: $g^{-\nu (2^{n-w})}$

We compute $g^{-\nu (2^{n-w})}$ for $\nu \isin {1, ..., 2^w - 1}$. This can be stored as a vector:

$
\begin{pmatrix}
g^{2^{-39}} & (g^{2^{-39}})^2 & ... & (g^{2^{-39}})^{2^8 - 1}
\end{pmatrix}
$

## Procedure

In the following procedure, let $u=N/D$.

### Step 1: Compute $v=u^{\frac{m-1}{2}}$

We define $v = u^{\frac{m-1}{2}}$. This corresponds to line 2 of the `findSqRoot` Algorithm 1 in [Sarkar 2020].

Substituting $u=N/D$:

$v = (\frac N D)^{\frac{m-1}{2}} = N^{\frac{m-1}{2}} * D^{- \frac{m-1}{2}} $

Applying Fermat's Little Theorem (i.e. $D^{p-1} = 1 \mod p$):

$v = N^{\frac{m-1}{2}} * D^{p - 1 - \frac{m-1}{2}} $

Substituting $p- 1 = 2^n m$ and rearranging:

$v = N^{\frac{m-1}{2}} * D^{2^n m - \frac{m-1}{2}} $
$v = N^{\frac{m-1}{2}} * D^{\frac 1 2 (2^{n+1} m - m - 1)} $
$v = N^{\frac{m-1}{2}} * D^{\frac 1 2 (2^{n+1} m - m - 1 - 2^{n+1} + 2^{n+1})} $
$v = N^{\frac{m-1}{2}} * D^{\frac 1 2 (2^{n+1} - 1) (m - 1) + 2^{n}} $
$v = N^{\frac{m-1}{2}} * D^{\frac 1 2 (2^{n+1} - 1) (m - 1)} * D^{2^{n}} $

### Step 2: Compute $x$

Compute $x = u * v^2$. This corresponds to line 4 of the `findSqRoot` Algorithm 1 in [Sarkar 2020].

### Step 3: Compute $x_i$

We next compute $x_{i} = x^{2^{n - 1 - (\ell_0 +... + \ell_i)}}$ for $i=0,..,k-1$. This corresponds to line 5 of the `findSqRoot` Algorithm 1 in [Sarkar 2020]. This gives us the following components:

$x_0 = x^{2^{n - 1 - \ell_0}} = x^{2^{39}} = x_1 ^ {2^7}$
$x_1 = x^{2^{n - 1 - (\ell_0 + \ell_1)}} = x^{2^{32}}  = x_2 ^ {2^8}$
$x_2 = x^{2^{24}} = x_3 ^ {2^8}$
$x_3 = x^{2^{16}} = x_4 ^ {2^8}$
$x_4 = x^{2^8} = x_5 ^ {2^8}$
$x_5 = x^{2^0} = x$

### Step 4: Compute $s_i$ and $t_i$

Next, we loop over $k$. This corresponds to lines 6-9 of the `findSqRoot` Algorithm 1 in [Sarkar 2020]. 

#### For $i=0$

$t_0 = 0$
$\alpha_0 = x_0$

Using $\alpha_0 g^{s_0} = 1$ (Lemma 3 [Sarkar 2020]):

$x_0 g^{s_0} = 1$
$x_0 = g^{-s_0}$

We know $x_0$ from step 3 and $g$ is a constant, and we find $s_0$ using a lookup in Table 3. The table lookups give us $\nu$ where $s=\nu * 2^{n-w}$,

#### For $i=1$

$t_1 = s_0 2^{-\ell_1} = s_0 2^{-7} $
$\alpha_1 = x_1 g^{t_1} = x_1 g^{s_0 2^{-7}}$

Similarly to $i=0$, using $\alpha_1 g^{s_1} = 1$ we lookup $\alpha_1 = g^{-s_1}$ in the table.

#### For $i=2,...,5$

The remaining iterations yield:

$\alpha_i = x_i g^{t_i}$ where $t_i = (t_{i-1} + s_{i - 1})2^{-8})$

For each $\alpha_i$ we use the lemma $\alpha_i g^{s_i} = 1$. Once rearranged as $\alpha_i = g^{-s_i}$, we lookup in Table 3 to get $s_i$.

At the end of this step, we have found $s_i$ and $t_i$ for $i \isin {0,...,k-1}$.

### Step 5: Return result $y$

Next, given $t=t_5 + s_5$ from the previous step, $uv$ from step 1, we compute:

$y = uv g^{t/2}$

To compute $g^{t/2}$, we lookup entries in Tables 1 and 2. This corresponds to line 10 of the `findSqRoot` Algorithm 1 in [Sarkar 2020].

We can use the result of this computation $y$ to determine whether or not the square exists, recalling from Step 1 that $u=N/D$:

* If $u$ is square, then $y=\sqrt{N/D}$, and $y^2 D - N = 0$
* If $u$ is non-square, then $y=\sqrt{\zeta N/D}$ and $y^2 - \zeta N = 0$.

[^note]: In cases where $w$ divides $n$, table 1 and 2 can be combined. Since $w=8$ and $n=47$, we compute table 1 and 2 separately.

[internet-draft]: https://datatracker.ietf.org/doc/draft-irtf-cfrg-ristretto255-decaf448/01/
[Sarkar 2020]: https://eprint.iacr.org/2020/1407
[zcash-pasta]: https://github.com/zcash/pasta_curves
