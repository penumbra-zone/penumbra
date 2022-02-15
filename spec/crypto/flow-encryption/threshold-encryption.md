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
                  ┌──────────┐    │                   │    │ ┌──────────┐ │
               ┌─▶│v_0 (u16) │────▶                   ├────┼▶│   c_0    │ │
               │  └──────────┘    │                   │    │ └──────────┘ │
               │  ┌──────────┐    │ElGamal Encryption │    │ ┌──────────┐ │
┌────────────┐ ├─▶│v_1 (u16) │────▶   D: DKG Pubkey   ├────┼▶│   c_1    │ │
│            │ │  └──────────┘    │     M = v_i*G     │    │ └──────────┘ │
│v [0, 2^64) │─┤  ┌──────────┐    │     e <- F_q      │    │ ┌──────────┐ │
│            │ ├─▶│v_2 (u16) │────▶ c_i = (e*G, M+eD) ├────┼▶│   c_2    │ │
└────────────┘ │  └──────────┘    │                   │    │ └──────────┘ │
               │  ┌──────────┐    │                   │    │ ┌──────────┐ │
               └─▶│v_3 (u16) │────▶                   ├────┼▶│   c_3    │ │
                  └──────────┘    │                   │    │ └──────────┘ │
                                  │                   │    │              │
                                  │                   │    │              │
                                  └───────────────────┘    └──────────────┘
```


A sketch of one construction is as follows.

A *value* $v$ is given by an unsigned 64-bit integer $v \in [0, 2^{64})$. Split $v$ into four 16-bit limbs 

$$v_q = v_0 + v_1 2^{16} + v_2 2^{32} + v_3 2^{48}$$ with $v_i \in [0, 2^{16}]$.


Then, perform ElGamal encryption to form the ciphertext $v_e$ by taking (for each $v_i$)

$M_i = v_i*G$

$e \overset{{\scriptscriptstyle\\$}}{\leftarrow} \mathbb{F_q}$

$c_i = (e\mathbb{G},  M_i + eD)$


Where $\mathbb{G}$ is the basepoint generator for `decaf377`, $\mathbb{F_q}$ is
the scalar field, and $D$ is the public key output from [DKG](./dkg.md).

The encryption of value $v$ is given as $v_e = [c_1, c_2, c_3, c_4]$.

Each ciphertext $c_i$ is two group elements. `decaf377` group elements are
encoded as 32-byte values, thus every encrypted value $v_e$ is 256 bytes.

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


### Value Decryption
```
┌──────────────┐                                                                       
│     v_e      │                                                                       
│  (encrypted  │                                                                       
│    value)    │                                                                       
│ ┌──────────┐ │     ┌─────────────────────────┐                                       
│ │   c_0    │─┼────▶│                         │                                       
│ └──────────┘ │     │                         │     ┌───────────────────────────┐     
│ ┌──────────┐ │     │                         │     │                           │     
│ │   c_1    │─┼────▶│Create Decryption Shares │     │    Gossip (ABCI++ Vote    │     
│ └──────────┘ │     │                         │     │        Extensions)        │     
│ ┌──────────┐ │     │  s_{pi} = -d_{p}c_{i0}  │────▶│                           │     
│ │   c_2    │─┼────▶│                         │     │  v_m = sum(s_pi + c_i1)   │     
│ └──────────┘ │     │                         │     │                           │     
│ ┌──────────┐ │     │                         │     └───────────────────────────┘     
│ │   c_3    │─┼────▶│                         │                   │                   
│ └──────────┘ │     └─────────────────────────┘                   │     ┌───────┐     
│              │                                                   │     │  LUT  │     
│              │                                                   │     └───┬───┘     
└──────────────┘    ┌─────────────────────────┐     ┌──────────────▼─────────▼────────┐
                    │                         │     │       Reverse dLog Lookup       │
                    │ Reconstruct from Limbs  │     │                                 │
                    │                         │◀────│v_q = [LUT[v_mi], ..., LUT[v_mn]]│
                    │                         │     │                                 │
                    └─────────────────────────┘     └─────────────────────────────────┘
                                 │                                                     
                                 │                                                     
                            ┌────▼────┐                                                
                            │         │                                                
                            │v (u128) │                                                
                            │         │                                                
                            └─────────┘                                                
```

To decrypt each $v_e$, take each ciphertext $c_i$ and perform threshold ElGamal
decryption using the participant's DKG private key share $d_p$ to produce
decryption share $s_pi$:

$$s_{pi} = -d_{p}c_{i0}$$

Then broadcast each decryption share $s_{pi}$ to every participant. 

Upon receiving $s_{pi}$ from every participant, perform threshold decryption by taking

$$v_m = \sum_{p=0}^{n} s_{pi} + c_{i1}$$

***TODO/NOTE***: This needs a notion of *verifiability*: a single participant can influence the decryption of $v_m$ by biasing their broadcast $s_{pi}$. Can we address this with a commitment, or is a SNARK required? 

***TODO/NOTE***: As described, this is a n/n scheme. How do we transform it to t/n for an arbitrary `t`? (lagrange interpolation)

Now we have the output $v_m = [v_{im}...]$. Each $v_{im}$ is a `decaf377` group
element. Use our lookup table $LUT$ from the setup phase to transform each
value to its discrete log relative to the basepoint: $$v_i = LUT[v_{im}]$$ Now
we have the decrypted value $$v_q = [v_0, v_1, v_2, v_3]$$ 

where each $v_i$ is bounded in $[0, 2^{23})$.

To recombine the value, iterate over each $v_i$, packing each $v_i$ into a `u16` value $v_{ui}$, performing carries if necessary. This yields the final value

$$v = v_{ui} + v_{ui} * 2^{16} + v_{ui} * 2^{32} + v_{ui} * 2^{48} + v_{ui} * 2^{64}$$

This value is bounded by $[0, 2^71]$, assuming that the coefficients in the previous step were correctly bounded.
