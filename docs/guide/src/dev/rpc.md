# Using Buf Studio

The [Buf Studio](https://studio.buf.build) webapp provides a GUI
for interacting with gRPC endpoints. Developers can use it to explore Penumbra's
use of gRPC, for example, viewing transactions and transaction plans.

## Using the public testnet

<!---
The use of the buf-studio agent is only necessary because the public gRPC endpoint
is HTTP-only; as soon as we have HTTPS support on the public testnet,
we can revise these docs to drop use of the agent, and connect directly to the HTTPS
endpoint. See for details: https://github.com/penumbra-zone/penumbra/issues/2341
--->

To get started quickly, you can use the publicly available gRPC endpoint
from the testnet deployments run by Penumbra Labs.

  * For the current testnet, use `http://testnet.penumbra.zone:8080`
  * For ephemeral devnets, use `http://testnet-preview.penumbra.zone:8080`

You must first download the [`buf` CLI](https://buf.build/docs/installation), and run it locally:

```bash
buf beta studio-agent --port 9090
```

Using a custom port when starting `studio-agent` as above helps to differentiate the agent's
service from the use of `8080/TCP` by `pd`. Then, in the Buf Studio web UI:

  1. Set the **Target** field to the appropriate URL for the environment you want to test against,
     e.g. `http://testnet.penumbra.zone:8080` (see above).
  2. Click **Set Agent URL** and enter `http://localhost:9090`.

You can then select a **Method** and explore the associated services.

## Using a local node

First, make sure you've [joined a testnet](https://guide.penumbra.zone/main/pd/join-testnet.html)
by setting up a node on your local machine. Once it's running, you can use 
[Buf Studio to connect directly to the `pd` port](https://studio.buf.build/penumbra-zone/penumbra/penumbra.client.v1alpha1.ObliviousQueryService/ChainParameters?selectedProtocol=grpc-web&target=http%3A%2F%2Flocalhost%3A8080),
via `http://localhost:8080`. With this method, there's no need to run the `buf` CLI for the `studio-agent`.

## Using local pclientd

First, make sure you've [configured pclientd locally](https://guide.penumbra.zone/main/pcli/pclientd.html)
with your full viewing key. Once it's running, you can use
[Buf Studio to connect directly to the `pclientd` port](https://studio.buf.build/penumbra-zone/penumbra/penumbra.client.v1alpha1.ObliviousQueryService/ChainParameters?selectedProtocol=grpc-web&target=http%3A%2F%2Flocalhost%3A8081),
via `http://localhost:8081`. With this method, as with using a local `pd`, there's no need to run the `buf` CLI for the `studio-agent`.
