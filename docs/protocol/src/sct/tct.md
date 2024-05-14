# Tiered Commitment Tree

The Penumbra protocol's [state commitment tree (SCT)](../sct.md) stores
cryptographic commitments to shielded state, such as note and swap commitments.
The SCT is instantiated using the _tiered commitment tree_ (TCT) data structure,
an append-only, ZK-friendly Merkle tree. The unique features of the TCT are
that it is:

* *Quaternary:* Each node of the tiered commitment tree has four children. This means
that the tree can be much shallower while having the same amount of capacity for
leaves, which reduces the number of hash operations required for each insertion.
The academic research surrounding the Poseidon hash recommends a quaternary tree
specifically for its balance between proof depth and proof size: a binary tree
means too many expensive hash operations, whereas an octal tree means too large
proofs.

* *Sparse:* A client’s version of the tree need only represent the state
commitments pertinent to that client’s state, with all other “forgotten”
commitments and internal hashes related to them pruned to a single summary hash.

* *Semi-Lazy:* The internal hashes along the frontier of the tree are only
computed on demand when providing a proof of inclusion or computing the root
hash of the tree, which means a theoretical performance boost of (amortized)
12x during scanning.

* *Tiered:* the structure of the tree is actually a triply-nested
tree-of-trees-of-trees. The global, top-level tree contains at its leaves up to
65,536 epoch trees, each of which contains at its own leaves up to 65,536 block
trees, each of which contains at its own leaves up to 65,536 note commitments.
It is this structure that enables a client to avoid ever performing the hashes
to insert the state commitments in blocks or epochs when it knows it didn’t
receive any notes or swaps. When a client detects that an entire block, or an
entire epoch, contained nothing of interest for it, it doesn’t need to construct
that span of the commitment tree: it merely inserts the singular summary hash
for that block or epoch, which is provided by the chain itself inline with the
stream of blocks.

```
Eternity┃           ╱╲ ◀───────────── Anchor
    Tree┃          ╱││╲               = Global Tree Root
        ┃         * ** *           ╮
        ┃      *   *  *   *        │ 8 levels
        ┃   *     *    *     *     ╯
        ┃  ╱╲    ╱╲    ╱╲    ╱╲
        ┃ ╱││╲  ╱││╲  ╱││╲  ╱││╲ ◀─── Global Tree Leaf
                        ▲             = Epoch Root
                     ┌──┘
                     │
                     │
   Epoch┃           ╱╲ ◀───────────── Epoch Root
    Tree┃          ╱││╲
        ┃         * ** *           ╮
        ┃      *   *  *   *        │ 8 levels
        ┃   *     *    *     *     ╯
        ┃  ╱╲    ╱╲    ╱╲    ╱╲
        ┃ ╱││╲  ╱││╲  ╱││╲  ╱││╲ ◀─── Epoch Leaf
                 ▲                    = Block Root
                 └───┐
                     │
                     │
   Block┃           ╱╲ ◀───────────── Block Root
    Tree┃          ╱││╲
        ┃         * ** *           ╮
        ┃      *   *  *   *        │ 8 levels
        ┃   *     *    *     *     ╯
        ┃  ╱╲    ╱╲    ╱╲    ╱╲
        ┃ ╱││╲  ╱││╲  ╱││╲  ╱││╲ ◀─── Block Leaf
                                      = State Commitment
```
