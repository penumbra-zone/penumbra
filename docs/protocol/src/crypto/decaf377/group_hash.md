# Group Hash

Elligator can be applied to map a field element to a curve point. The map can be applied once to derive a curve point suitable for use with computational Diffie-Hellman (CDH) challenges, and twice to derive a curve point indistinguishable from random. 

In the following section, $a$ and $d$ are the curve parameters as described [here](../decaf377.md#curve-parameters). $\zeta$ is a constant and `isqrt(x)` is a function, both defined in the [Inverse Square Roots](./invsqrt.md) section.

The Elligator map is applied as follows to a field element $r_0$:

1. $r \gets \zeta r_0^2$.

2. $u_1 \gets (dr - d - a)(dr - ar - d)$.

3. $n_1 \gets (r + 1)(a - 2d)$.

4. $m, x =$ `isqrt`$(u_1 n_1)$ where $m$ is a boolean indicating whether or not a square root exists for the provided input. 

5. If a square root for $u_1 n_1$ does not exist, then $q=-1$ and $x = r_0 \zeta x$. Else, $q=1$ and $x$ is unchanged.

6. $s \gets x n_1$.

7. $t \gets -q x s (r-1) (a - 2d)^2 - 1$. 

8. If ($s < 0$ and $m$ is true) or ($s > 0$ and $m$ is false) then $s = -s$.

The Jacobi quartic representation of the resulting point is given by $(s, t)$. The resulting point can be converted from its Jacobi quartic representation to affine Edwards coordinates via: 

$x \gets 2s / (1 + as^2)$

$y \gets (1 - as^2) / t$ 

For single-width hash-to-group (`map_to_group_cdh`), we apply the above map once. For double-width (`map_to_group_uniform`) we apply the map to two field elements and add the resulting curve points.
