# Homomorphic Threshold Encryption 

The core of the flow encryption system requires a partially homomorphic
encryption scheme, which allows for users to publish transactions that contain
*encrypted* [values](../../concepts/asset_amounts.md). These encrypted values
are then *aggregated*, using the homomorphism, by validators. The aggregate
value (the "batched flow") is then decrypted using the threshold decryption
scheme described here.

## Desired Properties

For our threshold encryption scheme, we require two important properties:

* Homomorphism: we must be able to operate over ciphertexts, by combining value commitments from many participants into a batched value.
* Verifiability: we must be able to verify that a given value $v_i$ was encrypted correctly to a given ciphertext $c_i$
* Robustness: up to $n-t$ validators must be permitted to either fail to provide a decryption share or provide in invalid decryption share.

## Concrete Instantiation: Homomorphic ElGamal

### Setup

Compute a lookup table $LUT$ for every $v_i \in [0, 2^{23})$ by setting
$LUT[v_i] = v_i\mathbb{G}$ where $\mathbb{G}$ is the basepoint of `decaf377`.
Store $LUT$ for later use in value decryption.


### Value Encryption

```
                                  ┌───────────────────┐                           
                                  │DKG Public Key (D) │                           
                                  │                   │                           
                                  └───────────────────┘                           
                                            │                                     
                                  ┌─────────▼─────────┐    ┌──────────────┐       
                                  │                   │    │     v_e      │       
                                  │                   │    │  (encrypted  │       
                                  │                   │    │    value)    │       
                  ┌──────────┐    │ElGamal Encryption │    │ ┌──────────┐ │       
               ┌─▶│v_0 (u16) │────▶   D: DKG Pubkey   ├────┼▶│   c_0    │ │       
               │  └──────────┘    │     M = v_i*G     │    │ └──────────┘ │       
               │  ┌──────────┐    │     e <- F_q      │    │ ┌──────────┐ │       
┌────────────┐ ├─▶│v_1 (u16) │────▶ c_i = (e*G, M+eD) ├────┼▶│   c_1    │ │       
│            │ │  └──────────┘    │                   │    │ └──────────┘ │       
│v [0, 2^64) │─┤  ┌──────────┐    │                   │    │ ┌──────────┐ │       
│            │ ├─▶│v_2 (u16) │────▶ Correctness Proof ├────┼▶│   c_2    │ │       
└────────────┘ │  └──────────┘    │         σ         │    │ └──────────┘ │       
               │  ┌──────────┐    │                   │    │ ┌──────────┐ │       
               └─▶│v_3 (u16) │────▶                   ├────┼▶│   c_3    │ │       
                  └──────────┘    │                   │    │ └──────────┘ │       
                                  │                   │    │              │       
                                  │                   │    │              │       
                                  └───────────────────┘    └──────────────┘       
                                            │                                     
                                            │           ┌────────────────────────┐
                                            └──────────▶│proofs σ_ci = (α,γ,r,s) │
                                                        └────────────────────────┘
```


A *value* $v$ is given by an unsigned 64-bit integer $v \in [0, 2^{64})$. Split $v$ into four 16-bit limbs 

$$v_q = v_0 + v_1 2^{16} + v_2 2^{32} + v_3 2^{48}$$ with $v_i \in [0, 2^{16}]$.


Then, perform ElGamal encryption to form the ciphertext $v_e$ by taking (for each $v_i$)

$$M_i = v_i*G$$
$$e \overset{rand}{\leftarrow} \mathbb{F_q}$$
$$c_i = (e*G,  M_i + e*D)$$

Where $G$ is the basepoint generator for `decaf377`, $\mathbb{F_q}$ is
the scalar field, and $D$ is the public key output from [DKG](./dkg.md).

Next, compute a proof of correctness of the ElGamal encryption by executing the following protocol:

$$k_{1} \overset{rand}{\leftarrow} \mathbb{F_q} $$
$$k_{2} \overset{rand}{\leftarrow} \mathbb{F_q}$$
$$\alpha = k_{1}*G + k_{2}*D$$
$$\gamma = k_{2}*G$$
$$c = H(r, s)$$

$$r = k_{1} + v_i*c$$
$$s = k_{2} + e*c$$

The proof is then $\sigma_{c_i} = (r, s, \alpha, \gamma)$.
The encryption of value $v$ is given as $v_e = [c_1, c_2, c_3, c_4]$.

Upon receiving an encrypted value $v_e$ with proofs $\sigma_{c_i}$, a validator
or validating full node should verify each proof $\sigma_{c_i}$ by checking

