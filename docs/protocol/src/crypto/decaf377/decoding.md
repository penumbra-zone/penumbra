# Decoding

Decoding to a point works as follows where $a$ and $d$ are the curve parameters as described [here](../decaf377.md#curve-parameters).

1. Decode `s_bytes` to a field element $s$. We interpret these bytes as unsigned
little-endian bytes. We check if the length has 32 bytes, even though
the top 3 bits of the last byte are not used. The 253 bits are verified to be canonical, and rejected if not (If the input is already a field element in the circuit case, skip this step).

2. Check that $s$ is nonnegative, or reject (sign check 1).

3. $u_1 \gets 1 + as^2$.

4. $u_2 \gets u_1^2 - 4d s^2$.

5. `(was_square, v) = sqrt_ratio_zeta(1, u_2 * u_1^2)`, rejecting if `was_square` is false.

6. $v \gets -v$ if $2s u_1 v$ is negative (sign check 2)[^1].

7. $(x, y) \gets (2s v^2 u_1 u_2, (1 - as^2)vu_1)$.

The resulting coordinates are the affine Edwards coordinates of an internal
representative of the group element.

[^1]: Note this differs from the Decaf paper in Appendix A.2, but
implementations of `decaf377` should follow the convention described here.
