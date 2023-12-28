# Maintaining protobuf specs

The Penumbra project dynamically generates code for interfacing
with [gRPC]. The following locations within the repository
are relevant:

  * `proto/penumbra/**/*.proto`, the developer-authored spec files
  * `crates/proto/src/gen/*.rs`, the generated Rust code files
  * `proto/go/**/*.pb.go`, the generated Go code files
  * `tools/proto-compiler/`, the build logic for generating the Rust code files

We use [buf] to auto-publish the protobuf schemas at
[buf.build/penumbra-zone/penumbra][protobuf], and to generate Go and Typescript packages.
The Rust code files are generated with our own tooling, located at `tools/proto-compiler`.

## Installing protoc

The `protoc` tool is required to generate our protobuf specs via `tools/proto-compiler`.
We mandate the use of a specific major version of the `protoc` tool, to make outputs
predictable. Currently, the supported version is `24.x`. Obtain the most recent
pre-compiled binary from the [`protoc` website] for that major version.
After installing, run `protoc --version` and confirm you're running
at least `24.4` (or newer). Don't install `protoc` from package managers
such as `apt`, as those versions are often outdated, and will not work
with Penumbra.

To install the protoc tool from the zip file, extract it to a directory on your PATH:

```shell
unzip protoc-24.4-linux-x86_64.zip -d ~/.local/
```

## Installing buf

The `buf` tool is required to update lockfiles used for version management in
the [Buf Schema Registry](https://buf.build/penumbra-zone/penumbra). Visit
the [buf download page](https://buf.build/docs/installation/) to obtain a version.
After installing, run `buf --version` and confirm you're running at least
`1.27.0` (or newer).

## Building protos

From the top-level of the git repository:

```shell
./deployments/scripts/protobuf-codegen
```

Then run `git status` to determine whether any changes were made.
The build process is deterministic, so regenerating multiple times
from the same source files should not change the output.

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

Then commit and PR in the results.

[`protoc` website]: https://protobuf.dev/downloads/
[proto-compiler]: https://github.com/penumbra-zone/penumbra/tree/main/tools/proto-compiler
[gRPC]: https://grpc.io/
[protobuf]: https://buf.build/penumbra-zone/penumbra
[buf]: https://buf.build/
