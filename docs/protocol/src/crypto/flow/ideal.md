# Ideal Functionality

Flow encryption is ultimately "just" a form of additively homomorphic threshold
encryption.  However, usefully integrating that primitive into a ledger and
byzantine consensus system requires a number of additional properties, and it's
useful to have a single name that refers to that complete bundle of properties.

In this section, we sketch what those properties are, and the algorithms that a
flow encryption construction should provide, separately from our instantiation
of flow encryption, `eddy`, described in the next section.

## Participants

There are three *participants* in flow encryption:

* the *ledger*, which is assumed to be a BFT broadcast mechanism that accepts messages based on some application-specific rules;
* *users*, who encrypt their individual contributions to some flow of value and submit them to the ledger;
* *decryptors*, who aggregate encryptions posted to the ledger and decrypt the batched flow.

In this description, we refer to *decryptors*, rather than *validators*, in
order to more precisely name their role in flow encryption.  In Penumbra, the
decryptors are also the validators who perform BFT consensus to agree on the
ledger state.  However, this is not a requirement of the construction; one could
also imagine a set of decryptors who jointly decrypted data posted to a common,
external ledger such as Juno or Ethereum.

We include the *ledger* as a participant to highlight that a BFT broadcast
mechanism is assumed to be available, and to explicitly define when
communication between other participants happens via the ledger, and is thus
already BFT, and when communication happens out-of-band, and must be made BFT.

Importantly, *users* are not required to have any online interactions or perform
any protocols with any other participant.  They must be able to encrypt and
submit their contribution noninteractively.  This is essential for the intended
role of flow encryption in allowing private interaction with public shared
state: the decryption protocol acts as a form of specialized, lightweight MPC,
and the encryption algorithm acts as a way for users to delegate participation
in that MPC to a set of decryptors, without imposing wider coordination
requirements on the users of the system.

## Algorithms and Protocols

###### `FlowEnc/DistKeyGen`

This is a multi-round protocol performed by the decryptors.  On input a set of
$n$ decryptors and a decryption threshold $1 \leq t \leq n$, this protocol
outputs a common threshold encryption key $D$, a private key share $d_p$ for
each participant $p$, and a public set of commitments to decryption shares
$\phi_p$.  The exact details of the DKG, its number of rounds, etc., are left to
the specific instantiation.

###### `FlowEnc/Encrypt`

This is an algorithm run by users. On input an encryption key $D$ and the opening
$(v, \widetilde{v})$
to a Pedersen commitment $C$, this algorithm outputs a
ciphertext $E = \operatorname{Enc}(v)$ and a proof $\pi_{\operatorname{Enc}}$ which establishes that
$E = \operatorname{Enc}(v)$ is well-formed and is consistent, in the sense that it
encrypts the same value committed to by $C = \operatorname{Commit}(v, \widetilde{v})$.

We assume that all ciphertexts are submitted to the ledger, which verifies 
$\pi_{\operatorname{Enc}}$ along with any other application-specific validity
rules.  These rules need to check, among other things, that $v$ is the *correct*
value to encrypt, and the Pedersen commitment provides a flexible way to do so.
The consistency property established by $\pi_{\operatorname{Enc}}$ allows
application-specific proof statements about (a commitment to) $v$ to extend to
the ciphertext $\operatorname{Enc}(v)$.

###### `FlowEnc/Aggregate`

This algorithm describes how to add ciphertexts $\sum_i \operatorname{Enc}(v_i)$ to
output the encryption $\operatorname{Enc}(\sum_i v_i)$ of their sum.

We also assume that, because ciphertexts are posted to the ledger, all
decryptors have a consistent view of available ciphertexts, and of the
application-specific rules concerning which contributions should be batched
together, over what time period, etc., so that they also have a consistent view
of the aggregated ciphertext to decrypt.  

In the case of same-block decryption,
this assumption requires some care to integrate with the process for coming to
consensus on the block containing transactions to batch and decrypt, but this is
out-of-scope for flow encryption itself.  See [Flow Encryption and
Consensus](../../protocol/flow-consensus.md) for details on this aspect in
Penumbra specifically.

###### `FlowEnc/PreDecrypt`

On input ciphertext $E = \operatorname{Enc}(w)$ and key share $d_p$, this algorithm outputs a decryption share $S_p$ and a decryption share integrity proof $\pi_p$.