$$c = H(r, s)$$
$$G*s \stackrel{?}{=} \gamma + c_{i0}*c$$
$$G*r+ D*s \stackrel{?}{=} \alpha + c_{i1}*c$$

Considering the value invalid if the proof fails to verify.

Each ciphertext $c_i$ is two group elements, accompanied by a proof
$\sigma_{c_i}$ which is two group elements and two scalars. `decaf377` group
elements and scalars are encoded as 32-byte values, thus every encrypted value
$v_e$ combined with its proof $\sigma_{ci}$ is $6*32*4$ = 768 bytes.

### Value Aggregation

```
                          n (batch size)                         
┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┐
│┌──────────────┐   ┌──────────────┐            ┌──────────────┐│
 │              │   │              │            │              │ 
 │   v_{e0}     │   │    v_{e1}    │            │    v_{ek}    │ 
 │              │   │              │            │              │ 
 │ ┌──────────┐ │   │ ┌──────────┐ │            │ ┌──────────┐ │ 
 │ │   c_0    │ │   │ │   c_0    │ │            │ │   c_0    │ │ 
 │ └──────────┘ │   │ └──────────┘ │            │ └──────────┘ │ 
 │ ┌──────────┐ │   │ ┌──────────┐ │            │ ┌──────────┐ │ 
 │ │   c_1    │ │   │ │   c_1    │ │            │ │   c_1    │ │ 
 │ └──────────┘ │   │ └──────────┘ │            │ └──────────┘ │ 
 │ ┌──────────┐ │   │ ┌──────────┐ │    ...     │ ┌──────────┐ │ 
 │ │   c_2    │ │   │ │   c_2    │ │            │ │   c_2    │ │ 
 │ └──────────┘ │   │ └──────────┘ │            │ └──────────┘ │ 
 │ ┌──────────┐ │   │ ┌──────────┐ │            │ ┌──────────┐ │ 
 │ │   c_3    │ │   │ │   c_3    │ │            │ │   c_3    │ │ 
 │ └──────────┘ │   │ └──────────┘ │            │ └──────────┘ │ 
 │              │   │              │            │              │ 
 │              │   │              │            │              │ 
 └──────────────┘   └──────────────┘            └──────────────┘ 
         │                  │             │             │        
         │                  │             │             │        
         └──────────────────┴─────┬───────┴─────────────┘        
                                  │             ┌──────────────┐ 
                                  ▼             │              │ 
                                ┌───┐           │   v_{agg}    │ 
                                │ + │───────────▶              │ 
                                └───┘           │ ┌──────────┐ │ 
                                                │ │   c_0    │ │ 
                                                │ └──────────┘ │ 
                                                │ ┌──────────┐ │ 
                                                │ │   c_1    │ │ 
                                                │ └──────────┘ │ 
                                                │ ┌──────────┐ │ 
                                                │ │   c_2    │ │ 
                                                │ └──────────┘ │ 
                                                │ ┌──────────┐ │ 
                                                │ │   c_3    │ │ 
                                                │ └──────────┘ │ 
                                                │              │ 
                                                │              │ 
                                                └──────────────┘  
```


To batch flows, we must use the homomorphic property of ElGamal ciphertexts.
Aggregation should be done component-wise, that is, on each limb of the
ciphertext ($c_i$). To aggregate a given $v_e, v_e'$, simply add each limb:

$$v_n = v_e + v_e' = [c_0+c_0', c_1+c_1', c_2+c_2', c_3+c_3'] = \cdots = v_q + v_q' = v + v'$$

This holds due to the homomorphic property of ElGamal cipertexts.

Aggregation can be publicly verified by any validator or validating full node,
by simply publicly adding together all ciphertexts and verifying that the same
result as $v_{agg}$ was achieved.


