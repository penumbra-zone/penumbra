# Parameter Considerations

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

[flashlight]: https://www.zfnd.org/blog/blockchain-privacy/#flashlight-attack