# Fuzzy Message Detection

By design, privacy-preserving blockchains like Penumbra don't reveal metadata
about the sender or receiver of a transaction.  However, this means that users
must scan the entire chain to determine which transactions relate to their
addresses.  This imposes large bandwidth and latency costs on users who do not
maintain online replicas of the chain state, as they must "catch up" each time
they come online by scanning all transactions that have occurred since their
last activity.

Alternatively, users could delegate scanning to a third party, who would monitor
updates to the chain state on their behalf and forward only a subset of those
updates to the user.  This is possible using *viewing keys*, as in Zcash, but
viewing keys represent the capability to view all activity related to a
particular address, so they can only be delegated to trusted third parties.

Instead, it would be useful to be able to delegate only a probabilistic
*detection* capability.  Analogous to a Bloom filter, this would allow a
detector to identify all transactions related to a particular address (no false
negatives), while also identifying unrelated transactions with some false
positive probability.  Unlike viewing capability, detection capability would not
include the ability to view the details of a transaction, only a probabilistic
association with a particular address.  This is the problem of *fuzzy message
detection* (FMD), analyzed by Beck, Len, Miers, and Green in their paper [Fuzzy
Message Detection][fmd-paper], which proposes a cryptographic definition of
fuzzy message detection and three potential constructions.

This section explores how Penumbra could make use of fuzzy message detection,
and proposes a generalization of the original definition that is more practical
to integrate into a larger system.

## Defining FMD

The paper's original definition of fuzzy message detection is as a tuple of
algorithms `(KeyGen, Flag, Extract, Test)`.  The receiver uses `KeyGen` to
generate a *root key* and a *flag key*[^1].  A sender uses the
receiver's flag key as input to `Flag` to produce a *flag ciphertext*.  The
`Extract` algorithm takes the root key and a false positive rate $p$
(chosen from some set of supported rates), and produces a *detection key*.  This
detection key can be applied to a flag ciphertext using `Test` to produce a
detection result.

This scheme should satisfy certain properties, formalizations of which can be
found in the paper:

###### Correctness.

Valid matches must always be detected by `Test`; i.e., there are no false negatives.

###### Fuzziness.

Invalid matches should produce false positives with probability approximately
$p$, as long as the flag ciphertexts and detection keys were honestly generated.

###### Detection Ambiguity.

An adversarial detector must be unable to distinguish between a true positive
and a false positive, as long as the flag ciphertexts and detection keys were
honestly generated.

The paper then defines two schemes that realize this functionality for
restricted false-positive rates of the form $p = 2^{-n}$.  Intuitively, these
schemes are constructed similarly to a Bloom filter: the `Flag` procedure
encrypts a number of `1` bits, and the `Test` procedure uses information in a
detection key to check whether some subset of them are `1`, returning `true` if
so and `false` otherwise.  The false positive probability is controlled by
extracting only a subset of the information in the root key into the detection key,
so that it can only check a subset of the bits encoded in the flag ciphertext.

## Sender and Receiver FMD

In the definition of FMD above, the receiver has control over how much detection
precision they delegate to a third party, because they choose the false positive
probability when they extract a detection key from their root key.  This fits
with the idea of [attenuating credentials][macaroons], and intuitively, it seems
correct that the receiver should control how much information they reveal to
their detector.  But it turns out that this feature makes it difficult to
usefully integrate FMD into a larger system, and it's unclear how the receiver
would make a principled choice of the detection precision they delegate.

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

However, this isn't possible if the false positive rate is chosen by the
receiver, because the receiver has no way to know in advance what rate will
produce a correctly sized stream of filtered messages.

One way to address this is to rename the original FMD definition as *Receiver
FMD* (R-FMD), and tweak it to obtain *Sender FMD* (S-FMD), in which the sender
chooses the detection probability.  Like R-FMD, S-FMD consists of a tuple of
algorithms `(KeyGen, Flag, Extract, Test)`.  Like R-FMD, `KeyGen` generates a
root key and a flag key, and `Test` takes a detection key and a flag ciphertext
and produces a detection result.  Unlike R-FMD, `Extract` produces a detection
key directly from the root key, and `Flag` takes the false positive rate $p$ and
the receiver's flag key to produce a flag ciphertext.  As discussed below, S-FMD
can be realized using tweaks to either of the R-FMD constructions in the
original paper.

In R-FMD, flag ciphertexts are universal with respect to the false positive
rate, which is applied to the detection key; in S-FMD, the false positive rate
is applied to the flag ciphertext and the detection key is universal.  (This
means there is no meaningful difference in capability between the root key and
the detection key, so the distinction is maintained just for ease of comparison
between the two variants).

Unlike R-FMD, S-FMD allows detection precision to be adaptive, by having senders
use a (consensus-determined) false positive parameter.  This parameter should
vary as the global message rates vary, so that filtered message streams have a
bounded rate, and it should be the same for all users, so that messages cannot
be distinguished by their false positive rate.

## Considerations for Sender FMD

- How should the false positive rate be determined? In some epoch, let $p$
be the false positive rate, $N$ be the total number of messages, $M$ be the
number of true positives for some detection key, and $D$ be the number of
detections for that detection key.  Then
$$
E[D] = M + p(N-M) = pN + M(1-p),
$$
and ideally $p$ should be chosen so that:
  1. $E[D]$ is bounded above;
  2. When $M$ is within the range of "normal use", $E[D]$ is close enough to
  $pN$ that it's difficult for a detector to distinguish (what does this mean
  exactly?);

- The notion of detection ambiguity only requires that true and false
positives be ambiguous in isolation. In practice, however, a detector has
additional context: the total number of messages, the number of detected
messages, and the false positive probability. What's the right notion in this
context?

- What happens when an adversary manipulates $N$ (diluting the global
message stream) or $M$ (by sending extra messages to a target address)?  There
is some analogy here to [flashlight attacks][flashlight], although with the
critical difference that flashlight attacks on decoy systems degrade privacy of
the transactions themselves, whereas here the scope is limited to transaction
detection.

- If a detector has detection keys for both the sender and receiver of a
transaction, they will detect the corresponding message with both keys with
probability $1$, relative to a base rate of probability $p^2$.  How does this
affect their information gain?  How does this change as the detector has not
just two keys, but some proportion of all detection keys?  How much more of the
transaction graph could they infer?

- How are detection keys derived and/or shared, so that they can actually be
used by participants in the protocol?

## Realizing Sender FMD

- [ ] Fill in details in this section

For FMD1, `Extract` becomes the identity, and `Flag` only produces the first `n` ciphertexts.  For FMD2, `Extract` becomes the identity, and `Flag` has $i \in [n]$ rather than $i \in [\gamma]$.

## Acknowledgements

Thanks to George Tankersley and Sarah Jamie Lewis for discussions on this topic
(and each independently suggesting the modifications to realize S-FMD), and to
Gabrielle Beck for discussions about the paper and ideas about statistical
attacks.

[^1]: The paper calls these the secret and public keys respectively, but we
avoid this in favor of capability-based terminology that names keys according to
the precise capability they allow.


[fmd-paper]: https://eprint.iacr.org/2021/089
[macaroons]: https://static.googleusercontent.com/media/research.google.com/en//pubs/archive/41892.pdf
[flashlight]: https://www.zfnd.org/blog/blockchain-privacy/#flashlight-attack