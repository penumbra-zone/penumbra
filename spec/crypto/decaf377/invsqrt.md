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

We then define a $k \ge 1$ and $\ell_0, ..., \ell_{k-1} > 0$ such that $\ell_0 + ... + \ell_{k-1} = n - 1$. We also define a parameter $w$ where $1 \le w \le \max{(\ell_0, \ell_1, ..., \ell_{k-1})}$. For decaf377 we choose:
- $k=6$
- $\ell_{0,1} = 7$
- $\ell_{2, 3, 4, 5} = 8$
- $w = 8$

## Precomputation

Lookup tables are needed which can be precomputed as they depend only on the 2-adicity $n$ and the choice of $\ell_i$ above.

### $g$ lookup table: $g^{\nu 2 ^ {x}}$

We compute $g^{\nu 2 ^ {x}}$ for $\nu \isin {1, ..., 2^w - 1}$ and $x \isin {0, 7, 8, 15, 16, 23, 24, 31, 32, 40}$, indexed on $x$ and $\nu$:

$
\begin{pmatrix}
g & g^{2} & ... & g^{2^8 - 1}\\
g^{2^{7}} & g^{2^{8}} & ... & (g^{2^7})^{2^8 - 1}\\
\vdots & \vdots & \ddots & \vdots  \\
g^{2^{40}} & g^{2^{41}} & ... & (g^{2^{40}})^{2^8 - 1}
\end{pmatrix}
$

This table lets us lookup powers of $g$. The required values of $x$ are the powers of 2 that appear in our expressions for $t_i$, i.e. ${0, 7, 8, 15, 16, 23, 24, 31, 32}$, as well as any additional powers of 2 that are needed to compute $g^{t/2}$ in Step 5, which adds $40$.

### $s$ lookup table: $g^{-\nu (2^{n-w})}$

We compute $g^{-\nu (2^{n-w})}$ for $\nu \isin {0, ..., 2^w - 1}$, indexed on $g^{-\nu (2^{n-w})}$:

$
\begin{pmatrix}
g^0 & g^{2^{-39}} & (g^{2^{-39}})^2 & ... & (g^{2^{-39}})^{2^8 - 1}
\end{pmatrix}
$

We use this table in the procedure that follows to find $q_i$ (they are the $\nu$ values) in order to compute $s_i$.

## Procedure

In the following procedure, let $u=N/D$. We use the following relations from [Sarkar 2020]:

* Equation 1: $\alpha_i = x_i g^{t_i}$ and $t_i = (t_{i - 1} + s_{i - 1})/2^{\ell_i}$ for $i \isin {0, ..., k-1}$ and $t_k = t_{k - 1} + s_{k - 1}$
* Lemma 3: $\alpha_i g^{s_i} = 1$ for $i \isin {0, ..., k-1}$
* Equation 2: $s_i = q_i 2^{n - l_i}$

### Step 1: Compute $v=u^{\frac{m-1}{2}}$

We compute $v = u^{\frac{m-1}{2}}$. This corresponds to line 2 of the `findSqRoot` Algorithm 1 in [Sarkar 2020].

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

Using Lemma 3:

$\alpha_0 g^{s_0} = 1$

Substituting the definition of $\alpha_0$ from equation 1:

$x_0 g^{t_0} g^{s_0} = 1$

Rearranging and substituting in $t_0 = 0$ (initial condition):

$x_0 = g^{-s_0} $

Substituting in equation 2:

$x_0 = g^{-q_0 2^{n - \ell_0}} = g^{-q_0 2^{40}} $

This is almost in a form where we can look up $q_0$ in our s lookup table to get $q_0$ and thus $s_0$. If we define $q'_0 = 2 q_0$ we get:

$x_0 = g^{-q'_0 2^{39}}$

Which we can use with our s lookup table to get $q'_0$. Expressing $s_0$ in terms of $q'_0$, we get $s_0 = q'_0 2^{39}$.

#### For $i=1$

First we compute $t_1$ using equation 1:

$t_1 = (t_0 + s_0) / 2^{\ell_1} = (t_0 + s_0)/2^7 = q'_0 2^{32}$

Next, similar to the first iteration, we use lemma 3 and substitute in $t_1$ and $s_1$ to yield:

$\alpha_1 g^{s_1} = 1$

$x_1 g^{q'_0 2^{32}} = g^{-q'_1 2^{39}}$

In this expression we can compute the quantities on the left hand side, and the right hand side is in the form we expect for the s lookup table, yielding us $q'_1$. Note that here too we define $q'_1 = 2 q_1$ such that the s lookup table can be used. Expressing $s_1$ in terms of $q'_1$, we get $s_1 = q'_1 2^{39}$.

#### For $i=2,...,5$

The remaining iterations proceed similarly, yielding the following expressions:

$t_2 = q'_0 2^{24} + q'_1 2^{31}$
$s_2 = q_2 2^{39}$

Note for $q_2$ and the remaining iterations we do not require a trick (i.e. with $q'_0$, $q'_1$) to get $s_2$ in a form where it can be used with the s lookup table.

$t_3 = q'_0 2^{16} + q'_1 2^{23} + q_2 2^{31}$
$s_3 = q_3 2^{39}$

$t_4 = q'_0 2^{8} + q'_1 2^{15} + q_2 2^{23} + q_3 2^{31}$
$s_4 = q_4 2^{39}$

$t_5 = q'_0 + q'_1 2^{7} + q_2 2^{15} + q_3 2^{23} + q_4 2^{31} $
$s_5 = q_5 2^{39}$

At the end of this step, we have found $s_i$ and $t_i$ for $i \isin {0,...,k-1}$.

### Step 5: Return result $y$

Next, we can use equation 1 to compute $t=t_5 + s_5$ using $t_5$ and $s_5$ from the previous step:

$t = q'_0 + q'_1 2^{7} + q_2 2^{15} + q_3 2^{23} + q_4 2^{31} + q_5 2^{39} $

This matches the expression from Lemma 4 in [Sarkar 2020]. 

Next, to compute $g^{t/2}$, we lookup entries in the g lookup table. To do so, we can decompose $t/2$ into:

$t/2 = v_0 2^0 + v_1 2^8 + v_2 2^{16} + v_3 2^{24} + v_4 2 ^{32} + v_5 2^{40}$ 

then $g^{t/2}$ is computed as:

$g^{t/2} = g^{v_0 2^0} g^{v_1 2^8} g^{v_2 2^{16}} g^{v_3 2^{24}} g^{v_4 2 ^{32}} g^{v_5 2^{40}}$

Multiplying in $uv$ from step 1, we compute:

$y = uv g^{t/2}$

This corresponds to line 10 of the `findSqRoot` Algorithm 1 in [Sarkar 2020].

We can use the result of this computation $y$ to determine whether or not the square exists, recalling from Step 1 that $u=N/D$:

* If $u$ is square, then $y=\sqrt{N/D}$, and $y^2 D - N = 0$
* If $u$ is non-square, then $y=\sqrt{\zeta N/D}$ and $y^2 - \zeta N = 0$.

[internet-draft]: https://datatracker.ietf.org/doc/draft-irtf-cfrg-ristretto255-decaf448/01/
[Sarkar 2020]: https://eprint.iacr.org/2020/1407
[zcash-pasta]: https://github.com/zcash/pasta_curves
