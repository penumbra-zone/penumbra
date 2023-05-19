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

# Transfers into Penumbra

IBC transfer mechanics are specified in [ICS20]. The
[`FungibleTokenPacketData`][ftpd] packet describes the transfer:
```
FungibleTokenPacketData {
    denomination: string,
    amount: uint256,
    sender: string,
    receiver: string,
}
```

The `sender` and `receiver` fields are used to specify the sending account on
the source chain and the receiving account on the destination chain. However,
for inbound transfers, the destination chain is Penumbra, which has no
accounts. Instead, token transfers into Penumbra create an
`OutputDescription` describing a new shielded note with the given amount and
denomination, and insert an encoding of the description itself into the
`receiver` field.

[ICS20]: https://github.com/cosmos/ibc/blob/master/spec/app/ics-020-fungible-token-transfer/README.md
[ftpd]: https://github.com/cosmos/ibc/blob/master/spec/app/ics-020-fungible-token-transfer/README.md#data-structures

## Handling Bridged Assets 

Penumbra's native state model uses notes, which contain an amount of a
particular asset. Amounts in Penumbra are 128-bit unsigned integers, in order
to support assets which have potentially large base denoms (such as Ethereum).
When receiving an IBC transfer, if the amount being transferred is greater than
`u128`, we return an error. 

