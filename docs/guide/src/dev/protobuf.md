# Maintaining protobuf specs

The Penumbra project dynamically generates code for interfacing
with [gRPC]. The following locations within the repository
are relevant:

  * `proto/penumbra/**/*.proto`, the developer-authored spec files
  * `crates/proto/src/gen/*.rs`, the generated Rust code files
  * `proto/go/gen/**/*.pb.go`, the generated Go code files

We use [buf] to auto-publish the protobuf schemas at
[buf.build/penumbra-zone/penumbra][BSR], and to generate Go and Rust packages.

Our custom tooling for generating the Rust files will also shape the Serde implementations
of the derived Rust types to have more favorable JSON output (such as rendering
addresses as [Bech32]-encoded strings).

## Installing buf

The `buf` tool is required to manage the codegen for protobuf definitions.
the [Buf Schema Registry](https://buf.build.penumbra-zone/penumbra). Visit
the [buf download page](https://buf.build/docs/installation/) to obtain a version.
After installing, run `buf --version` and confirm you're running at least
`1.17.0` (or newer).

## Building protos

Switch to the [proto] directory and run:

```shell
cd proto/
buf generate
```

Then run `git status` to determine whether any changes were made.
The build process is deterministic, so regenerating multiple times
from the same source files should not change the output.
A possible exception to this rule is if `prost` makes a superficial
change to the output that isn't substantive.

If the generated output would change in any way, CI will
fail, prompting the developer to commit the changes.

## Updating buf lockfiles
We pin specific versions of upstream Cosmos deps in the buf lockfile
for our proto definitions. Doing so avoids a tedious chore of needing
to update the lockfile frequently when the upstream BSR entries change.
We should review these deps periodically and bump them, as we would any other dependency.

```shell
cd proto/penumbra
# edit buf.yaml to remove the tags, i.e. suffix `:<tag>`
buf mod update
```

then commit and PR in the results.

[`protoc` website]: https://grpc.io/docs/protoc-installation/#install-pre-compiled-binaries-any-os
[proto-compiler]: https://github.com/penumbra-zone/penumbra/tree/main/tools/proto-compiler
[gRPC]: https://grpc.io/
[BSR]: https://buf.build/penumbra-zone/penumbra
[buf]: https://buf.build/
[Bech32]: https://en.bitcoin.it/wiki/Bech32
