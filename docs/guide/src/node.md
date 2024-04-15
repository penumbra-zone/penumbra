# Running a node

Running a node is not necessary to use the protocol. Both the web extension and
`pcli` are designed to operate with any RPC endpoint. However, we've tried to
make it as easy as possible to run nodes so that users can host their own RPC.

There are two kinds of Penumbra nodes:

* Penumbra fullnodes run `pd` and `cometbft` to synchronize and verify the entire chain state, as described in [_Running a node: `pd`_](./node/pd.md).
* Penumbra ultralight nodes run `pclientd` to scan, decrypt, and synchronize a specific wallet's data, as well as build and sign transactions, as described in [_Running a node: `pclientd`_](./node/pclientd.md).  

The web extension and `pcli` embed the view and custody functionality provided
by `pclientd`, so it is not necessary to run `pclientd` to use them. Instead,
`pclientd` is intended to act as a local RPC for programmatic tooling (e.g.,
trading bots) not written in Rust that cannot easily embed the code for working
with Penumbra's shielded cryptography.
