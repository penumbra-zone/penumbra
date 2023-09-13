# Local RPC with `pclientd`

Penumbra's architecture separates public shared state from private per-user
state.  Each user's state is known only to them and other parties they disclose
it to.  While this provides many advantages -- and enables the core features of
the chain -- it also creates new operational challenges.  Most existing
blockchain tooling is built on the assumption that all chain state is available
from a fullnode via RPC, allowing the tooling to be relatively stateless,
obtaining its information from an RPC.

The role of `pclientd`, the Penumbra client daemon, is to restore this paradigm,
allowing third-party tooling to query both public and private state via RPC, and
to handle all of the Penumbra-specific cryptography.  It does this by:

* scanning and synchronizing a local, decrypted copy of all of a specific user's private data;
* exposing that data through a "view service" RPC that can query state and plan and build transactions;
* proxying requests for public chain state to its fullnode;
* optionally authorizing and signing transactions if configured with a spending key.

Client software can be written in any language with GRPC support, using
`pclientd` as a single endpoint for all requests.

```
   ┌────────┐  ┌─────────────────┐  ┌────────┐
   │  Client│  │         pclientd│  │Penumbra│
   │Software│◀─┼─┐             ┌─┼─▶│Fullnode│
   └────────┘  │ │             │ │  └────────┘
            ╭  │ │ ┌───────┐   │ │
     public │  │ │ │grpc   │   │ │
 chain data │  │ ├▶│proxy  │◀──┤ │
            ╰  │ │ └───────┘   │ │
               │ │             │ │
            ╭  │ │ ┌───────┐   │ │
    private │  │ │ │view   │   │ │
  user data │  │ ├▶│service│◀──┘ │
            ╰  │ │ └───────┘     │
               │ │               │
            ╭  │ │ ┌ ─ ─ ─ ┐     │
   spending │  │ │  custody      │
 capability │  │ └▶│service│     │
 (optional) ╰  │    ─ ─ ─ ─      │
               └─────────────────┘
```

# WARNING

Currently, `pclientd` does not support any kind of transport security or
authentication mechanism. Do not expose its RPC to untrusted access.  We intend
to remedy this gap in the future.



