# Concentrated Liquidity

Penumbra uses a hybrid, order-book-like AMM with automatic routing. Liquidity on Penumbra is recorded as many individual concentrated liquidity positions, akin to an order book. Each liquidity position is its own AMM, with its own fee tier, and that AMM has the simplest possible form, a constant-sum (fixed-price) market maker. These component AMMs are synthesized into a global AMM by the DEX engine, which optimally routes trades across the entire liquidity graph. Because each component AMM is of the simplest possible form, this optimization problem is easy to solve: it’s a graph traversal.


### Liquidity positions

At a high-level, a liquidity position on Penumbra sets aside reserves for one or both assets in a trading pair and specifies a fixed exchange rate between them. The reserves are denoted by $R_1$ and $R_2$, and the valuations of the assets are denoted by $p_1$ and $p_2$, respectively. The constant-sum trading function $\varphi(R)$ combines the reserves and prices of the two assets according to the formula $\varphi(R) = p_1 \cdot R_1 + p_2 \cdot $_2$. The spread between the bid/ask offered by the position is controlled by a fee $\gamma$.

In practice, a **liquidity position** consists of:

* A trading pair $(\mathsf a_1, \mathsf a_2)$ recording the asset IDs of the assets in the pair. The asset IDs are $F_q$ elements, and the pair is made order-independent by requiring that $\mathsf a_1 < \mathsf a_2$.
* A trading function $\varphi$, specified by $p_1, p_2, \gamma$.
* A random, globally-unique 32-byte nonce $n$.

This data is hashed to form the **position ID**, which uniquely identifies the position.  The position nonce ensures that it is not possible to create two positions with colliding position IDs.

The reserves are pointed to by the position ID and recorded separately, as they change over time as trades are executed against the position. One way to think of this is to think of the position ID as an ephemeral account content-addressed by the trading function whose assets are the reserves and which is controlled by bearer NFTs recorded in the shielded pool.

Positions have four **position states**, and can only progress through them in sequence:

* an **opened** position has reserves and can be traded against;
* a **closed** position has been deactivated and cannot be traded against, but still has reserves;
* a **withdrawn** position has had reserves withdrawn;
* a **claimed** position has had any applicable liquidity incentives claimed.

Control over a position is tracked by a **liquidity position NFT** (**LPNFT**) that records both the position ID and the position state.  Having the LPNFT record both the position state and ID means that the transaction value balance mechanism can be used to enforce state transitions:

- the `PositionOpen` action debits the initial reserves and credits an opened position NFT;
- the `PositionClose` action debits an opened position NFT and credits a closed position NFT;
- the `PositionWithdraw` action debits a closed position NFT and credits a withdrawn position NFT and the final reserves;
- the `PositionRewardClaim` action debits a withdrawn position NFT and credits a claimed position NFT and any liquidity incentives. 

Separating _closed_ and _withdrawn_ states is necessary because phased execution means that the exact state of the final reserves may not be known until the closure is processed position is removed from the active set. 

However, having to wait for the next block to withdraw funds does not necessarily cause a gap in available capital: a marketmaker wishing to update prices block-by-block can stack the `PositionWithdraw` for the last block's position with a `PositionOpen` for their new prices and a `PositionClose` that expires the new position at the end of the next block.

Separating _withdrawn_ and _claimed_ states allows retroactive liquidity incentives (e.g., $X$ rewards over some time window, allocated pro rata to liquidity provided, etc).  As yet there are no concrete plans for liquidity incentives, but it seems desirable to build a hook for them, and to allow them to be funded permissionlessly (e.g., so some entity can decide to subsidize liquidity on X pair of their own accord).

The set of all liquidity positions between two assets forms a market, which indicates the availability of inventory at different price levels, just like an order book.

