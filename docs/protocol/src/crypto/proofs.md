# Proving Considerations

Penumbra needs SNARK proofs.  Because the choice of proving system and proving
curve can't really be cleanly separated from the rest of the system choices
(e.g., the native field of the proving system informs what embedded curve is
available, and how circuit programming is done), large parts of the rest of the
system design block on making a choice of proving system.

## Goals

1.  *Near-term implementation availability*. We'd like to ship a useful product first, and iterate and expand it later.

2.  *High performance for fixed functionality*.  Penumbra intends to support
fixed functionality initially; programmability is a good future goal but isn't a
near-term objective.  The fixed functionality should have as high performance as
possible.

3.  *Longer-term flexibility*.  The choice should ideally not preclude many
future choices for later functionality.  More precisely, it should not impose
high switching costs on future choices.

4.  *Recursion capability*.  Penumbra doesn't currently make use of recursion,
but there are a lot of interesting applications it could be used for.

Setup ceremonies are beneficial to avoid for *operational reasons*, but not for security reasons.  A decentralized setup procedure is sufficient for security.

## Options

Proof systems:

- *Groth16*: 
  - Pros: high performance, very small proofs, mature system
  - Cons: requires a setup for *each* proof statement
- *PLONK*:
  - Pros: universal setup, still fairly compact proofs, seems to be a point of convergence with useful extensions (plookup, SHPLONK, etc)
  - Cons: bigger proofs, worse constants than Groth16
- *Halo 2*
  - Pros: no setup, arbitrary depth recursion
  - Cons: bigger proof sizes, primary implementation for the Pallas/Vesta curves which don't support pairings

Curve choices:

- *BLS12-381*:
  - Pros: very mature, used by Sapling already
  - Cons: no easy recursion

- *BLS12-377*:
  - Pros: constructed as part of Zexe to support depth 1 recursion using a bigger parent curve, deployed in Celo, to be deployed in Zexe
  - Cons: ?

- *Pallas/Vesta*:
  - Pros: none other than support for Halo 2's arbitrary recursion
  - Cons: no pairings mean they cannot be used for any pairing-based SNARK

## Considerations

Although the choice of proof system (Groth16, Plonk, Halo, Pickles, ...) is not
completely separable from the choice of proving curve (e.g., pairing-based
SNARKs require pairing-friendly curves), to the extent that it is, *the choice
of the proof system is relatively less important than the choice of proving
curve*, because it is easier to encapsulate.

The choice of proving curve determines the scalar field of the arithmetic
circuit, which determines which curves are efficient to implement in the
circuit, which determines which cryptographic constructions can be performed in
the circuit, which determines what kind of key material the system uses, which
propagates all the way upwards to user-visible details like the address format.
While swapping out a proof system using the same proving curve can be
encapsulated within an update to a client library, swapping out the proving
curve is extremely disruptive and essentially requires all users to generate new
addresses and migrate funds.

This means that, in terms of proof system flexibility, the Pallas/Vesta curves
are relatively disadvantaged compared to pairing-friendly curves like BLS12-381
or BLS12-377, because they cannot be used with any pairing-based SNARK, or any
other pairing-based construction.  Realistically, choosing them is committing to
using Halo 2.

Choosing BLS12-377 instead of BLS12-381 opens the possibility to do depth-1
recursion later, without meaningfully restricting the near-term proving choices.
For this reason, BLS12-377 seems like the best choice of proving curve.

Penumbra's approach is to first create a useful set of fixed functionality, and
generalize to custom, programmable functionality only later. Compared to
Sapling, there is *more* functionality (not just `Spend` and `Output` but
`Delegate`, `Undelegate`, `Vote`, ...), meaning that there are more proof
statements.  Using Groth16 means that each of these statements needs to have its
own proving and verification key, generated through a decentralized setup. 

So the advantage of a universal setup (as in PLONK) over per-statement setup (as
in Groth16) would be:

1. The setup can be used for additional fixed functionality later;
2. Client software does not need to maintain distinct proving/verification keys
for each statement.

(2) is a definite downside, but the impact is a little unclear.  As a point of
reference, the Sapling spend and output parameters are 48MB and 3.5MB
respectively.  The size of the spend circuit could be improved using a
snark-friendly hash function.

With regard to (1), if functionality were being developed in many independent
pieces, doing many setups would impose a large operational cost.  But doing a
decentralized setup for a dozen proof statements simultaneously does not seem
substantially worse than doing a decentralized setup for a single proof
statement.  So the operational concern is related to the frequency of groups of
new statements, not the number of statements in a group. Adding a later group of
functionality is easy if the first group used a universal setup.  But if it
didn't, the choice of per-statement setup initially doesn't prevent the use of a
universal setup later, as long as the new proof system can be implemented using
the same curve.

Because Penumbra plans to have an initial set of fixed functionality, and
performance is a concern, Groth16 seems like a good choice, and leaves the door
open for a future universal SNARK.  Using BLS12-377 opens the door to future
recursion, albeit only of depth 1.
