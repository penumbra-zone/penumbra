# Decoding

Decoding to works as follows:

1. Decode `s_bytes` to a field element $s$, rejecting if the encoding is
non-canonical.  (If the input is already a field element in the circuit case,
skip this step).

2. Check that $s$ is nonnegative, or reject (sign check 1).

3. $u_1 \gets 1 - s^2$.

4. $u_2 \gets u_1^2 - 4d s^2$.

5. `(was_square, v) = sqrt_ratio_zeta(1, u_2 * u_1^2)`, rejecting if `was_square` is false.

6. $v \gets -v$ if $2s u_1 v$ is negative (sign check 2).

7. $(x, y) \gets (2s v^2 u_1 u_2, (1+s^2)vu_2)$.

The resulting coordinates are the affine Edwards coordinates of an internal
representative of the group element.

- [ ] simplify formulas using numerator instead of 1
