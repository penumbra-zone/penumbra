# Addresses and Keys

The key hierarchy is a modification of the design in Zcash [Sapling].  The main
differences are that it is designed for use with BLS12-377 rather than
BLS12-381, that it also uses Poseidon[^1] as a hash and PRF, `decaf377` as the embedded
group, and that it includes support for [fuzzy message
detection](./crypto/fmd.md).

All key material within a particular spend authority - an *account* - is ultimately derived from
a single root secret.  The internal key components and their derivations are
described in the following sections:

* [Spending Keys](./addresses_keys/spend_key.md) describes derivation of the
  spending key from the root key material;
* [Viewing Keys](./addresses_keys/viewing_keys.md) describes derivation of the full, incoming, and outgoing viewing keys;
* [Addresses and Detection Keys](./addresses_keys/addresses.md) describes derivation of multiple, publicly unlinkable addresses for a single account, each with their own detection key.

The diagram in the [Overview](./concepts/addresses_keys.md) section describes
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
        spend_key_bytes;
    end;

    BIP44(BIP44 Seed Phrase) --> spend_key_bytes;
    BIP39(Legacy Raw BIP39) --> spend_key_bytes;

    spend_key_bytes --> ask;
    spend_key_bytes --> nk;

    ask --> ak;

    ak & nk --> fvk_blake{ };
    ak & nk --> fvk_poseidon{ };
    fvk_blake --> ovk & dk;
    fvk_poseidon --> ivk;

    index(address index);
    d1[d];

    d1 --- d2;

    index --> aes{ };
    dk ---> aes{ };
    aes --> d1;

    d1 & ivk --> scmul_ivk{ };
    scmul_ivk --> pk_d;

    d1 & ivk --> dtk_blake{ };
    dtk_blake --> dtk_d;
    dtk_d --> ck_d;
```


[Sapling]: https://zips.z.cash/protocol/protocol.pdf
[^1]: In general, the Poseidon hash function is used in places where hashing
needs to be done in-circuit, for example, when hashing new commitments into the
state commitment tree. Elsewhere in the protocol, we use BLAKE2b-512 e.g.
in the derivation of the spend authorization key, which is not done in-circuit.