```
                               ┌────────────────────┐
                               │                    │
                               │      Asset B       │
                               │                    │
                               └─────┬─┬─┬──┬───────┘
                                     │ │ │  │
                                     │ │ │  │
                                     │ │ │  │
┌───────────┐                        │ │ │  │
│           │sell 50A@100, buy 0B@40 │ │ │  │ phi(50,100) - 50*100 + 0*40 = 5000
│           ├────────────────────────┘ │ │  │
│  Asset A  │sell 0A@101, buy 25B@43   │ │  │ psi(0,25) = 0*101 + 25*43 = 1075
│           ├──────────────────────────┘ │  │
│           │sell 99A@103, buy 46@38     │  │ chi(99,46) = 99*103 + 46*38 = 11945
│           ├────────────────────────────┘  │
│           │sell 50A@105, buy 1@36         │ om(50,1) = 50*105 + 1*36 = 5286
│           ├───────────────────────────────┘
└───────────┘
```


```
┌────────────────────────────────┐
│ x                              │
│ xx                             │
│ xxx                          x │
│ xxx                        xxx │
│ xxxxx                     xxxx │  sell 50A@100, buy 0B@40
│ xxxxxx                   xxxxx │  ────────────────────────
│ xxxxxxx                 xxxxxx │  sell 0A@101, buy 25B@43
│ xxxxxxxx               xxxxxxx │  ────────────────────────
│ xxxxxxxx              xxxxxxxx │  sell 99A@103, buy 46@38
│ xxxxxxxxx            xxxxxxxxx │  ────────────────────────
│ xxxxxxxxxxx         xxxxxxxxxx │  sell 50A@105, buy 1@36
│ xxxxxxxxxxx         xxxxxxxxxx │  ────────────────────────
│ xxxxxxxxxxx         xxxxxxxxxx │
│ xxxxxxxxxxx        xxxxxxxxxxx │
│ xxxxxxxxxxx      xxxxxxxxxxxxx │
└────────────────────────────────┘
```
### Liquidity composition

During execution, assets and liquidity positions create a graph that can be traversed. To create and execute routes, it is helpful to understand the available liquidity between two assets, even in the absence of direct liquidity positions between them.

As a result, it is desirable to develop a method for combining two liquidity positions that cover separate but intersecting assets into a unified synthetic position that represents a section of the overall liquidity route. For example, combining positions for the pair `A <> B` and `B <> C` into a synthetic position for `A <> C`.

Given two AMMs, $\varphi(R_1, R_2) = p_1 R_1 + p_2 R_2$ with fee $\gamma$ trading between assets $1$ and $2$ and $\psi(S_2, S_3) = q_2 S_2 + q_3 S_3$ with fee $\delta$ trading between assets $2$ and $3$, we can compose $\varphi$ and $\psi$ to obtain a synthetic position $\chi$ trading between assets $1$ and $3$ that first trades along $\varphi$ and then $\psi$ (or along $\psi$ and then $\varphi$).

We want to write the trading function of this AMM as $\chi(T_1, T_3) = r_1 T_1 + r_3 T_3$ with fee $\varepsilon$, prices $r_1, r_2$, and reserves $T_1, T_2$.

First, write the trade inputs and outputs for each AMM as $\Delta^\chi = (\Delta^\chi_1, \Delta^\chi_3)$, $\Lambda^\chi = (\Lambda^\chi_1, \Lambda^\chi_3)$, $\Delta^\varphi = (\Delta^\varphi_1, \Delta^\varphi_2)$, $\Lambda^\varphi = (\Lambda^\varphi_1, \Lambda^\varphi_2)$, $\Delta^\psi = (\Delta^\psi_2, \Delta^\psi_3)$, $\Lambda^\psi = (\Lambda^\psi_2, \Lambda^\psi_3)$, where the subscripts index the asset type and the superscripts index the AMM. We want $\Delta^\chi = \Delta^\varphi + \Delta^\psi$ and $\Lambda^\chi = \Lambda^\varphi + \Lambda^\psi$, meaning that:

$$
(\Delta_1^\chi, \Delta_3^\chi) = (\Delta_1^\varphi, \Delta_3^\psi), \\
(\Lambda_1^\chi, \Lambda_3^\chi) = (\Lambda_1^\varphi, \Lambda_3^\psi), \\
(\Delta_2^\varphi, \Delta_2^\psi) = (\Lambda_2^\psi, \Lambda_2^\varphi).
$$

