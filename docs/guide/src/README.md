# Penumbra Guide

[Penumbra] is a fully shielded zone for the Cosmos ecosystem, allowing anyone to
securely transact, stake, swap, or marketmake without broadcasting their
personal information to the world.

This site contains documentation on how to use, deploy, and develop the Penumbra
software.  The description of the protocol itself can be found in the [protocol specification][protocol], and the API documentation can be found [here][rustdoc].

## Test networks

Penumbra is a decentralized protocol, so Penumbra Labs is [building in
public][how-were-building], launching (and crashing) lots of work-in-progress
testnets to allow community participation, engagement, and feedback.

The basic architecture of Penumbra is as follows:
```
┌─────────────────────────┐ ╮
│ ┌────┐   Penumbra Client│ │private
│ │pcli│                  │ │user
│ └────┘                  │ │data
│  ▲  │tx submission      │ │
│  │  └────────────┐      │ │
└──┼───────────────┼──────┘ ╯
   │private        │
   │state sync     │
┌──┼───────────────┼──────┐ ╮
│  │     Penumbra Fullnode│ │public
│  │               │      │ │chain
│  │               ▼      │ │data
│ ┌──┐ app   ┌──────────┐ │ │
│ │pd│◀─────▶│tendermint│ │ │
│ └──┘ sync  └──────────┘ │ │
│               ▲         │ │
└───────────────┼─────────┘ ╯
             .──│.
           ,'   │ `.
      .───;     │consensus
     ;          │sync
   .─┤          │   ├──.
 ,'             │       `.
;   Penumbra    │         :
:   Network  ◀──┘         ;
 ╲                       ╱
  `.     `.     `.     ,'
    `───'  `───'  `───'
```

Currently, Penumbra only has a command line client, `pcli` (pronounced
"pickle-y").  To get started with the Penumbra test network, all that's required
is to download and build `pcli`, as described in
[Installation](./pcli/install.md).

As a shielded chain, Penumbra's architecture is slightly different than a
transparent chain, because user data such as account balances, transaction
activity, etc., is not part of the public chain state.  This means that clients
need to synchronize with the chain to build a copy of the private user data they
have access to.  This logic is currently implemented in `pcli`.

The Penumbra node software is the Penumbra daemon, `pd`.  This is an ABCI
application, which must be driven by Tendermint, so a Penumbra full node
consists of both a `pd` instance and a `tendermint` instance.

[how-were-building]: https://penumbra.zone/blog/how-were-building-penumbra
[protocol]: https://protocol.penumbra.zone
[rustdoc]: https://rustdoc.penumbra.zone
