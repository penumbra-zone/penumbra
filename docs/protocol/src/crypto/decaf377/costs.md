# Costs and Alternatives

Arithmetic circuits have a different cost model than software.  In the software
cost model, software executes machine instructions, but in the circuit cost
model, relations are certified by constraints.  Unfortunately, while Decaf is a
clearly superior choice in the software context, in the circuit context it
imposes some additional costs, which must be weighed against its benefits.

At a high level, Decaf implements a prime-order group using a non-prime-order
curve by constructing a group quotient.  Internally, group elements are
represented by curve points, with a custom equality check so that equivalent
representatives are considered equal, an encoding function that encodes
equivalent representatives as identical bitstrings, and a decoding function that
only accepts canonical encodings of valid representatives.

To do this, the construction defines a canonical encoding on a Jacobi quartic
curve mod its 2-torsion (a subgroup of size 4) by making two independent sign
choices.  Then, it uses an isogeny to transport this encoding from the Jacobi
quartic to a target curve that will be used to actually implement the group
operations.  This target curve can be an Edwards curve or a Montgomery curve.
The isogenies are only used for deriving the construction. In implementations,
all of these steps are collapsed into a single set of formulas that perform
encoding and decoding on the target curve.

In other words, one way to think about the Decaf construction is as some
machinery that transforms two sign choices into selection of a canonical
representative. [Ristretto] adds extra machinery to handle cofactor 8 by making
an additional sign choice.

In the software cost model, where software executes machine instructions, this
construction is essentially free, because the cost of both the Decaf and
conventional Edwards encodings are dominated by the cost of computing an inverse
or an inverse square root, and the cost of the sign checks is insignificant.

However, in the circuit cost model, where relations are certified by various
constraints, this is no longer the case.  On the one hand, certifying a square
root or an inverse just requires checking that $y^2 = x$ or that $xy = 1$, which
is much cheaper than actually computing $y = \sqrt x$ or $y = 1/x$.  On the
other hand, performing a sign check involves bit-constraining a field element, 
requiring hundreds of constraints.

## Sign checks

The definition of which finite field elements are considered nonnegative is
essentially arbitrary.  The Decaf paper suggests three possibilities:

- using the least significant bit, defining $x$ to be nonnegative if the least
absolute residue for $x$ is even;

- using the most significant bit, defining $x$ to be nonnegative if the least
absolute residue for $x$ is in the range $0 \leq x \leq (q-1)/2$;

- for fields where $q \equiv 3 \pmod 4$, using the Legendre symbol, which
distinguishes between square and nonsquare elements.

Using the Legendre symbol is very appealing in the circuit context, since it has
an algebraic definition and, at least in the case of square elements, very
efficient certification.  For instance, if square elements are chosen to be
nonnegative, then certifying that $x$ is nonnegative requires only one
constraint, $x = y^2$.  However, the reason for the restriction to $3 \pmod 4$
fields is that $1$ and $-1$ should have different signs, which can only be the
case if $-1$ is nonsquare. Unfortunately, many SNARK-friendly curves, including
BLS12-377, are specifically chosen so that $q \equiv 1 \pmod {2^\alpha}$ for as
large a power $\alpha$ as possible (e.g., $\alpha = 47$ in the case of
BLS12-377).

This leaves us with either the LSB or MSB choices.  The least significant bit is
potentially simpler for implementations, since it is actually the low bit of the
encoding of $x$, while the most significant bit isn't, because it measures from
$(q-1)/2$, not a bit position $2^k$, so it seems to require a comparison or
range check to evaluate.  However, these choices are basically equivalent, in
the following sense:

###### Lemma.[^1]

The most significant bit of $x$ is $0$ if and only if the least significant bit of $2x$ is $0$.

###### Proof.

The MSB of $x$ is $0$ if and only if $2x \leq q - 1$, but this means that $2x$,
which is even, is the least absolute residue, so the LSB of $2x$ is also $0$.
On the other hand, the MSB of $x$ is $1$ if and only if $x > (q-1)/2$, i.e., if
$x \geq (q-1)/2 + 1$, i.e., if $2x \geq q - 1 + 2 = q +1$.  This means that the
least absolute residue of $2x$ is $2x - q$; since $2x$ is even and $q$ is odd,
this is odd and has LSB $1$. $\square$

This means that transforming an LSB check to an MSB check or vice versa requires
multiplication by $2$ or $1/2$, which costs at most one constraint.

