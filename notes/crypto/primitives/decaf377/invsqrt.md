# Inverse Square Roots

As in the [internet-draft], the `decaf377` functions are defined in terms of the
following function, which computes the square root of a ratio of field elements,
with the special behavior that if the input is nonsquare, it returns the square
root of a related field element, to allow reuse of the computation in the
hash-to-group setting.

- [ ] TODO: actually specify this procedure.

Fix $\zeta$ as
2841681278031794617739547238867782961338435681360110683443920362658525667816.
Then $\zeta$ is a nonsquare $2^{47}$-th root of unity.

- [ ] TODO: possibly change this value to be convenient for implementations,
rather than something that's convenient for SAGE;

Define `sqrt_ratio_zeta(u,v)` as:

- (True, $\sqrt{\frac u v}$) if $u$ and $v$ are nonzero, and $\frac u v$ is square;
- (True, $0$) if $u$ is zero;
- (False, $0$) if $v$ is zero;
- (False, $\sqrt{ \zeta \frac u v}$) if $u$ and $v$ are nonzero, and $\frac u v$ is nonsquare.

Since $\zeta$ is nonsquare, if $\frac u v$ is nonsquare, $\zeta \frac u v$ is
square.  Note that **unlike** the similar function in the
`ristretto255`/`decaf448` [internet-draft], this function does not make any
claims about the sign of its output.

- [ ] TODO: describe efficient implementation using [2020/1407]

[2020/1407]: https://eprint.iacr.org/2020/1407