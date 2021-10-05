# Group Hash

Elligator can be applied to map a field element to a curve point. The map can be applied once to derive a curve point suitable for use with computational Diffie-Hellman (CDH) challenges, and twice to derive a curve point indistinguishable from random. 

In the following section, $A$ and $D$ are the curve parameters and $\zeta$ is a constant defined in the [Inverse Square Roots](./invsqrt.md) section.

The Elligator map is applied as follows to a field element $r_0$:

1. $r \gets \zeta r_0^2$.

2. $u_1 \gets \frac{D * r - (D - A)}{(D - A) r - D}$.

3. $n_1 \gets (r + 1)(A - 2D)/u_1$.

4. $n_2 \gets r n_1$.

5. If a square root for $n_1$ exists, then the Jacobi quartic representation $(s,t)$ of the resulting point is $(s, t) \gets (|\sqrt{n1}|, -(r - 1)(A - 2D)^2)/u_1 - 1)$. Else $(s,t) \gets (-|\sqrt{n2}|, r*(r - 1)(A - 2D)^2)/u_1 - 1)$.

The resulting point can be converted from its Jacobi quartic representation to affine Edwards coordinates via: 

$x \gets 2s / (1 + A*s^2)$
$y \gets 1 - As^2 / t$ 

For single-width hash-to-group (`map_to_group_cdh`), we apply the above map once. For double-width (`map_to_group_uniform`) we apply the map to two field elements and add the resulting curve points.

- [ ] TODOs: specify optimized version, need to match the choice of quadratic nonresidue with $\zeta$ used in invsqrt.
