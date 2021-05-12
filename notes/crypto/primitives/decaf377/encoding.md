# Encoding

Given a representative in extended coordinates $(X,Y,Z,T)$, encoding works as follows.  

1. $u_1 \gets (X+T)(Y-T)$.

2. `(_ignored, v) = sqrt_ratio_zeta(1, u_1 * (1 - d) * x^2)`.

3. $u_2 \gets |v u_1|$ (sign check 1).

4. $u_3 \gets u_2 Z - T$.

5. $s \gets |(-1-d) v u_3 X|$.

6. Set `s_bytes` to be the canonical little-endian encoding of $s$.

