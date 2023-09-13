# Making RPC requests

`pclientd` exposes a GRPC and GRPC-web endpoint at its `bind_addr`. Several
services are available.

To interactively explore requests and responses, try running [GRPCUI] locally or
using [Buf Studio][buf-studio] in the browser. Buf Studio has a nicer user
interface but does not (currently) support streaming requests.

[GRPCUI]: https://github.com/fullstorydev/grpcui
[buf-studio]: https://buf.build/studio/penumbra-zone/penumbra/penumbra.client.v1alpha1.ObliviousQueryService/ChainParameters?selectedProtocol=grpc-web&target=http%3A%2F%2Flocalhost%3A8081

## Accessing public chain state

`pclientd` has an integrated GRPC proxy, routing requests about public chain
state to the fullnode it's connected to.

Documentation on these RPCs is available here:
- [`SpecificQueryService`](https://buf.build/penumbra-zone/penumbra/docs/main:penumbra.client.v1alpha1#penumbra.client.v1alpha1.SpecificQueryService)
- [`ObliviousQueryService`](https://buf.build/penumbra-zone/penumbra/docs/main:penumbra.client.v1alpha1#penumbra.client.v1alpha1.ObliviousQueryService)
- [`TendermintProxyService`](https://buf.build/penumbra-zone/penumbra/docs/main:penumbra.client.v1alpha1#penumbra.client.v1alpha1.TendermintProxyService)

Note: in the future, these RPCs will be refactored by component (e.g., DEX queries through a `DexQuery`), etc.

## Accessing private chain state

Access to a user's private state is provided by the [`ViewService` RPC](https://buf.build/penumbra-zone/penumbra/docs/main:penumbra.view.v1alpha1#penumbra.view.v1alpha1.ViewProtocolService).

In addition to ordinary queries, like
[`Balances`](https://buf.build/penumbra-zone/penumbra/docs/main:penumbra.view.v1alpha1#penumbra.view.v1alpha1.ViewProtocolService.Balances),
which gets a user's balances by account, the RPC also contains utility methods
that allow computations involving cryptography.  For instance, the
[`AddressByIndex`](https://buf.build/penumbra-zone/penumbra/docs/main:penumbra.view.v1alpha1#penumbra.view.v1alpha1.ViewProtocolService.AddressByIndex)
request computes a public address from an account index, and the
[`IndexByAddress`](https://buf.build/penumbra-zone/penumbra/docs/main:penumbra.view.v1alpha1#penumbra.view.v1alpha1.ViewProtocolService.IndexByAddress)
request decrypts an address to its private index.

Finally, the view service can plan and build transactions, as described in [the next section](./build_transaction.md).

## Requesting transaction authorization

If `pclientd` was configured in custody mode, it exposes a [`CustodyService`](https://buf.build/penumbra-zone/penumbra/docs/main:penumbra.custody.v1alpha1#penumbra.custody.v1alpha1.CustodyProtocolService).

This allows authorization of a `TransactionPlan`, as described in [the next section](./build_transaction.md).
