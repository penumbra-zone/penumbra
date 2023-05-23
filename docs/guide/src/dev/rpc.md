# Using Buf Studio

The [Buf Studio](https://studio.buf.build) webapp provides a GUI
for interacting with [gRPC] endpoints. Developers can use it to explore Penumbra's
use of gRPC, for example, viewing transactions and transaction plans.

## Using the public testnet

To get started quickly, you can use the publicly available gRPC endpoint
from the testnet deployments run by Penumbra Labs.

  * For the current testnet, use `https://grpc.testnet.penumbra.zone`
  * For ephemeral devnets, use `https://grpc.testnet-preview.penumbra.zone`

Set the request type to **gRPC-web** at the bottom of the screen.
You can then select a **Method** and explore the associated services.
Click **Send** to submit the request and view response data in the right-hand pane.

## Using a local node

First, make sure you've [joined a testnet](https://guide.penumbra.zone/main/pd/join-testnet.html)
by setting up a node on your local machine. Once it's running, you can use 
[Buf Studio to connect directly to the `pd` port](https://studio.buf.build/penumbra-zone/penumbra/penumbra.client.v1alpha1.ObliviousQueryService/ChainParameters?selectedProtocol=grpc-web&target=http%3A%2F%2Flocalhost%3A8080),
via `http://localhost:8080`.

## Using local pclientd

First, make sure you've [configured pclientd locally](https://guide.penumbra.zone/main/pcli/pclientd.html)
with your full viewing key. Once it's running, you can use
[Buf Studio to connect directly to the `pclientd` port](https://studio.buf.build/penumbra-zone/penumbra/penumbra.client.v1alpha1.ObliviousQueryService/ChainParameters?selectedProtocol=grpc-web&target=http%3A%2F%2Flocalhost%3A8081),
via `http://localhost:8081`.

[gRPC]: https://grpc.io/docs/what-is-grpc/introduction/