###### `FlowEnc/Decrypt`

On input a ciphertext $E = \operatorname{Enc}(w)$ and *any set* of $k$ decryption
shares and proofs 
$(S_1, \pi_1), \ldots, (S_k, \pi_k)$ 
with
$t \leq k \leq n$,
output $w$ if at least $t$ of $k$ decryption shares were valid.

## Properties

We require the following properties of these algorithms:

###### Additive Homomorphism

Ciphertexts must be additively homomorphic:
$$
\sum_i \operatorname{Enc}(v_i) = \operatorname{Enc}\left(\sum_i v_i\right)
$$

###### Value Integrity

The flow encryption must ensure conservation of value, from the individual
users' contributions all the way to the decrypted batch total.  Establishing
value integrity proceeds in a number of steps:

1. Application-specific logic proves that each user's contribution value $v_i$ conserves value according to the application rules, by proving statements about the commitment $C_{i}$ to $v_{i}$.
2. The encryption proof $\pi_{\operatorname{Enc}}$ extends integrity from $C_{i}$ to $E_{i}$.
3. Integrity extends to the aggregation $E = \sum_i E_i$ automatically, since the aggregation can be publicly recomputed by anyone with access to the ledger.
4. The decryption share integrity proofs extend integrity from $E$ to $w$.  This requires that, if $w$ is the result of decryption using valid decryption shares, than $E = \operatorname{Enc}(w)$.
5. Publication of (any) decryption transcript allows any participant to check the end-to-end property that $w = \sum_i v_i$ for (application-)valid $v_i$.

###### Decryption Robustness

The decryption process must succeed after receiving any $t$ valid decryption
shares, so that any decryptor who can receive messages from $t$ honest
decryptors must be able to compute the correct plaintext.

Unlike the DKG, where we do not impose a constraint on the number of rounds, we
require that decryption succeed with only one round of communication.  This
allows us to integrate the decryption process with the consensus process, so
that in the case where decryptors are validators, they can jointly learn the
batched flow at the same time as they finalize a block and commit its state
changes.  This property is important to avoid requiring a pipelined execution
model.  For more details, see [Flow Encryption and
Consensus](../../protocol/flow-consensus.md).

Note that we do not require that any *specific* subset of decryption shares is
used to get the (unique) decryption result in `FlowEnc/Decrypt`.  This permits a
streaming implementation where all $n$ decryptors participate, but decryption
completes for each participant as soon as they receive $t$ valid shares,
ensuring that decryption is [not bottlenecked on the slowest
participant][tail-at-scale].

###### DKG Verifiability

The DKG must be *verifiable*:
participants (decryptors) must be able to verify that counterparty participants
(other decryptors) are contributing to the DKG honestly, without the use of a
trusted dealer. This can be achieved using something similar to [Feldman's
Verifiable Secret Sharing][feldman] protocol, where each participant shares a
commitment to their share which is visible to all other participants. In
addition, our DKG must be able to tolerate *rogue-key attacks*: that is, it
must tolerate the instance where a validator maliciously chooses their share
based on the value of the other validator's shares in order to cancel out other
validator's keyshares and gain unilateral control over the resulting DKG key.
One way this can be prevented is by each validator producing a proof of
knowledge of their secret share.

###### DKG Robustness

The DKG must have *robustness*. The DKG should be able to tolerate a byzantine
threshold of decryptors intentionally refusing to participate in the DKG round,
or intentionally contributing malformed shares during DKG execution, without
requiring a full restart of the DKG protocol. This is due to DoS concerns: with
a naive, non-robust implementation, a single malicious decryptor could
potentially indefinitely delay the beginning of an epoch by refusing to
participate in DKG or by contributing invalid shares.

[tail-at-scale]: https://cseweb.ucsd.edu/classes/sp18/cse291-c/post/schedule/p74-dean.pdf


[ethdkg]: https://eprint.iacr.org/2019/985
[feldman]: https://www.cs.umd.edu/~gasarch/TOPICS/secretsharing/feldmanVSS.pdf
[gennaro]: http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.134.6445&rep=rep1&type=pdf
[GJMMST]: https://eprint.iacr.org/2021/005.pdf
[frost]: https://eprint.iacr.org/2020/852.pdf
