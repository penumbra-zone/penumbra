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