Visually this gives (using $\Theta$ as a placeholder for $\psi$):

```
┌─────┬───────────────────────────────────────────────────────────┬─────┐
│     │                            ┌───────────────────────────┐  │     │
│     │                            │                           │  │     │
│     ├──────┐    ┌────┬───┬────┐  │   ┌────┬───┬────┐         │  │     │
│ Δᵡ₁ │      │    │    │   │    │  │   │    │   │    │         │  │ Λᵡ₁ │
│     │      └───►│ Δᵠ₁│   │ Λᵠ₁├──┘ ┌►│ Δᶿ₂│   │ Λᶿ₂├───► 0   └─►│     │
│     │           │    │   │    │    │ │    │   │    │            │     │
├─────┤           ├────┤ Φ ├────┤    │ ├────┤ Θ ├────┤            ├─────┤
│     │           │    │   │    ├────┘ │    │   │    │            │     │
│     │      0 ──►│ Δᵠ₂│   │ Λᵠ₂│   ┌─►│ Δᶿ₃│   │ Λᶿ₃├───────────►│ Λᵡ₃ │
│ Δᵡ₃ │───┐       │    │   │    │   │  │    │   │    │            │     │
│     │   │       └────┴───┴────┘   │  └────┴───┴────┘            │     │
│     │   └─────────────────────────┘                             │     │
└─────┴───────────────────────────────────────────────────────────┴─────┘
```

The reserves $T_1$ are precisely the maximum possible output $\Lambda_1^\chi$. On the one hand, we have $\Lambda_1^\chi = \Lambda_1^\varphi \leq R_1$, since we cannot obtain more output from $\varphi$ than its available reserves. On the other hand, we also have
$$
\Lambda_1^\chi = \Lambda_1^\varphi = \frac {p_2} {p_1} \gamma \Delta_2^\varphi = \frac {p_2} {p_1} \gamma \Lambda_2^\psi \leq \frac {p_2} {p_1} \gamma S_2,
$$

since we cannot input more into $\varphi$ than we can obtain as output from $\psi$. This means we have
$$
T_1 = \max \left\{ R_1, \frac {p_2} {p_1} \gamma S_2 \right\} \\
T_3 = \max \left\{ S_3, \frac {q_2} {q_3} \delta R_2 \right\},
$$

using similar reasoning for $T_3$ as for $T_1$.

On input $\Delta^\chi_1$, the output $\Lambda^\chi_3$ is
$$
\Lambda^\chi_3 = \Lambda^\psi_3
= \frac {q_2} {q_3} \delta \Delta^\psi_2
= \frac {q_2} {q_3} \delta \Lambda^\varphi_2
= \frac {q_2 p_1} {q_3 p_2} \delta \gamma \Delta^\varphi_1
= \frac {q_2 p_1} {q_3 p_2} \delta \gamma \Delta^\chi_1,
$$

and similarly on input $\Delta^\chi_3$, the output $\Lambda^\chi_1$ is
$$
\Lambda^\chi_1 = \Lambda^\varphi_1
= \frac {p_2} {p_1} \gamma \Delta^\varphi_2
= \frac {p_2} {p_1} \gamma \Lambda^\psi_2
= \frac {p_2 q_3} {p_1 q_2} \gamma \delta \Delta^\psi_1
= \frac {p_2 q_3} {p_1 q_2} \gamma \delta \Delta^\chi_1,
$$

so we can write the trading function $\chi$ of the composition as
$$
\chi(T_1, T_3) = r_1 T_1 + r_3T_3
$$

with $r_1 = p_1 q_2$, $r_3 = p_2 q_3$, fee $\varepsilon = \gamma \delta$, and reserves $T_1$, $T_3$.


### Binning

## Tickspace 

## 


### Replicating payoffs

1. why, talk about active management vs. passive LPing
1.5 positive externalities of liveness for liquidity 
2. provide an example of a polynomial trading function getting interpolated with CSMMs 
3. payoffs vs. focus on trading functions?