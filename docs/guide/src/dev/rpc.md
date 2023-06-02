# Working with gRPC for Penumbra

The Penumbra [`pd`](../pd.md) application exposes a [gRPC] service for integration
with other tools, such as [`pcli`](../pcli.md) or the [web extension](../extension.md).
A solid understanding of how the gRPC methods work is helpful when
building software that interoperates with Penumbra.

## Using gRPC UI

The Penumbra Labs team runs [gRPC UI] instances for testnet deployments:

  * For the current testnet: [https://grpcui.testnet.penumbra.zone](https://grpcui.testnet.penumbra.zone)
  * For rapid-release preview: [https://grpcui.testnet-preview.penumbra.zone](https://grpcui.testnet-preview.penumbra.zone)

You can use this interface to perform queries against the relevant chain.
It's also possible to run gRPC UI locally on your machine, to connect
to a local devnet.

## Using Buf Studio

The [Buf Studio](https://studio.buf.build) webapp provides a polished GUI
and [comprehensive documentation](https://buf.build/docs/bsr/studio). However,
a significant limitation for use with Penumbra is that it lacks
support for streaming requests, such as [`penumbra.client.v1alpha1.CompactBlockRangeRequest`](https://buf.build/penumbra-zone/penumbra/docs/main:penumbra.client.v1alpha1#penumbra.client.v1alpha1.CompactBlockRangeRequest).

To get started with Buf Studio, you can use the publicly available gRPC endpoint
from the testnet deployments run by Penumbra Labs:

  * For the current testnet, use `https://grpc.testnet.penumbra.zone`
  * For ephemeral devnets, use `https://grpc.testnet-preview.penumbra.zone`

Set the request type to **gRPC-web** at the bottom of the screen.
You can then select a **Method** and explore the associated services.
Click **Send** to submit the request and view response data in the right-hand pane.

## Interacting with local devnets

Regardless of which interface you choose, you can connect to an instance of `pd` running
on your machine, which can be useful while adding new features.
First, make sure you've [joined a testnet](https://guide.penumbra.zone/main/pd/join-testnet.html)
by setting up a node on your local machine. Once it's running, you can connect directly
to the pd port via `http://localhost:8080`.

Alternatively, you can use `pclientd`. First, make sure you've [configured pclientd locally](https://guide.penumbra.zone/main/pcli/pclientd.html)
with your full viewing key. Once it's running, you can connect directly
to the pclient port via `http://localhost:8081`.

[gRPC]: https://grpc.io/docs/what-is-grpc/introduction/
[gRPC UI]: https://github.com/fullstorydev/grpcui
