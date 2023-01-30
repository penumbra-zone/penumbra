# Penumbra Guide

[Penumbra] is a fully shielded zone for the Cosmos ecosystem, allowing anyone to
securely transact, stake, swap, or marketmake without broadcasting their
personal information to the world.

This site contains documentation on how to use, deploy, and develop the Penumbra
software.  The description of the protocol itself can be found in the [protocol
specification][protocol], and the API documentation can be found
[here][rustdoc].

## Test networks

Penumbra is a decentralized protocol, so Penumbra Labs is [building in
public][how-were-building], launching (and crashing) lots of work-in-progress
testnets to allow community participation, engagement, and feedback.

Currently, Penumbra only has a command line client, `pcli` (pronounced
"pickle-y"), which bundles all of the client components in one binary, and a
chain-scanning daemon, `pclientd`, which runs just the view service, without spend
capability.  To get started with the Penumbra test network, all that's required
is to download and build `pcli`, as described in
[Installation](./pcli/install.md).

The Penumbra node software is the Penumbra daemon, `pd`.  This is an ABCI
application, which must be driven by Tendermint, so a Penumbra full node
consists of both a `pd` instance and a `tendermint` instance.

The basic architecture of Penumbra is as follows:

```text
          ╭   ┌───────┐
  spending│   │custody│
capability│   │service│
          ╰   └───────┘
               ▲     │
               │tx   │auth
               │plan │data
               │     ▼
          ╭   ┌───────┐
   viewing│   │wallet │ tx submission
capability│   │logic  │────────┐
          │   └───────┘        │
          │    ▲               │
          │    │view private state
          │    │               │
          │    │               │
          │   ┌───────┐        │
          │   │view   │        │
          │   │service│        │
          ╰   └───────┘        │
               ▲               │
               │sync private state
               │               │
          ╭ ┌──┼───────────────┼──────┐
    public│ │  │     Penumbra Fullnode│
     chain│ │  │               │      │
      data│ │  │               ▼      │
          │ │ ┌──┐ app   ┌──────────┐ │
          │ │ │pd│◀─────▶│tendermint│ │
          │ │ └──┘ sync  └──────────┘ │
          │ │               ▲         │
          ╰ └───────────────┼─────────┘
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

The custody service holds signing keys and is responsible for authorizing
transaction plans.  The view service holds viewing keys and scans the chain
state.  Wallet logic can query the view service to get information about what
funds are available, submit a transaction plan to the custody service for
signing, and then use the returned signatures to build the transaction and
submit it.

As a shielded chain, Penumbra's architecture is slightly different than a
transparent chain, because user data such as account balances, transaction
activity, etc., is not part of the public chain state.  This means that clients
need to synchronize with the chain to build a copy of the private user data they
have access to.  This logic is provided by the *view service*, which is bundled
into `pcli`, but can also be run as a standalone `pclientd` daemon.

Modeling authorization as an (asynchronous) RPC to a custody service means that
the client software is compatible with many different custody flows by default
-- an in-process "SoftHSM", a hardware wallet with user intervention, a cluster
of online threshold signers, an offline threshold signing process, etc.

[how-were-building]: https://penumbra.zone/blog/how-were-building-penumbra
[protocol]: https://protocol.penumbra.zone
[rustdoc]: https://rustdoc.penumbra.zone
[Penumbra]: https://github.com/penumbra-zone/penumbra
