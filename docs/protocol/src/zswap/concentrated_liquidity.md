# Concentrated Liquidity

ZSwap offers similar market functions to a central limit order book (CLOB), a `Swap` action is equivalent to a market order that takes liquidity, and opening a liquidity position is similar to posting limit orders that quote one or both sides of the pair.


### Liquidity positions

At a high level, a liquidity position sets aside reserves for one or both assets and specifies  a fixed exchange rate. Algebraically, a liquidity position is defined by a constant-sum trading function $\phi(R) = p_1 \cdot R_1 + p_2 \cdot R_2$, which involves reserves and prices of the two assets, denoted by $(R_1, p_1)$ and $(R_2, p_2)$, respectively.

Each liquidity position in ZSwap is its own AMM with a constant-sum invariant, this means that routing happens over the space

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

Composing liquidity positions means aggregating two liquidity positions for distinct but overlapping pairs (e.g. `A <> B <> C => A <> C`).

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

### Binning

### Replicating 