Checking the MSB requires checking whether a value is in the range $[0,
(q-1)/2]$.  Using [Daira Hopwood's optimized range constraints][daira_range],
the range check costs $73C$[^2]. However, the input to the range check is a
bit-constrained unpacking of a field element, not a field element itself.  This
unpacking costs $253C$.

Checking the LSB is no less expensive, because although the check only examines
one bit, the circuit must certify that the bit-encoding is canonical.  This
requires checking that the value is in the range $[0, q-1]$, which also costs $73C$, and as before, the unpacking costs $253C$.

In other words, checking the sign of a field element costs $253C + 73C = 326C$,
or $76C$ in the case where the field element is already bit-encoded for other
reasons.  These checks are the dominant cost for encoding and decoding, which
both require two sign checks.  Decoding from bits costs c. $73C + 326C \cong
400C$, decoding from a field element costs c. $326C + 326C \cong 750C$, and
encoding costs c. $750C$ regardless of whether the output is encoded as bits or
as a field element.

For `decaf377`, we choose the LSB tests for sign checks.

## Alternative approaches to handling cofactors

Decaf constructs a prime-order group whose encoding and decoding methods perform
validation.  A more conventional alternative approach is to use the underlying
elliptic curve directly, restrict to its prime-order subgroup, and do subgroup
validation separately from encoding and decoding.  If this validation is done
correctly, it provides a prime-order group.  However, because validation is an
additional step, rather than an integrated part of the encoding and decoding
methods, this approach is necessarily more brittle, because each
implementation must be sure to do both steps.

In the software cost model, there is no reason to use subgroup validation,
because it is both more expensive and more brittle than Decaf or Ristretto.
However, in the circuit cost model, there are cheaper alternatives, previously
analyzed by Daira Hopwood in the context of Ristretto for JubJub
([1][suggestion], [2][costs]).  

###### Multiplication by the group order.

The [first validation method][gporder] is to do a scalar multiplication and
check that $[q]P = 1$.  Because the prime order is fixed, this scalar
multiplication can be performed more efficiently using a hardcoded sequence of
additions and doublings.

###### Cofactor preimage.

The [second validation method][preimage] provides a preimage $Q = [1/4]P$ in
affine coordinates $(x,y)$. Because the image of $[4]: \mathcal E \rightarrow
\mathcal E$ is the prime-order subgroup, checking that $Q$ satisfies the curve
equation and that $P = [4]Q$ checks that $P$ is in the prime-order subgroup.

In the software context, computing $[1/4]P$ and computing $[q]P$ cost about the
same, although both are an order of magnitude more expensive than decoding.  But
in the circuit context, the prover can compute $Q = [1/4]P$ outside of the
circuit and use only a few constraints to check the curve equation and two
doublings.  These costs round to zero compared to sign checks, so the validation
is almost free.

The standard "compressed Edwards y" format encodes a point $(x,y)$ by the
$y$-coordinate and a sign bit indicating whether $x$ is nonnegative.  In
software, the cost of encoding and decoding are about the same, and dominated by
taking an inverse square root.  In circuits, the costs of encoding and decoding
are also about the same, but they are instead dominated by a sign check that
matches the sign of the recovered $x$-coordinate with the supplied sign bit.
This costs c. $325C$ as above.

## Comparison and discussion

This table considers only approximate costs.

| Operation | `decaf377` | Compressed Ed + Preimage |
|-|-|-|
| Decode (from bits) | 400C | 400C |
| Decode (from $F_q$) | 750C | 325C |
| Encode (to bits) | 750C | 750C |
| Encode (to $F_q$) | 750C | 325C |

When decoding from or encoding to field elements, the marginal cost of Decaf
compared to the compressed Edwards + cofactor preimage is an extra bit-unpacking
and range check.  While this effectively doubles the number of constraints, the
marginal cost of c. $325C$ is still small relative to other operations like a
scalar multiplication, which at [6 constraints per bit][daira_scmul] is
approximately $1500C$.  

When decoding from or encoding to bits, the marginal cost of Decaf disappears.
When the input is already bit-constrained, Decaf's first sign check can reuse
the bit constraints, saving c. $250C$, but the compressed Edwards encoding must
range-check the bits (which Decaf already does), costing c. $75C$ extra.
Similarly, in encoding, Decaf's second sign check produces bit-constrained
variables for free, while the compressed Edwards encoding spends c. $250C + 75C$
bit-constraining and range-checking them.

However, in the software context, the prime-order validation check costs
approximately 10x more than the cost of either encoding.  Many applications
require use of the embedded group both inside and outside of the circuit, and
uses outside of the circuit may have additional resource constraints (for
instance, a hardware token creating a signature authorizing delegated proving,
etc.).  

Performing validation as an additional, optional step also poses additional
risks.  While a specification may require it to be performed, implementations
that skip the check will appear to work fine, and the history of invalid-point
attacks (where implementations should, but don't, check that point coordinates
satisfy the curve equation) suggests that structuring validation as an integral
part of encoding and decoding is a safer design.  This may not be a concern for
a specific application with a single, high-quality implementation that doesn't
make mistakes, but it's less desirable for a general-purpose construction.

In summary, Decaf provides a construction that works the same way inside and
outside of a circuit and integrates validation with the encoding, imposing only
a modest cost for use in circuits and no costs for lightweight, software-only
applications, making it a good choice for general-purpose constructions.

[why-ristretto]: https://ristretto.group/why_ristretto.html
[Decaf]: https://www.shiftleft.org/papers/decaf/
[Ristretto]: https://ristretto.group
[daira_range]: https://github.com/zcash/zcash/issues/3366
[daira_range_python]: https://github.com/zcash/zcash/issues/3366#issuecomment-402077803
[suggestion]: https://github.com/zcash/zcash/issues/3924#issuecomment-488028365
[costs]: https://github.com/zcash/zcash/issues/4024
[gporder]: https://github.com/zcash/zcash/issues/4024#issuecomment-495024496
[preimage]: https://github.com/zcash/zcash/issues/4024#issuecomment-500473198
[daira_scmul]: https://github.com/zcash/zcash/issues/3924

[^1]: I have no idea whether this is common knowledge; I learned of this fact
from its use in Mike Hamburg's Ed448-Goldilocks implementation.

[^2]: The value 73 is computed as:
```python
from itertools import groupby

def cost(k):
  return min(k-1, 2)

def constraints(bound):
  costs = [cost(len(list(g))+1) for (c, g) in groupby(bound.bits()[:-1]) if c == 1]
  return sum(costs)

constraints(ZZ((q-1)/2))
```
as [here][daira_range_python].
