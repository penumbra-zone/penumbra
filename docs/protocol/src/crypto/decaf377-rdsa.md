# Randomizable Signatures

Penumbra's signatures are provided by `decaf377-rdsa`, a variant of the Zcash
RedDSA construction instantiated using the [`decaf377` group](./decaf377.md).
These are Schnorr signatures, with two additional properties relevant to
Penumbra:

1. They support randomization of signing and verification keys.  Spending a note
requires use of the signing key that controls its spend authorization, but if
the same spend verification key were included in multiple transactions, they
would be linkable.  Instead, both the signing and verification keys are kept
secret[^1], and each spend description includes a randomization of the
verification key, together with a proof that the randomized verification key was derived from the correct spend verification key.

2. They support addition and subtraction of signing and verification keys.  This
property is used for binding signatures, which bind zero-knowledge proofs to the
transaction they were intended for and enforce conservation of value.

## `decaf377-rdsa`

Let $\mathbb G$ be the `decaf377` group of prime order $r$.  Keys and signatures
are parameterized by a domain $D$.  Each domain has an associated generator
$B_D$.  Currently, there are two defined domains: $D = \mathsf{SpendAuth}$ and
$D = \mathsf{Binding}$.  The hash function $H^\star : \{0,1\}^* \rightarrow
\mathbb F_r$ is instantiated by using `blake2b` with the personalization string
`decaf377-rdsa---`, treating the 64-byte output as the little-endian encoding of
an integer, and reducing that integer modulo $r$.

A *signing key* is a scalar $a \in \mathbb F_r$.  The corresponding *verification
key* is the group element $A = [a]B_D$.

###### `Sign`

On input message `m` with signing key $a$, verification key $A$, and domain $D$:

1. Generate 80 random bytes.
2. Compute the nonce as $r \gets H^\star(\mathtt{random\_bytes} ||
\mathtt{A\_bytes} || \mathtt{m})$.
3. Commit to the nonce as $R \gets [r]B_D$.
4. Compute the challenge as $c \gets H^\star(\mathtt{R\_bytes} ||
\mathtt{A\_bytes} || \mathtt m)$.
5. Compute the response as $s \gets r + ac$.
6. Output the signature `R_bytes || s_bytes`.

###### `Verify`

On input message `m`, verification key `A_bytes`, and signature `sig_bytes`:

1. Parse $A$ from `A_bytes`, or fail if `A_bytes` is not a valid (hence
canonical) `decaf377` encoding.
2. Parse `sig_bytes` as `R_bytes || s_bytes` and the components as $R$ and $s$,
or fail if they are not valid (hence canonical) encodings.
3. Recompute the challenge as $c \gets H^\star(\mathtt{R\_bytes} ||
\mathtt{A\_bytes} || m)$.
4. Check the verification equation $R = [s]B - [c]A$, rejecting the signature if
it is not satisfied.

## `SpendAuth` signatures

The first signature domain used in Penumbra is for spend authorization signatures.  The basepoint $B_{\mathsf{SpendAuth}}$ is the conventional `decaf377` basepoint `0x0800000...`.

Spend authorization signatures support randomization:

###### `Randomize.SigningKey`

Given a randomizer $r \xleftarrow{\\$} \mathbb F_r$, the randomized signing key is $ra$.

###### `Randomize.VerificationKey`

Given a randomizer $r \xleftarrow{\\$} \mathbb F_r$, the randomized verification key is $[r]A$.

Randomizing a signing key and then deriving the verification key associated to the randomized signing key gives the same result as randomizing the original verification key (with the same randomizer).

## `Binding` signatures

The second signature domain used in Penumbra is for binding signatures.  The
basepoint $B_{\mathsf{Binding}}$ is the result of converting
`blake2b(b"decaf377-rdsa-binding")` to an $\mathbb F_r$ element and applying
`decaf377`'s CDH encode-to-curve method.

Since the verification key corresponding to the signing key $a \in \mathbb F_r$ is $A = [a]B_D$, adding and subtracting signing and verification keys commutes with derivation of the verification key, as desired.

[^1]: This situation is a good example of why it's better to avoid the terms
"public key" and "private key", and prefer more precise terminology that names
keys according to the cryptographic capability they represent, rather than an
attribute of how they're commonly used. In this example, the verification key
should not be public, since it could link different transactions.
