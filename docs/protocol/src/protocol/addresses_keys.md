# Addresses and Keys

The key hierarchy is a modification of the design in Zcash [Sapling].  The main
differences are that it is designed for use with BLS12-377 rather than
BLS12-381, that it uses Poseidon as a hash and PRF, `decaf377` as the embedded
group, and that it includes support for [fuzzy message
detection](../crypto/fmd.md).

All key material within a particular spend authority is ultimately derived from
a single root secret.  The internal key components and their derivations are
described in the following sections:

* [Spending Keys](./addresses_keys/spend_key.md) describes derivation of the
  spending key from the root key material;
* [Viewing Keys](./addresses_keys/viewing_keys.md) describes derivation of the full, incoming, and outgoing viewing keys;
* [Addresses and Detection Keys](./addresses_keys/addresses.md) describes derivation of multiple, publicly unlinkable addresses for a single spending authority, each with their own detection key.

The diagram in the [Overview](../concepts/addresses_keys.md) section describes
the key hierarchy from an external, functional point of view.  Here, we zoom in
to the internal key components, whose relations are depicted in the following
diagram.  Each internal key component is represented with a box; arrows depict
key derivation steps, and diamond boxes represent key derivation steps that
combine multiple components.

```mermaid
flowchart BT
    subgraph Address
        direction TB;
        d2[d];
        pk_d;
        ck_d;
    end;
    subgraph DTK[Detection Key]
        dtk_d;
    end;
    subgraph IVK[Incoming\nViewing Key]
        ivk;
        dk;
    end;
    subgraph OVK[Outgoing\nViewing Key]
        ovk;
    end;
    subgraph FVK[Full Viewing Key]
        ak;
        nk;
    end;
    subgraph SK[Spending Key]
        direction LR;
        ask;
        seed;
    end;

    seed --> ask;
    seed --> nk;

    ask --> ak;

    ak & nk --> fvk_blake{ };
    ak & nk --> fvk_poseidon{ };
    fvk_blake --> ovk & dk;
    fvk_poseidon --> ivk;

    index(Diversifier Index);
    d1[d];

    d1 --- d2;

    index --> aes_ff1{ };
    dk ---> aes_ff1{ };
    aes_ff1 --> d1;

    d1 & ivk --> scmul_ivk{ };
    scmul_ivk --> pk_d;

    d1 & ivk --> dtk_blake{ };
    dtk_blake --> dtk_d;
    dtk_d --> ck_d;
```


[Sapling]: https://zips.z.cash/protocol/protocol.pdf
