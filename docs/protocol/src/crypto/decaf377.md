# The `decaf377` group

Penumbra, like many other zero-knowledge protocols, requires a cryptographic
group that can be used inside of an arithmetic circuit.  This is accomplished by
defining an "embedded" elliptic curve whose base field is the scalar field of
the proving curve used by the proof system.

The [Zexe paper][Zexe], which defined BLS12-377, also defined (called $E_{Ed/BLS}$ in Figure 16 of the paper) a
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

## Curve Parameters

The cofactor-4 Edwards curve defined over the BLS12-377 scalar field has the
following parameters:

* Base field: Integers mod prime $q=8444461749428370424248824938781546531375899335154063827935233455917409239041$
* Elliptic curve equation: $ax^2 + y^2 = 1 + dx^2y^2$ with $a=-1$ and $d=3021$
* Curve order: $4r$ where $r=2111115437357092606062206234695386632838870926408408195193685246394721360383$

We use a conventional generator basepoint selected to have a convenient hex encoding: 

```
0x0800000000000000000000000000000000000000000000000000000000000000
``` 

In affine coordinates this generator point has coordinates:

* $x=4959445789346820725352484487855828915252512307947624787834978378872129235627$
* $y=6060471950081851567114691557659790004756535011754163002297540472747064943288$

## Implementation

An implementation of `decaf377` can be found [here][impl].

[why-ristretto]: https://ristretto.group/why_ristretto.html
[Decaf]: https://www.shiftleft.org/papers/decaf/
[impl]: https://github.com/penumbra-zone/decaf377
[Zexe]: https://eprint.iacr.org/2018/962