### Value Decryption
```
┌──────────────┐                                                                        
│     v_e      │                                                                        
│  (encrypted  │                                                                        
│    value)    │                                                                        
│ ┌──────────┐ │     ┌─────────────────────────┐                                        
│ │   c_0    │─┼────▶│                         │    ┌─────────────────────────────────┐ 
│ └──────────┘ │     │                         │    │       Gossip (ABCI++ Vote       │ 
│ ┌──────────┐ │     │                         │    │           Extensions)           │ 
│ │   c_1    │─┼────▶│Decryption Shares + Proof│    │                                 │ 
│ └──────────┘ │     │                         │    │        verify share proof       │ 
│ ┌──────────┐ │     │  s_{pi} = d_{p}c_{i0}   │───▶│          σ_pi for s_pi          │ 
│ │   c_2    │─┼────▶│   σ_{pi} = (r, α, γ)    │    │                                 │ 
│ └──────────┘ │     │                         │    │      d = sum(s_pi*λ_{0,i})      │ 
│ ┌──────────┐ │     │                         │    │        v_mi = -d + c_{i1}       │ 
│ │   c_3    │─┼────▶│                         │    │                                 │ 
│ └──────────┘ │     └─────────────────────────┘    └─────────────────────────────────┘ 
│              │                                                     │    ┌───────┐     
│              │                                                    ┌┘    │  LUT  │     
└──────────────┘                                                    │     └───┬───┘     
                   ┌─────────────────────────┐       ┌──────────────▼─────────▼────────┐
                   │                         │       │       Reverse dLog Lookup       │
                   │ Reconstruct from Limbs  │       │                                 │
                   │                         │◀──────│v_q = [LUT[v_mi], ..., LUT[v_mn]]│
                   │                         │       │                                 │
                   └─────────────────────────┘       └─────────────────────────────────┘
                                │                                                       
                                ▼                                                       
                           ┌─────────┐                                                  
                           │         │                                                  
                           │v (u128) │                                                  
                           │         │                                                  
                           └─────────┘                                                   
```

To decrypt each $v_e$, take each ciphertext $c_i$ and perform threshold ElGamal
decryption using the participant's DKG private key share $d_p$ to produce
decryption share $s_pi$:

$$s_{pi} = d_{p}c_{i0}$$

Next, each participant must compute a proof that their decryption share is well
formed relative to the commitment to their secret share $\phi_{p} = G \cdot d_p$.
This is accomplished by adopting the Chaum-Pedersen protocol for proving
DH-triples.

With $c_{i0}$, $s_{pi}$, and $d_p$ as inputs, each participant computes their proof $\sigma_{pi}$ by taking 

$$k \overset{rand}{\leftarrow} \mathbb{F_q}$$
$$\alpha = k * G$$
$$\gamma = k * c_{i0}$$
$$e = H(i, p, \alpha, \gamma)$$
$$r = k + d_p * e$$

The proof is the tuple $\sigma_{pi} = (r, \alpha, \gamma)$.

Every participant then broadcasts their proof of knowledge $\sigma_{pi}$ along
with their decryption share $s_{pi}$ to every other participant.

After receiving $s_{pi}, \sigma_{pi} = (r, \alpha, \gamma)$ from each participant, each
participant verifies that $s_{pi}$ is valid by checking

$$e = H(i, p, \alpha, \gamma)$$
$$G * r \stackrel{?}{=} \alpha + \phi_{p} * e$$
$$c_{i0} * r \stackrel{?}{=} \gamma + s_{pi} * e$$

and aborting if verification fails. (TODO: should we ignore this participant's share, or report/slash them?)

This protocol is the Chaum-Pedersen sigma protocol which here proves the relation $$\phi_{p} = G * d_p \wedge s_{pi} = c_{i0} * d_p$$

Now each participant can sum their received and validated decryption shares by taking 

$$d = \sum_{p=0}^{n} s_{pi} \lambda_{0,i}$$

where $\lambda_{i}$ is the lagrange coefficient (for x=0) at $i$, defined by 

$$\lambda_{0,i} = \prod_{n \in S, n \neq i} \frac{n}{n - i}$$

where $S$ is the set of all participant indicies.

Then, compute the resulting decryption by taking

$$v_m = -d + c_{i1}$$

Now we have the output $v_m = [v_{im}...]$. Each $v_{im}$ is a `decaf377` group
element. Use our lookup table $LUT$ from the setup phase to transform each
value to its discrete log relative to the basepoint: $$v_i = LUT[v_{im}]$$ Now
we have the decrypted value $$v_q = [v_0, v_1, v_2, v_3]$$ 

where each $v_i$ is bounded in $[0, 2^{23})$.

To recombine the value, iterate over each $v_i$, packing each $v_i$ into a `u16` value $v_{ui}$, performing carries if necessary. This yields the final value

$$v = v_{ui} + v_{ui} * 2^{16} + v_{ui} * 2^{32} + v_{ui} * 2^{48} + v_{ui} * 2^{64}$$

This value is bounded by $[0, 2^71]$, assuming that the coefficients in the previous step were correctly bounded.


## Note

On verifiability, this scheme must include some snark proof that coefficients
were correctly created from input values. This can be accomplished by providing
a SNARK proof $\pi$ that accompanies each value. It may also be desirable to
SNARK the sigma protocol given in the value encryption phase in order to save
on chain space.



***TODO***: end-to-end complexity analysis (number of scalar mults per block, LUT size, etc)
