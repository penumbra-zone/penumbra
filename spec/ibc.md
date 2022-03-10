# IBC Protocol Implementation

Penumbra supports the [IBC protocol](https://ibcprotocol.org/) for
interoperating with other counterparty blockchains. Unlike most blockchains that currently deploy IBC, Penumbra is not based on the [Cosmos SDK](https://github.com/cosmos/cosmos-sdk). IBC as a protocol supports replication of data between two communicating blockchains. It provides basic building blocks for building higher-level cross chain applications, as well as a protocol specification for the most commonly used IBC applications, the [ICS-20 transfer](https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer) protocol.

Penumbra implements the core IBC protocol building blocks: [ICS-23 compatible state inclusion proofs](https://github.com/cosmos/ibc/tree/master/spec/core/ics-023-vector-commitments), [connections](https://github.com/cosmos/ibc/tree/master/spec/core/ics-003-connection-semantics) as well as [channels and packets](https://github.com/cosmos/ibc/tree/master/spec/core/ics-004-channel-and-packet-semantics).

## IBC Actions

In order to support the IBC protocol, Penumbra adds a single additional Action
`IBCAction`. an IBCAction can contain any of the IBC datagrams:

### ICS-003 Connections

* `ConnOpenInit`
* `ConnOpenTry`
* `ConnOpenAck`
* `ConnOpenConfirm`

### ICS-004 Channels and Packets

* `ChanOpenInit`
* `ChanOpenTry`
* `ChanOpenAck`
* `ChanOpenConfirm`
* `ChanCloseInit`
* `ChanCloseConfirm`
* `RecvPacket`
* `Timeout`
* `Acknowledgement`

These datagrams are implemented as protocol buffers, with the enclosing
`IBCAction` type using profobuf's `OneOf` directive to encapsulate all possible
IBC datagram types.

