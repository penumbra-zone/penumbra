# Sender and Receiver FMD

The goal of detection capability is to be able to filter the *global* stream of
state updates into a *local* stream of state updates that includes all updates
related to a particular address, without identifying precisely which updates
those are.  The rate of updates on this filtered stream should be bounded below,
to ensure that there is a large enough anonymity set, and bounded above, so that
users processing the stream have a constant and manageable amount of work to
process it and catch up with the current chain state.  This means that the
detection precision must be adaptive to the global message rates: if the false
positive rate is too low, the filtered stream will have too few messages, and if
it is too high, it will have too many messages.

However, this isn't possible using the paper's original definition of fuzzy
message detection, because the false positive probability is chosen by the
receiver, who has no way to know in advance what probability will produce a
correctly sized stream of messages.  One way to address this problem is to
rename the original FMD definition as *Receiver FMD* (R-FMD), and tweak it to
obtain *Sender FMD* (S-FMD), in which the sender chooses the detection
probability.

## Receiver FMD

The paper's original definition of fuzzy message detection is as a tuple of
algorithms `(KeyGen, CreateClue, Extract, Examine)`.[^1]  The receiver uses
`KeyGen` to generate a *root key* and a *clue key*.  A sender uses the
receiver's clue key as input to `CreateClue` to produce a *clue*.  The `Extract`
algorithm takes the root key and a false positive rate $p$ (chosen from some set
of supported rates), and produces a *detection key*.  The `Examine` algorithm
uses a detection key to examine a clue and produce a detection result.

This scheme should satisfy certain properties, formalizations of which can be
found in the paper:

###### Correctness.

Valid matches must always be detected by `Examine`; i.e., there are no false negatives.

###### Fuzziness.

Invalid matches should produce false positives with probability approximately
$p$, as long as the clues and detection keys were honestly generated.

###### Detection Ambiguity.

An adversarial detector must be unable to distinguish between a true positive
and a false positive, as long as the clues and detection keys were
honestly generated.

In this original definition, the receiver has control over how much detection
precision they delegate to a third party, because they choose the false positive
probability when they extract a detection key from their root key.  This fits
with the idea of [attenuating credentials][macaroons], and intuitively, it seems
correct that the receiver should control how much information they reveal to
their detector.  But the information revealed to their detector is determined by
both the false positive probability *and* the amount of other messages that can
function as cover traffic.  Without knowing the extent of other activity on the
system, the receiver has no way to make a principled choice of the detection
precision to delegate.

## Sender FMD

To address this problem, we generalize the original definition (now *Receiver
FMD*) to *Sender FMD*, in which the false positive probability is chosen by the
sender.

S-FMD consists of a tuple of algorithms `(KeyGen, CreateClue, Examine)`.  Like
R-FMD, `CreateClue` creates a clue and `Examine` takes a detection key and a
clue and produces a detection result.  As discussed in the [next
section](./construction.md), S-FMD can be realized using tweaks to either of the
R-FMD constructions in the original paper.

Unlike R-FMD, the false positive rate is set by the sender, so `CreateClue`
takes both the false positive rate $p$ and the receiver's clue key.  Because the
false positive rate is set by the sender, there is no separation of capability
between the root key and a detection key, so `KeyGen` outputs a clue key and a
detection key, and `Extract` disappears.

In R-FMD, flag ciphertexts are universal with respect to the false positive
rate, which is applied to the detection key; in S-FMD, the false positive rate
is applied to the flag ciphertext and the detection key is universal.

Unlike R-FMD, S-FMD allows detection precision to be adaptive, by having senders
use a (consensus-determined) false positive parameter.  This parameter should
vary as the global message rates vary, so that filtered message streams have a
bounded rate, and it should be the same for all users, so that messages cannot
be distinguished by their false positive rate.

[^1]: We change terminology from the FMD paper; the paper calls detection and
clue keys the secret and public keys respectively, but we avoid this in favor of
capability-based terminology that names keys according to the precise capability
they allow.  The "clue" terminology is adopted from the [_Oblivious Message
Retrieval_][omr] paper of Zeyu Liu and Eran Tromer; we `CreateClue` and `Examine` clues
rather than `Flag` and `Test` flag ciphertexts.

[macaroons]: https://static.googleusercontent.com/media/research.google.com/en//pubs/archive/41892.pdf
[omr]: https://eprint.iacr.org/2021/1256
