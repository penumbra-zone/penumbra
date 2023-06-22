## Overview

The problem of routing a desired trade on Penumbra can be thought of as a special case of the [minimum-cost flow problem](https://en.wikipedia.org/wiki/Minimum-cost_flow_problem): given an input of the source asset `S`, we want to find the flow to the target asset `T` with the minimum cost (the best execution price).

Penumbra's liquidity positions are each individual constant-sum AMMs with their own reserves, fees, and price.  Each position allows exchanging some amount of the asset `A` for asset `B` at a fixed price, or vice versa.

This means that liquidity on Penumbra can be thought of as existing at two different levels of resolution:

- a "macro-scale" graph consisting of trading pairs between assets
- a "micro-scale" multigraph with one edge for each individual position.

In the "micro-scale" view, each edge in the multigraph is a single position, has a linear cost function and a maximum capacity: the position has a constant price (marginal cost), so the cost of routing through the position increases linearly until the reserves are exhausted.

In the "macro-scale" view, each edge in the graph has a **convex** cost function, representing the aggregation of all of the positions on that pair: as the cheapest positions are traded against, the price (marginal cost) increases, and so the cost of routing flow through the edge varies with the amount of flow.

To route trades on Penumbra, we can switch back and forth between these two views, solving routing by _spilling successive shortest paths_.

In the _spill phase_, we perform a bounded graph traversal of the macro-scale graph from the source asset $S$ to the target asset $T$, ignoring capacity constraints and considering only the best available price for each pair.  At the end of this process, we obtain a best _fill path_ $P$ with price $p$, and a second-best _spill path_ $P'$ with _spill price_ $p' > p$.

In the _fill phase_, we increase capacity routed on the fill path $P$, walking up the joint order book of all pairs along the path $P$, until the resulting price would exceed the spill price $p'$. At this point, we are no longer sure we're better off executing along the fill path, so we switch back to the spill phase and re-route.

Since the fill path $P$ and the spill path $P'$ might overlap, it's possible that the spill price $p'$ might not exist after filling along $P$, so we will have to re-run the graph traversal on the updated state in each spill phase.  In the future, we could explore ways to reuse computation from previous routings, but for now, it's simpler to start from scratch.

The intuition on why this should be a reasonable approach is the expectation that in practice, routes will break down coarsely over different paths and finely among positions within a path, rather than having many tiny positions on many different paths, all of which are price-competitive.

## Path search

We aim to find the most efficient route from a source asset ($S$) to a destination asset ($T$) subject to a routing price limit. The high-level idea is to use a variant of Bellman-Ford to explore paths via edge relaxation, bounding the size of the graph traversal by constraining both the overall path length as well as the maximum out-degree during edge relaxation.

### Candidate Sets

We want to bound the maximum number of possible path extensions we consider, but this requires choosing which neighbors to consider as candidates to extend a path along. We don't want to make these choices a priori, but if the candidate selection is entirely based on on-chain metrics like liquidity, price, trading activity, etc., it might be possible for someone to manipulate the routing algorithm.

As a compromise, we define a _candidate set_ $c(A)$ for each asset $A$ with a mix of hardcoded and dynamic candidates.

One idea for $c(A)$ is to choose:

- the target asset $T$
- the staking token $U$
- the IBC-similar[^1] asset $A' \neq A$ with the largest market cap
- the $N$ assets with most liquidity from $A$

[^1]: We say that two IBC-bridged assets $A$, $A'$ are IBC-similar if they are different path representations of the same underlying asset (e.g., ATOM via Cosmos Hub vs ATOM via Osmosis)

In this way, even if the dynamically-selected candidates were completely manipulated, routing would still consider reasonable routes (e.g., through the native token). 


### Traversal

Unfortunately, we don't have a _distance_ metric, since we can only compare costs between paths with the same start and end, so our ability to prune the search space is limited.

What we can do is something similar to [Bellman-Ford](https://en.wikipedia.org/wiki/Bellman%E2%80%93Ford_algorithm)/[SPFA](https://en.wikipedia.org/wiki/Shortest_path_faster_algorithm), where we repeatedly relax paths along candidate edges. We maintain a record of the shortest known path from $S$ to each intermediate node $A$. These are the candidate paths. We perform $\ell$ iterations, where $\ell$ is the maximum path length bound.

At each iteration, we iterate over each candidate path, and relax it along each of its candidate edges. For each relaxation, we use its price to compare-and-swap the relaxed path against the existing best-path from the source to the new end.

The [SPFA](https://en.wikipedia.org/wiki/Shortest_path_faster_algorithm) optimization is to also record whether the best-path-to-$A$ was updated in the last iteration. If not, we know that every possible relaxation is worse than a known alternative, so we can skip relaxing it in later iterations.


We consider all relaxations for a given depth in concurrent tasks. We use a shared cache that records the best intermediate route for each asset.

After `max_length` iterations, the `PathEntry` for target asset $T$ contains the shortest path to $T$, along with a spill path (the next-best path to $T$).


## Routing Execution (Fill Phase)

In the fill phase, we have a path to fill, represented as a `Vec<asset::Id>` or  $(\mathsf a_0, \mathsf a_1, \ldots, \mathsf a_n)$, and an estimate of the spill price for the next-best path.  The spill price indicates how much we can fill and still know we're on the optimal route. Our goal is to fill as much of the trade intent $\Delta$ as possible, until we get a marginal price that's worse than the spill price.

### Optimal execution along a route: `fill_route`

After performing path search, we know that a route $\langle a_{src}, ..., a_{dst}\rangle$ exists with positive capacity and a path price that is equal to or better than some spill price. Executing over the route means filling against the best available positions at each hop. At each filling iteration, we need to discover what's the maximum amount of flow that we can push through without causing a marginal price increase.

To achieve this, the DEX engine assembles a _frontier of positions_ $F$, which is comprised of the best positions for each hop along the path: $F = (P_1, P_2 ..., P_n)$ where $P_i$ is the best position for the directed pair $a_{i-1} \rightarrow a_i$.


```
///   Swapping S for T, routed along (S->A, A->B, B->C, C->T),
///   each pair has a stack of positions that have some capacity and price
/// 
///   ┌───┐    ┌─────┐       ┌─────┐       ┌─────┐       ┌─────┐
///   │   │    │     │       │ B1  │       │     │       │     │
///   │   │    │     │    ┌► │     ├───┐   │ C1  │       │ T1  │
///   │   ├──► │ A1  ├────┘  ├─────┤   └─► │     ├───┐   │     │
///   │   │    │     │       │     │       │     │   │   │     │
///   │   │    │     ├─────► │     ├─────► │     │   ├─► │     │
///   │ S │    ├─────┤       │ B2  │       │     ├───┴─► │     │
///   │   │    │     ├─────► │     ├──┐    ├─────┤       │     │
///   │   ├──► │ A2  │       │     │  └──► │ C2  ├─────► │     │
///   │   │    │     │       ├─────┤       ├─────┤       ├─────┤
///   │   │    │     ├─────► │ B3  ├─────► │ C3  ├─────► │     │
///   └───┘    ├─────┤       │     │       │     │       │     │
///            │     │       ├─────┤       ├─────┤       │ T2  │
///            │     │       │     │       │     │       │     │
///            │     │       │     │       │     │       │     │
///            │     │       ├─────┤       ├─────┤       │     │
///            │     │       │     │       │     │       │     │
///            └─────┘       └─────┘       └─────┘       └─────┘
/// 
///              A              B             C             T
/// 
/// 
/// ┌────┬────────────────┐  As we route input along the route,
/// │ #  │  Frontier      │  we exhaust the inventory of the best
/// │    │                │  positions, thus making the frontier change.
/// ├────┼─────────────── │
/// │ 1  │ A1, B1, C1, T1 │
/// │    │                │  Routing execution deals with two challenges:
/// ├────┼─────────────── │
/// │ 2  │ A1, B2, C1, T1 │  -> capacity constraints along a route: some position
/// │    │                │     provision less inventory than needed to fill
/// ├────┼─────────────── │     the swap.
/// │ 3  │ A2, B2, C2, T1 │
/// │    │                │
/// ├────┼─────────────── │  -> exactly filling all the reserves so as to not
/// │ 4  │ A2, B3, C3, T2 │     leave any dust positions behind.
/// │    │                │
/// └────┴────────────────┘  Our example above starts with a frontier where B1 is
///                          the most constraining position in the frontier set.
///                          Routing as much input as possible means completely
///                          filling B1. The following frontier assembled contains
///                          the next best available position for the pair A->B,
///                          that is B2.
```



#### Sensing constraints

In the simple case, each position in the frontier has enough inventory to satisfy routing to the next hop in the route until the target is reached. But this is not always the case, so we need to be able to sense which positions in the frontier are constraint and pick the most limiting one.

We implement a capacity sensing routing that returns the index of the most limiting constraint in the frontier. The routine operates statelessly, and works as follow:
First, it pulls every position in the frontier and simulates execution of the input against those positions, passing the filled amounts forward to the next position in the frontier. If at some point an unfilled amount is returned, it means the current position is a constraint. When that happens, we record the index of the position, passing the output of the trade to the next position. This last point is important because it means that picking the most limiting constraint is equivalent to picking the last recorded one.

Summary:


1. Pull each position in the frontier $F = \langle P_1, ... P_n\rangle$
2. Execute the entirety of the input  $\Delta$ against the first position
3. If a trade returns some unfilled amount $\Lambda_1$ that means we have sensed a constraint, record the position and proceed with executing $\Lambda_2$ against the next position.
4. Repeat until we have traversed the entire frontier.
 

```
/// We show a frontier composed of assets: <S, A, B, C, T>, connected by
/// liquidity positions of various capacity and cost. The most limiting
/// constraint detected is (*).
///         ┌─────┐      ┌─────┐      ┌─────┐      ┌─────┐      ┌─────┐
///         │     │      │     │      │     │      │     ├──────┤     │
///         │     │      │     │      │     │      │     │      │     │
///         │     ├──────┤     │      │     │      │     │      │     │
/// Swap In │     │      │     │      │     │      │     │      │     │
///         │     │      │     ├──────┤     │      │     │      │     │
///     │   │     │      │     │      │     ├──────┤     │      │     │
///     └─► │ src │      │  A  │      │  B  │  (*) │  C  │      │ dst ├─────┐
///         │     │      │     │      │     ├──┬───┤     │      │     │     │
///         │     │      │     │      │     │  │   │     │      │     │     ▼
///         │     │      │     ├───┬──┤     │  │   │     │      │     │ Swap Out
///         │     ├──┬───┤     │   │  │     │  │   │     │      │     │
///         │     │  │   │     │   │  │     │  │   │     │      │     │
///         │     │  │   │     │   │  │     │  │   │     ├───┬──┤     │
///         └─────┘  │   └─────┘   │  └─────┘  │   └─────┘   │  └─────┘
///                  │             │           │             │
///                  └─────────────┴─────┬─────┴─────────────┘
///                                      │
///                                      │
///                                      ▼
/// 
///                           Capacity constraints at each
///                           hop on the frontier.
```
#### Filling

Filling involves transforming an input asset $A$ into a corresponding output asset $B$, using a trading function and some reserves. If no constraint was found, we can simply route forward from the source, and execute against each position in the frontier until we have reached the target asset. However, if a constraint was found, we have to determine the maximum amount of input $\Delta^*$ that we can route through and make sure that we consume the reserves of the constraining position _exactly_, to not leave any dust behind.


##### Filling forward

Filling forward is trivial. For a given input, we pull each position and fill them. Since no unfilled amount is returned we can simply proceed forward, updating the reserves and pulling the next position:

$\Lambda_2 = \Delta_1 \frac {p} {q \gamma}$

##### Filling backward

Filling backward is more complicated because we need to ensure every position preceding the constraining hop is zeroed out. This is because the DEX engine only has access to a finite amount of precision and as a result perform divison can be lossy by some $\epsilon$, perpetually preventing a position to be deindexed.

Suppose that the limiting constraint is a index $j$, we compute $\Delta_1^* = \frac {p_2 \gamma} {p_1} R_2$

The goal is to propagate rounding loss backwards to the input and forwards the output. That means that for a constraint at index $j$, we fill backward from $j$ to the first position, and forward from $j$ to the last position. For backward fills, we compute $\Delta_1^* = \Lambda_2 \frac {p_2 \gamma} {p_1}$ and manually zero-out the $R_2$ of each position. The forward fill part works as described previously. There is no extra work to do because that segment of the path contains no constraints as we have reduced the flow to match the exact capacity of the frontier.

### Termination conditions
Termination conditions:
- we have completely filled the desired fill amount, $\Delta$,
- we have a partial fill, $\Delta^* \lt \Delta$, but the marginal price has reached the spill price.
