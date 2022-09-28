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

Here messages are individual *outputs*, each of which are associated with a single
note.

## Considered Attacks

An attacker can perform passive or active traffic analysis in order to learn
about user behavior on a statistical basis. Below we describe some possible
attacks assuming that an attacker knows:

- $p$, the false positive probability, since this is set by the current precision $n$, where $p=2^{-n}$,
- $N$, the total number of messages, since this is public,
- $D$, the number of detected messages for the targeted user, by being a
malicious detector or observing network traffic.

### Passive set intersection

This is a passive attack wherein an attacker compares the set of detections for
each user. An attacker can learn on a statistical basis, e.g. if a message A only appears in the detections for users X and Y, then it must be either user X or Y.

This is partially mitigated by users that do not opt-in to FMD and instead
download all state updates. For example, in the above example,
the message A will appear in the detections for user X, Y, and _all_ users that
download all state updates.

### Manipulating total traffic on the network

An attacker can artificially increase the traffic $N$ on the network by
sending transactions with many "fake" outputs, e.g. to another address the attacker
controls. The only cost to the attacker to do this are transaction fees.

Then the total number of messages $N$ consists of:

$N = A + M + G$

where $A$ are messages introduced by the attacker, $M$ are the targeted user's
messages, and $G$ are messages from all other honest users. In the limit where
$G=0$, even if very low precision is used, then the attacker will be able to identify
all true positives associated with the user.

TODO: Eqn 3 in https://arxiv.org/pdf/2109.06576.pdf 

Proposed additional mitigations:
- [ ] Scale transaction fees by the number of outputs in a transaction [^1]. This
imposes a cost to an attacker of creating many dummy/fake outputs that they control.

### Sending additional messages to a targeted address

An attacker can send additional messages to the targeted user to increase $M$.
There is some analogy here to [flashlight attacks][flashlight], although with the
critical difference that flashlight attacks on decoy systems degrade privacy of
the transactions themselves, whereas here the scope is limited to transaction
detection.

## Caveats/Limitations

Since the false positive rate is fixed for a given period of time (e.g. epoch),
a passive observer can learn the FMD user's transaction volume, since for e.g. high
volume users the set of matched transactions will be much higher than a user with
no or few transactions in a given epoch.

[flashlight]: https://www.zfnd.org/blog/blockchain-privacy/#flashlight-attack

[^1]: It may
make sense to scale transaction fees by the number of actions in a transaction
anyway in order to "pay" for the cost of proof verification time.
