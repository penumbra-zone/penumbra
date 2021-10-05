# The `decaf377` group

Penumbra, like many other zero-knowledge protocols, requires a cryptographic
group that can be used inside of an arithmetic circuit.  This is accomplished by
defining an "embedded" elliptic curve whose base field is the scalar field of
the proving curve used by the proof system.

The Zexe paper, which defined BLS12-377, also defined (but did not name) a
cofactor-4 Edwards curve defined over the BLS12-377 scalar field for exactly
this purpose.  However, [non-prime-order groups are a leaky
abstraction][why-ristretto], forcing all downstream constructions to pay
attention to correct handling of the cofactor.  Although it is usually possible
to do so safely, it requires additional care, and the optimal technique for
handling the cofactor is different inside and outside of a circuit.

Instead, applying the [Decaf] construction to this curve gives `decaf377`, a
clean abstraction that provides a prime-order group complete with hash-to-group
functionality and whose encoding and decoding functions integrate validation.
Although it imposes a modest additional cost in the circuit context, as
discussed in [Costs and Alternatives](./decaf377/costs.md), the
construction works the same way inside and outside of a circuit and imposes no
costs for lightweight, software-only applications, making it a good choice for
general-purpose applications.

## Implementation

A work-in-progress implementation of `decaf377` can be found [here][impl].

[why-ristretto]: https://ristretto.group/why_ristretto.html
[Decaf]: https://www.shiftleft.org/papers/decaf/
[impl]: https://github.com/penumbra-zone/decaf377