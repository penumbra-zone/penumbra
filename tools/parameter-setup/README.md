## How to redo the parameter setup

Note: this is a setup process only for testnet purposes since it is done by
a single party

* `cargo run` in this folder.

The verifying and proving keys for each circuit will be created in a serialized
form in the `proof-params/src/gen` folder. Those should be tracked using git lfs.

Each time you regenerate the parameters, increment the version of the
`penumbra-proof-params` crate.
