# Debugging a Penumbra node

Below are a list of FAQs about running a Penumbra node.

## How do I check whether my node is connected to other peers?
Poll the CometBFT RPC for current number of peers:
```
curl -s http://localhost:26657/net_info | jq .result.n_peers
```

## How do I check whether my node is synchronized with the network?
Poll the CometBFT RPC for sync status:

```
curl -s http://localhost:26657/status | jq .result.sync_info
```

Specifically, check that `catching_up=false`. You can also compare the `latest_block_height`
value with the tip of the chain visible when running `pcli view sync`.

## How long does it take to synchronize with the network?
A new node will sync at a rate of approximately 100,000 blocks per 6h.

## How do I check whether my validator is active?

You can view the list of known validators by running:

```
pcli query validator list --show-inactive
```

Remember that it will take time for delegations made against your validator
to become bonded. You can check how long this will take by running:

```
pcli query chain info --verbose
```

Inspect the values for `Current Block Height`, `Current Epoch`, and `Epoch Duration`.
You'll need to wait until the next epoch boundary post-delegation for the delegated weight
to be computed in your validator's voting power.
