# Building Transactions

Using the view and custody services to construct a transaction has four steps.

## Plan the Transaction

Using the [`TransactionPlanner`](https://buf.build/penumbra-zone/penumbra/docs/60489c71c3b64f179b2537b24a587abe:penumbra.view.v1alpha1#penumbra.view.v1alpha1.ViewProtocolService.TransactionPlanner) RPC in the view service, compute a `TransactionPlan`.

This RPC translates a general intent, like "send these tokens to this address" into a fully deterministic plan of the exact transaction, with all spends and outputs, all blinding factors selected, and so on.

## Authorize the Transaction

With a `TransactionPlan` in hand, use the
[`Authorize`](https://buf.build/penumbra-zone/penumbra/docs/main:penumbra.custody.v1alpha1#penumbra.custody.v1alpha1.CustodyProtocolService.Authorize)
RPC to request authorization of the transaction from the custody service.

Note that authorization happens on the cleartext transaction _plan_, not the shielded transaction, so that the custodian can inspect the transaction before signing it.

## Build the Transaction

With the `TransactionPlan` and `AuthorizationData` in hand, use the [`WitnessAndBuild`](https://buf.build/penumbra-zone/penumbra/docs/60489c71c3b64f179b2537b24a587abe:penumbra.view.v1alpha1#penumbra.view.v1alpha1.ViewProtocolService.WitnessAndBuild) RPC to have the view service build the transaction, using the latest witness data to construct the ZK proofs.

## Broadcast the Transaction

With the resulting shielded `Transaction` complete, use the [`BroadcastTransaction`](https://buf.build/penumbra-zone/penumbra/docs/60489c71c3b64f179b2537b24a587abe:penumbra.view.v1alpha1#penumbra.view.v1alpha1.ViewProtocolService.BroadcastTransaction)
request to broadcast the transaction to the network.

The `await_detection` parameter will wait for the transaction to be confirmed
on-chain. Using `await_detection` is a simple way to ensure that different
transactions can't conflict with each other.