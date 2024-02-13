# Resources

This page links to various resources that are helpful for working with and
understanding Penumbra.

### Getting started

  * The primary communication hub is our [Discord]; click the link to join the
discussion there.
  * Documentation on how to use `pcli`, how to run `pd`, and how to do development can be found at [guide.penumbra.zone][guide].

### For developers

  * The protocol specification is rendered at [protocol.penumbra.zone][protocol].
  * The API documentation is rendered at [rustdoc.penumbra.zone][rustdoc].
  * The protobuf documentation is rendered at [buf.build/penumbra-zone/penumbra][protobuf].
  * The current sprint progress is tracked on a [Github project board](https://github.com/orgs/penumbra-zone/projects/23/views/3) (hint: 🔖 this).
  * The development history by testnet can be found at [another Github board](https://github.com/orgs/penumbra-zone/projects/17).

## Tools

  * Public testnet `pd` endpoint: `https://grpc.testnet.penumbra.zone` This URL won't work in a web browser, as the service speaks gRPC.
  * Public testnet `cometbft` API endpoint: [https://rpc.testnet.penumbra.zone](https://rpc.testnet.penumbra.zone)
  * Block explorer: [https://cuiloa.testnet.penumbra.zone](https://cuiloa.testnet.penumbra.zone)
  * Metrics: [https://grafana.testnet.penumbra.zone](https://grafana.testnet.penumbra.zone)

For all those URLs, there's also a `preview` version available, e.g. `https://grpc.testnet-preview.penumbra.zone`,
that tracks the latest tip of the git repo, rather than the current public testnet.

[Discord]: https://discord.gg/hKvkrqa3zC
[protocol]: https://protocol.penumbra.zone
[rustdoc]: https://rustdoc.penumbra.zone
[guide]: https://guide.penumbra.zone
[protobuf]: https://buf.build/penumbra-zone/penumbra

### Talks and presentations

These talks were given at various conferences and events,
describing different aspects of the Penumbra ecosystem.

* [DevCon 2022: Building a Private DEX with ZKPs and Threshold Cryptography](https://archive.devcon.org/archive/watch/6/penumbra-building-a-private-dex-with-zkps-and-threshold-cryptography/?tab=YouTube)
* [ZK8: How to build a private DEX](https://www.youtube.com/watch?v=-ap9ja36EYU)
* [ZK8: Tiered Merkle Topiary in Rust](https://www.youtube.com/watch?v=mHoe7lQMcxU)
* [ZK Whiteboard Session: ZK Swaps](https://www.youtube.com/watch?v=ziUZyQmHh4c)
* [MEV Day: Minimizing MEV with sealed-input batch swaps](https://www.youtube.com/watch?v=oPIOIW2tvL4)
* [Nebular Summit: Creating Performant Interchain Privacy At Scale](https://www.youtube.com/watch?v=EEUKPrno3u4)
* [Interchain FM: Penumbra, Zero-Knowledge Decentralized Exchange](https://interchain.fm/episodes/penumbra-zero-knowledge-decentralized-exchange/transcript)
