# S-FMD Threat Model

## Open vs Closed World Setting

The previous literature on FMD ([fmd-paper],[Seres]) focuses on a model we will
call the "closed world" setting where:

* A single untrusted server performs FMD on behalf of users. 
* All users use FMD.

This is appropriate for a messaging application using a single centralized server
where FMD would be a requirement in the messaging protocol. 

However, Penumbra operate in what we will call the "open world" setting which
differs in the following ways:

* Multiple untrusted servers perform FMD on behalf of users, i.e. there is no single centralized server.
* Not all users use FMD. A fraction of users download all messages.

## Assumptions

We assume the FMD construction is secure and the properties of correctness,
fuzziness (false positives are detected with rate p), and detection ambiguity
(the server cannot distinguish between false and true positives) hold as described
in the previous section.

Each *detection server* knows all public data, including the current and
historical network activity and the current and historical false positive rates.
The server also observes the which messages are detected, but only
for the detection keys they have been given.

A *malicious detection server* can passively compare the detected sets between
users. They can also perform active attacks, sending additional messages to
themselves to artificially increase network volume, or to the user to increase
the number of true positives. We consider these attacks in the next section.
In the open world setting, multiple malicious detection servers may be
attempting to boost traffic globally, or targetting the same user, though this is
less likely.

We assume no detection servers have access to sender metadata, as would be the
case if participants routed their traffic through a network privacy layer such
as Tor.

## Open Questions

- [ ] Should we assume that malicious servers do not collude? What impact does this have? Or should we focus on the limit where there is a single server?

[fmd-paper]: https://eprint.iacr.org/2021/089
[Seres]: https://arxiv.org/pdf/2109.06576.pdf