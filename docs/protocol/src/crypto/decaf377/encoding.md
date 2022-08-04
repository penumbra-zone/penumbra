# Encoding

Given a representative in extended coordinates $(X,Y,Z,T)$, encoding works as follows where $a$ and $d$ are the curve parameters as described [here](../decaf377.md#curve-parameters).

1. $u_1 \gets (X+T)(X-T)$.

2. `(_ignored, v) = sqrt_ratio_zeta(1, u_1 * (a - d) * X^2)`.

3. $u_2 \gets |v u_1|$ (sign check 1).

4. $u_3 \gets u_2 Z - T$.

5. $s \gets |(a-d) v u_3 X|$.

6. Set `s_bytes` to be the canonical unsigned little-endian encoding of $s$, which is an integer mod $q$. `s_bytes` has extra `0x00` bytes appended to reach a length of 32 bytes.
