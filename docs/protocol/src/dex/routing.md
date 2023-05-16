# On-Chain Routing

The problem of routing a desired trade on Penumbra can be thought of as a special case of the [minimum-cost flow problem](https://en.wikipedia.org/wiki/Minimum-cost_flow_problem): given an input of the source asset `S`, we want to find the flow to the target asset `T` with the minimum cost (the best execution price).

Penumbra's liquidity positions are each individual constant-sum AMMs with their own reserves, fees, and price.  Each position allows exchanging some amount of the asset `A` for asset `B` at a fixed price, or vice versa.

This means liquidity on Penumbra can be thought of as existing at two different levels of resolution: a "macro-scale" graph consisting of trading pairs between assets, and a "micro-scale" multigraph with one edge for each individual position.

In the "micro-scale" view, each edge in the multigraph is a single position, has a linear cost function and a maximum capacity: the position has a constant price (marginal cost), so the cost of routing through the position increases linearly until the reserves are exhausted.

In the "macro-scale" view, each edge in the graph has a **convex** cost function, representing the aggregation of all of the positions on that pair: as the cheapest positions are traded against, the price (marginal cost) increases, and so the cost of routing flow through the edge varies with the amount of flow.

## Idea

To route trades on Penumbra, we can switch back and forth between these two views, solving routing by _spilling successive shortest paths_.

In the _spill phase_, we perform a bounded graph traversal of the macro-scale graph from the source asset $S$ to the target asset $T$, ignoring capacity constraints and considering only the best available price for each pair.  At the end of this process, we obtain a best _fill path_ $P$ with price $p$, and a second-best _spill path_ $P'$ with _spill price_ $p' > p$.

In the _fill phase_, we increase capacity routed on the fill path $P$, walking up the joint order book of all pairs along the path $P$, until the resulting price would exceed the spill price $p'$. At this point, we are no longer sure we're better off executing along the fill path, so we switch back to the spill phase and re-route.

Since the fill path $P$ and the spill path $P'$ might overlap, it's possible that the spill price $p'$ might not exist after filling along $P$, so we will have to re-run the graph traversal on the updated state in each spill phase.  In the future, we could explore ways to reuse computation from previous routings, but for now, it's simpler to start from scratch.

The intuition on why this should be a reasonable approach is the expectation that in practice, routes will break down coarsely over different paths and finely among positions within a path, rather than having many tiny positions on many different paths, all of which are price-competitive.

## Pathfinding (Spill Phase)

The high-level idea is to use a variant of Bellman-Ford to explore paths via edge relaxation, bounding the size of the graph traversal by constraining both the overall path length as well as the maximum out-degree during edge relaxation.

### Path Length

This should be a top-level parameter; lower is better.  For trades, length $3$ or $4$ seems useful.  We want to be able to compose stableswaps between different bridge representations with a trading pair providing price discovery between dissimilar assets.  Length $3$ could be a reasonable starting point.

### Candidate Sets

We want to bound the maximum number of possible path extensions we consider, but this requires choosing which neighbors to consider as candidates to extend a path along. We don't want to make these choices a priori, but if the candidate selection is entirely based on on-chain metrics like liquidity, price, trading activity, etc., it might be possible for someone to manipulate the routing algorithm.

As a compromise, we define a _candidate set_ $c(A)$ for each asset $A$ with a mix of hardcoded and dynamic candidates.

One idea for $c(A)$ is to choose:

- the target asset $T$
- the staking token $U$
- the IBC-similar[^1] asset $A' \neq A$ with the largest market cap
- the $N$ assets with most liquidity from $A$

[^1]: We say that two IBC-bridged assets $A$, $A'$ are IBC-similar if they are different path representations of the same underlying asset (e.g., ATOM via Cosmos Hub vs ATOM via Osmosis)

In this way, even if the dynamically-selected candidates were completely manipulated, routing would still consider reasonable routes (e.g., through the staking token). 

Choosing $N = 2$ would result in $|c(A)| = 5$, which seems reasonable together with the path length bounds: length $3$ searching could consider at most $5^3 = 125$ paths, and length $5$ searching could consider at most $5^5 = 3125$ paths (in practice, much fewer, because of pruning as described below).

The liquidity-based candidates will require us to maintain an index of the total liquidity on each pair. The IBC-similar candidates will require us to maintain an index of IBC families. As an intermediate step along the way, we could consider a stub candidate set consisting of $T$, $U$, and 3 assets chosen in an implementation-defined way (e.g., the 3 assets for which positions exist in storage order, or something), if that's easier to get started.

### Paths

The information we need to record about a path is bundled into a `Path` structure, something like

```rust
struct Path<S: StateRead + StateWrite> {
    /// The start point of the path
    pub start: asset::Id,
    /// The nodes along the path, implicitly defining the end
    pub nodes: Vec<asset::Id>,
    /// An estimate of the end-to-end effective price along the path
    pub price: U128x128,
    /// A forked view of the state after traveling along this path. 
    pub state: StateDelta<S>,
}

impl<S: StateRead + StateWrite> Path<S> {
    pub fn end(&self) -> &asset::Id {
        self.nodes.last().unwrap_or(&self.start)
    }
    
    pub fn begin(start: asset::Id, state: StateDelta<S>) {
        Self {
            start,
            nodes: Vec::new(),
            price: 1u64.into(),
            state,
        }
    }
    
    pub fn state(&self) -> &StateDelta<S> {
        &self.state
    }
}
```

The `Path` structure maintains the path itself (the list of assets along the path), an end-to-end price estimate for an infinitesimally-sized trade along the path, and a state fork used to ensure that the path doesn't double-count liquidity during routing.

To extend a `Path` to a `new_end`, we query for the least-price position on the pair `(end, new_end)`, multiply its effective price into `price`, push `new_end` into `nodes`, fork the `state`, and deindex the position in the forked state (to ensure we never double-count the same liquidity, even if we routed back along a cycle).  This could look something like this:

```rust
impl<S: StateRead + StateWrite> Path<S> {
    // We can't clone, because StateDelta only has an explicit fork() on purpose
    pub fn fork(&self) -> Self {
        Self {
            start: self.start.clone(),
            nodes: self.nodes.clone(),
            price: self.price.clone(),
            state: self.state.fork(),
        }
    }
    
    // Making this consuming forces callers to explicitly fork the path first.
    pub async fn extend_to(mut self, new_end: asset::Id) -> Result<Option<Path<S>>> {
        let Some(position) = state.best_position(self.end(), &new_end).await? else {
            return Ok(None)
        };
        // Deindex the position we "consumed" in this and all descendant state forks,
        // ensuring we don't double-count liquidity while traversing cycles.
        self.state.deindex_position(position.id());
        
        // Update and return the path.
        self.price *= best_price_position.effective_price();
        self.nodes.push(new_end)
        Ok(Some(self))
    }
}
```

The important detail here is that we don't want the future returned by the path extension method to have a lifetime bounded by `'self`, because we want it to be `'static` and therefore spawnable, allowing us to explore all possible path extensions in parallel.  This might require bounding `S: 'static` as well, although every instantiation of `S` we use should be anyways.

### Traversal

Unfortunately, we don't have a _distance_ metric, since we can only compare costs between paths with the same start and end, so our ability to prune the search space is limited.

What we can do is something similar to [Bellman-Ford](https://en.wikipedia.org/wiki/Bellman%E2%80%93Ford_algorithm)/[SPFA](https://en.wikipedia.org/wiki/Shortest_path_faster_algorithm), where we repeatedly relax paths along candidate edges. We maintain a record of the shortest known path from $S$ to each intermediate node $A$. These are the candidate paths. We perform $\ell$ iterations, where $\ell$ is the maximum path length bound.

At each iteration, we iterate over each candidate path, and relax it along each of its candidate edges. For each relaxation, we use its price to compare-and-swap the relaxed path against the existing best-path from the source to the new end.

The [SPFA](https://en.wikipedia.org/wiki/Shortest_path_faster_algorithm) optimization is to also record whether the best-path-to-$A$ was updated in the last iteration. If not, we know that every possible relaxation is worse than a known alternative, so we can skip relaxing it in later iterations.

Ideally, we would consider all relaxations for a given depth in concurrent tasks. To do this, we need to share the path registry, with something like
```rust
// Shared between tasks as Arc<Mutex<PathCache>>
pub struct PathCache<S>(pub BTreeMap<asset::Id, (Path<S>, bool)>);

impl<S> PathCache<S> {
    // Initializes a new PathCache with the identity path.
    pub fn begin(start: asset::Id, state: StateDelta<S>) -> Arc<Mutex<Self>> {
        let identity = Path::begin(start, state);
        let mut cache = BTreeMap::new();
        cache.insert(start, (identity, true));
        Arc::new(Mutex::new(Self(cache)))
    }
    
    // Consider a new candidate path.
    pub fn consider(&mut self, path: Path<S>) {
        self.0.entry(*path.end())
            .and_modify(|existing| {
                // compare-and-swap
            })
            .or_insert_with(|| (path, true))
    }
}
```

Then, at each iteration, we can extract the active paths...
```rust
let active_paths = cache.lock().0.values()
                .filter_map(|(path, active)| if active {
                    Some(path.fork())
                } else {
                    None
                })
                .collect::<Vec<_>>();
```
...and then concurrently relax them along candidate edges:
```rust
let mut js = JoinSet::new();
for path in active_paths {
    let cache2 = cache.clone();
    js.spawn(async move {
        // Exact candidate set computation TBD
        // (need to plumb in the source and target?)
        let candidates = path.state().candidates(path.end()).await?;
        let mut js2 = JoinSet::new();
        for new_end in candidates {
            let new_path = path.fork();
            let cache3 = cache2.clone();
            js2.spawn(async move {
                let new_path = new_path.extend_to(new_end).await?;
                cache3.lock().consider(new_path);
                anyhow::Ok(())
            })
        }
        // Wait for all candidates to be considered
        while let Some(task) = js2.join_next().await {
            task??
        }
    })
}
// Wait for all candidates of all active paths to be considered
while let Some(task) = js.join_next().await {
    task??
}
```
After `max_length` iterations, the entry in the path cache for the target asset $T$ is the shortest path to $T$. This still isn't quite what we want, though, because we want not just the shortest path but also the next-shortest path.  To fix that, we can change the `PathCache` to store the second-best path while we write in the first one.

Once we obtain the best path to $T$ and the spill price (from the second-best path), we're ready to move to the fill phase.

## Execution (Fill Phase)

In the fill phase, we have a path to fill along, represented as a `Vec<asset::Id>`, and an estimate of the spill price for the next-best path.  The spill price indicates how much we can fill and still know we're on the optimal route.  Our goal is to fill as much of the trade intent $\Delta_0$ as possible, until we get a marginal price that's worse than the spill price.

Termination conditions:
- we have completely filled the desired fill amount $\Delta_0$
- we have partially filled $\Delta'_0 < \Delta_0$, and filling more would have a higher marginal price than the spill price.

Suppose the path is $(\mathsf a_0, \mathsf a_1, \ldots, \mathsf a_n)$, and write $P_i$ for the set of open positions on the pair $(\mathsf a_{i-1}, \mathsf a_i)$, ordered by price.
Filling some amount $\Delta_0'$ along this path will use up the open positions along each component pair in order.  Write $(k_1, \ldots, k_n)$ for the number of positions consumed (i.e., have their reserves completely exhausted) during the fill.

While filling, our problem is to determine which position along the route we should consume next -- i.e., which position is the constraining one. We want to do this without assuming the existence of a numeraire, which we can do by simulating a test execution of the entire reserves of the active position on the first hop through to the end of the path. At each step, the intermediate output from the previous trade will either be greater than or less than the reserves of the active position on the next hop.  If the output is less than the reserves of the next position, the previous position was a capacity constraint.  The last capacity constraint on the path is the one we lift first.

- [ ] Restructure slightly in terms of frontiers
- [ ] Explain traces

