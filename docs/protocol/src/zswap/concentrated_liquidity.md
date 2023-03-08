# Concentrated Liquidity

ZSwap offers similar market functions to a central limit order book (CLOB), a `Swap` action is equivalent to a market order that takes liquidity, and opening a liquidity position is similar to posting limit orders that quote one or both sides of the pair.


### Liquidity positions

At a high level, a liquidity position sets aside reserves for one or both assets and specifies a fixed exchange rate. Algebraically, a liquidity position is defined by a constant-sum trading function $\phi(R) = p_1 \cdot R_1 + p_2 \cdot R_2$, which specifies reserves and prices of the two assets, denoted by $(R_1, p_1)$ and $(R_2, p_2)$, respectively.


In Penumbra, a **liquidity position** consists of:

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

Assets along with liquidity positions form a graph that can be traversed at execution. To create and execute routes, it is useful to have a notion of the available liquidity between two assets even if there are no direct liquidity positions between them.

Composing liquidity positions means combining two constant-sum trading functions that correspond to distinct but overlapping pairs (e.g. `A <> B <> C => A <> C`).

// reword this when writing `Path` doc
Aggregating liquidity positions means tallying two constant-sum trading functions that correspond to a same pair (e.g. `A <> B + A = B => A <> B`).
```
                       ┌───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
                       │                                                                                                                                                       │
   Da                  │                                                                                                                                                       │
                       │                                                                                                                                                       │
 ──────────────────────┼─►         ┌──────────────────────────────────────────────────────┐                 ┌──────────────────────────────────────────────────────┐       ────┼──────────────────►        La
                       │           │                                                      │                 │                                                      │           │
                       │           │     ┌────────────────┐       ┌────────────────┐      │                 │     ┌────────────────┐       ┌────────────────┐      │           │
                       │           │     │                │       │                │      │                 │     │                │       │                │      │           │
                       │      ─────┼─────┼──►  Da         │       │     La      ───┼──────┼──►       ───────┼─────┼─►    Db        │       │     Lb       ──┼──────┼───►       │
                       │           │     │                │       │                │      │                 │     │                │       │                │      │           │
                       │           │     │                │       │                │      │                 │     │                │       │                │      │           │
                       │           │     └────────────────┘       └────────────────┘      │                 │     └────────────────┘       └────────────────┘      │           │
                       │           │                                                      │                 │                                                      │           │
                       │           │     ┌────────────────┐       ┌────────────────┐      │                 │     ┌────────────────┐       ┌────────────────┐      │           │
                       │           │     │                │       │                │      │                 │     │                │       │                │      │           │
                       │           │     │                │       │                │      │                 │     │                │       │                │      │           │
                       │     ──────┼─────┼───►  Db        │       │     Lb      ───┼──────┼──►        ──────┼─────┼──►   Dc        │       │     Lc      ───┼──────┼───►       │
                       │           │     │                │       │                │      │                 │     │                │       │                │      │           │
                       │           │     └────────────────┘       └────────────────┘      │                 │     └────────────────┘       └────────────────┘      │           │
  Dc                   │           │                                                      │                 │                                                      │           │
                       │           └──────────────────────────────────────────────────────┘                 └──────────────────────────────────────────────────────┘           │
───────────────────────┼─►                                                                                                                                                  ───┼──────────────────►        Lc
                       │                                                                                                                                                       │
                       │                                                                                                                                                       │
                       └───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```


```
  ┌─────────┐               ┌─────────┐
  │         │               │         │
  │         │               │         │
  │         │               │         │
  │         │               │         │
  │         │               │         │
  │         │               │         │
  │         │               │         │
  │         │               │         │
┌─┼─────────┼───────────────┼─────────┼─┐
│ │         │               │         │ │
│ │         │               │         │ │
│ └─────────┘               └─────────┘ │
│                                       │
│             ┌────┐  ┌────┐            │
│             │    │  │    │            │
│         ┌───┼────┼──┼────┼───┐        │
│         │   │    │  │    │   │        │
│         │   │    │  │    │   │        │
│         │   └────┘  └────┘   │        │
│         │                    │        │
│         │                    │        │
│         │   ┌────┐  ┌────┐   │        │
│         │   │    │  │    │   │        │
│         │   │    │  │    │   │        │
│         └───┼────┼──┼────┼───┘        │
│             │    │  │    │            │
│             └────┘  └────┘            │
│                                       │
│                                       │
│             ┌────┐  ┌────┐            │
│             │    │  │    │            │
│         ┌───┼────┼──┼────┼───┐        │
│         │   │    │  │    │   │        │
│         │   │    │  │    │   │        │
│         │   └────┘  └────┘   │        │
│         │                    │        │
│         │                    │        │
│         │   ┌────┐  ┌────┐   │        │
│         │   │    │  │    │   │        │
│         │   │    │  │    │   │        │
│         └───┼────┼──┼────┼───┘        │
│             │    │  │    │            │
│             └────┘  └────┘            │
│                                       │
│                                       │
│  ┌────────┐                ┌────────┐ │
│  │        │                │        │ │
│  │        │                │        │ │
└──┼────────┼────────────────┼────────┼─┘
   │        │                │        │
   │        │                │        │
   │        │                │        │
   │        │                │        │
   │        │                │        │
   │        │                │        │
   └────────┘                └────────┘
```

