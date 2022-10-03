# S-FMD Threat Model

## Open vs Closed World Setting

The previous literature on FMD (e.g. [Beck et al. 2021], [Seres et al. 2021]) focuses on a model we will
call the *closed world* setting where:

* A single untrusted server performs FMD on behalf of users. This server has all
detection keys.
* All users in the messaging system use FMD.

This is appropriate for a messaging application using a single centralized server
where FMD is a requirement at the protocol-level.

However, Penumbra operates in what we will call the *open world* setting in which:

* Multiple untrusted servers perform FMD on behalf of users, i.e. there is no
single centralized detection server with all detection keys.
* Not all users use FMD: in Penumbra FMD is opt-in. A fraction of Penumbra users will download all messages, and never provide detection keys to a third party.

A further difference in Penumbra is that the total number of distinct *users* is
unknown. Each user can create multiple addresses, and they choose whether or not
to derive a detection key for a given address.

## Assumptions

We assume the FMD construction is secure and the properties of correctness,
fuzziness (false positives are detected with rate $p$), and detection ambiguity
(the server cannot distinguish between false and true positives) hold as described
in the previous section.

All parties malicious or otherwise can access the public chain state, e.g. the
current and historical global network activity rates and the current and
historical network false positive rates.

Each *detection server*  also observes which messages are detected, but only
for the detection keys they have been given. No detection server has detection
keys for all users, since only a subset of users opt-in to FMD.

A *malicious detection server* can passively compare the detected sets between
users. They can also perform active attacks, sending additional messages to
artificially increase global network volume, or to the user to increase
the number of true positives. We consider these attacks in the next section.
In the open world setting, multiple malicious detection servers may be
attempting to boost traffic globally, or may target the same user. Malicious
detection servers may collude, sharing the sets of detection keys they have in
order to jointly maximize the information they learn about network activity.

We assume no detection servers have access to sender metadata, as would be the
case if participants routed their traffic through a network privacy layer such
as Tor.

A *passive eavesdropper* can observe the network traffic between recipient and
detection server, and attempt to infer the number of messages they have
downloaded. We assume the connection between the detection server and recipient is
secured using HTTPS.

[Beck et al. 2021]: https://eprint.iacr.org/2021/089
[Seres et al. 2021]: https://arxiv.org/pdf/2109.06576.pdf
