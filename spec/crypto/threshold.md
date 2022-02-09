# Threshold Crypto for Flow Encryption

In Penumbra, we need some scheme to achieve privacy in a multi-party context,
where knowledge from multiple participants is required. Traditional
zero-knowledge proofs are not sufficient for this, since there is a global
state that no individual participant can prove knowledge of if the global state
is private. This is described in the [Batching
Flows](../concepts/batching_flows.md) section at length. To implement flow
encryption, we need a few constructions:

* [Distributed Key Generation](./flow-encryption/dkg.md): a distributed key generation scheme that allows a quorum of validators to derive a shared `decaf377` public key as well as a set of private key shares.

* [Homomorphic Threshold Encryption](./flow-encryption/threshold-encryption.md): a partially homomorphic encryption scheme which allows users to encrypt values and validators to aggregate (using the homomorphism) and decrypt aggregates (using the private key shares from dkg).