```
┌───────┬─────────────────────────────────────────────────────────────┬───────┐
│       │                                                             │       │
│       │                            ┌───────────────────────────┐    │       │
│       │                            │                           │    │       │
│       │           ┌────┬───┬────┐  │   ┌────┬───┬────┐         │    │       │
│       │           │    │   │    │  │   │    │   │    │         │    │       │
│       ├──────────►│ Da │   │ La ├──┘ ┌►│ Db │   │ Lb ├───► Db  └───►│       │
│       │           │    │   │    │    │ │    │   │    │              │       │
├───────┤           │    │   │    │    │ │    │   │    │              ├───────┤
│       │           ├────┤   ├────┤    │ ├────┤   ├────┤              │       │
│       ├───┐       │    │   │    ├────┘ │    │   │    ├─────────────►│       │
│       │   │  0 ──►│ Db │   │ Lb │      │ Dc │   │ Lc │              │       │
│       │   │       │    │   │    │   ┌─►│    │   │    │              │       │
│       │   │       └────┴───┴────┘   │  └────┴───┴────┘              │       │
│       │   │                         │                               │       │
│       │   └─────────────────────────┘                               │       │
│       │                                                             │       │
└───────┴─────────────────────────────────────────────────────────────┴───────┘
```

```
┌─────┬───────────────────────────────────────────────────────────┬─────┐
│     │                                                           │     │
│     │                            ┌───────────────────────────┐  │     │
│     │                            │                           │  │     │
│     ├──────┐    ┌────┬───┬────┐  │   ┌────┬───┬────┐         │  │     │
│     │      │    │    │   │    │  │   │    │   │    │         │  │     │
│     │      └───►│ Da │   │ La ├──┘ ┌►│ Db │   │ Lb ├───► Db  └─►│     │
│     │           │    │   │    │    │ │    │   │    │            │     │
├─────┤           ├────┤   ├────┤    │ ├────┤   ├────┤            ├─────┤
│     │           │    │   │    ├────┘ │    │   │    │            │     │
│     │      0 ──►│ Db │   │ Lb │      │ Dc │   │ Lc ├───────────►│     │
│     │           │    │   │    │   ┌─►│    │   │    │            │     │
│     ├───┐       └────┴───┴────┘   │  └────┴───┴────┘            │     │
│     │   │                         │                             │     │
│     │   └─────────────────────────┘                             │     │
│     │                                                           │     │
└─────┴───────────────────────────────────────────────────────────┴─────┘
```
```
┌─────┬───────────────────────────────────────────────────────────┬─────┐
│     │                            ┌───────────────────────────┐  │     │
│     │                            │                           │  │     │
│     ├──────┐    ┌────┬───┬────┐  │   ┌────┬───┬────┐         │  │     │
│ Δa  │      │    │    │   │    │  │   │    │   │    │         │  │ Λa  │
│     │      └───►│ Δa │   │ Λa ├──┘ ┌►│ Δb │   │ Λb ├───► Δb  └─►│     │
│     │           │    │   │    │    │ │    │   │    │            │     │
├─────┤           ├────┤ Φ ├────┤    │ ├────┤ Ψ ├────┤            ├─────┤
│     │           │    │   │    ├────┘ │    │   │    │            │     │
│     │      0 ──►│ Δb │   │ Λb │   ┌─►│ Δc │   │ Λc ├───────────►│ Λc  │
│ Δc  │───┐       │    │   │    │   │  │    │   │    │            │     │
│     │   │       └────┴───┴────┘   │  └────┴───┴────┘            │     │
│     │   └─────────────────────────┘                             │     │
└─────┴───────────────────────────────────────────────────────────┴─────┘
```

### Binning

### Replicating payoffs

1. why, talk about active management vs. passive LPing
2. provide an example of a polynomial trading function getting interpolated with CSMMs 
3. payoffs vs. focus on trading functions?