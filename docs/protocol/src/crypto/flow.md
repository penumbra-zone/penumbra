# Flow Encryption

**NOTE: Flow Encryption will not ship in Penumbra V1, because ABCI 2.0 was delayed. Instead, flow contributions will be in the clear until a future network upgrade.**

**These notes are a work-in-progress and do not reflect the current protocol.**

In Penumbra, we need some scheme to achieve privacy in a multi-party context,
where knowledge from multiple participants is required. Traditional
zero-knowledge proofs are not sufficient for this, since there is a global
state that no individual participant can prove knowledge of if the global state
is private. This is described in the [Batching
Flows](../concepts/batching_flows.md) section at length. To implement flow
encryption, we need a few constructions:

* [Ideal Functionality](./flow/ideal.md)
* [The Eddy Construction](./flow/eddy.md)
* [Distributed Key Generation](./flow-encryption/dkg.md): a distributed key generation scheme that allows a quorum of validators to derive a shared `decaf377` public key as well as a set of private key shares.
* [Homomorphic Threshold Encryption](./flow-encryption/threshold-encryption.md): a partially homomorphic encryption scheme which allows users to encrypt values and validators to aggregate (using the homomorphism) and perform threshold decryption on aggregate values using the private key shares from DKG.


