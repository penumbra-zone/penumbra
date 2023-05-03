# Maintaining protobuf specs

The Penumbra project dynamically generates code for interfacing
with [gRPC]. The following locations within the repository
are relevant:

  * `proto/proto/penumbra/**/*.proto`, the developer-authored spec files
  * `proto/src/gen/*.rs`, the generated Rust code files
  * `tools/proto-compiler/`, the build logic for generated the Rust code files

We use [buf] to auto-publish the protobuf schemas at
[buf.build/penumbra-zone/penumbra][protobuf], and to generate Go and Typescript packages.
The Rust code files are generated with our own tooling, located at `tools/proto-compiler`.

Our custom tooling for generating the Rust files will also shape the Serde implementations
of the derived Rust types to have more favorable JSON output (such as rendering
addresses as [Bech32]-encoded strings).

## Installing protoc

The `protoc` tool is required to generate our protobuf specs via `tools/proto-compiler`.
Obtain the most recent pre-compiled binary from the [`protoc` website].
After installing, run `protoc --version` and confirm you're running
at least `3.21.8` (or newer). Don't install `protoc` from package managers
such as `apt`, as those versions are often outdated, and will not work
with Penumbra.

## Installing buf

The `buf` tool is required to update lockfiles used for version management in
the [Buf Schema Registry](https://buf.build.penumbra-zone/penumbra). Visit
the [buf download page](https://buf.build/docs/installation/) to obtain a version.
After installing, run `buf --version` and confirm you're running at least
`1.17.0` (or newer).

## Building protos

Switch to the [proto-compiler] directory and run the tool:

```shell
cd tools/proto-compiler
cargo run
```

Then run `git status` to determine whether any changes were made.
The build process is deterministic, so regenerating multiple times
from the same source files should not change the output.
A possible exception to this rule is if `prost` makes a superficial
change to the output that isn't substantive.

If the generated output would change in any way, CI will
fail, prompting the developer to commit the changes.

## Updating buf lockfiles
Occasionally the lockfile pinning protobuf dependencies will drift from latest,
either due to changes in upstream Cosmos deps, or changes in our own. To update:

```shell
cd proto/proto
buf mod update
```

then commit and PR in the results. Eventually we hope to remove the need for this chore;
see [GH2321](https://github.com/penumbra-zone/penumbra/issues/2321) and
[GH2184](https://github.com/penumbra-zone/penumbra/issues/2184) for details.

[`protoc` website]: https://grpc.io/docs/protoc-installation/#install-pre-compiled-binaries-any-os
[proto-compiler]: https://github.com/penumbra-zone/penumbra/tree/main/tools/proto-compiler
[gRPC]: https://grpc.io/
[protobuf]: https://buf.build/penumbra-zone/penumbra
[buf]: https://buf.build/
[Bech32]: https://en.bitcoin.it/wiki/Bech32
