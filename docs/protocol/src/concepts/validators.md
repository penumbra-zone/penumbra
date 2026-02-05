# Validators

Validators in Penumbra undergo various transitions depending on chain activity.

## State Machine

```
          ┌───────────────────────────────────────────────────────┐
          │                      ┌──────────────────────────────┐ │
          ▼                      ▼                              │ │
  ╔═══════════════╗       ┌────────────┐                        │ │
 ▶║    Defined    ║◀─────▶│  Disabled  │                        │ │
  ╚═══════════════╝       └────────────┘                        │ │
         │                      │                               │ │
         │                      ▼                               │ │
         │               ┏━━━━━━━━━━━━┓                         │ │
         └──────────────▶┃            ┃                         │ │
                         ┃ Tombstoned ┃◀────────────┐           │ │
         ┌──────────────▶┃            ┃             │           │ │
         │               ┗━━━━━━━━━━━━┛             │           │ │
         │                      ▲                   │           │ │
         │                      │           ┌──────────────┐    │ │
  ┌─────────────┐        ┌────────────┐     │              │◀───┘ │
  │   Jailed    │◀───────│   Active   │◀───▶│   Inactive   │      │
  └─────────────┘        └────────────┘     │              │◀─────┘
         │                                  └──────────────┘
         │                                          ▲
         └──────────────────────────────────────────┘

  ╔═════════════════╗
  ║ starting state  ║
  ╚═════════════════╝
  ┏━━━━━━━━━━━━━━━━━┓
  ┃ terminal state  ┃
  ┗━━━━━━━━━━━━━━━━━┛
```

Validators become known to the chain either at genesis, or by means of a transaction with a `ValidatorDefinition` action. Validators transition through six states:

* **Defined**: the initial state for non-genesis validators. The validator definition has been published, but the delegation pool has not yet reached the minimum stake threshold required to be indexed.
* **Inactive**: a validator whose delegation pool meets the minimum stake threshold but is not large enough to be in the active consensus set.
* **Active**: a validator whose delegation pool is large enough to participate in consensus and must meet uptime requirements.
* **Jailed**: a validator that has been slashed for downtime, that may return later.
* **Tombstoned**: a validator that has been permanently slashed for byzantine misbehavior and may not return.
* **Disabled**: a validator that has been manually disabled by the operator.

## Validator Lifecycle

### Genesis Validators

Validators specified in the genesis config begin in the **Active** state, with whatever stake was allocated to their delegation pool at genesis. Their stake is immediately bonded.

### New Validators

New validators (created via `ValidatorDefinition` transactions) begin in the **Defined** state with zero voting power and unbonded stake. At this point, the validator is known to the chain, and stake can be contributed to its delegation pool.

When the validator's delegation pool reaches the `min_validator_stake` threshold (a chain parameter), the validator transitions to the **Inactive** state at the next epoch boundary. This transition is checked automatically during epoch processing.

Stake contributed to a Defined or Inactive validator's delegation pool does not earn rewards (the validator's rates are held constant), and is not bonded, so undelegations can be claimed immediately without waiting for an unbonding period.

### Becoming Active

The chain chooses a validator limit N as a consensus parameter. When a validator's delegation pool is in the top N validators by voting power, it moves into the **Active** state during the next epoch transition.

Active validators:
- Participate in consensus and are communicated to CometBFT
- Earn rewards (their exchange rates increase each epoch)
- Have bonded stake (undelegations require waiting through an unbonding period)

### Leaving the Active Set

An active validator can exit the consensus set in four ways:

**1. Jailed for downtime**

If a validator misses too many blocks, it is jailed and slashed. This can happen in any block, triggering an unscheduled epoch transition. Jailed validators are immediately removed from the consensus set. The validator's rates are updated to record the slashing penalty. Validators jailed for downtime are not permanently prohibited from participation in consensus; their operators can re-activate them by re-uploading the validator definition with the `enabled` flag set to true.

**2. Tombstoned for misbehavior**

If evidence of byzantine misbehavior is detected, the validator is tombstoned and slashed. This can happen in any block, triggering an unscheduled epoch transition. Tombstoned validators are immediately removed from the consensus set. The validator's rates are updated to record the slashing penalty. Tombstoned validators are permanently prohibited from participation in consensus (though their operators can create new identity keys if they choose).

**3. Manually disabled**

The operator can disable the validator by uploading a new validator definition with `enabled: false`. The validator enters the **Disabled** state and does not participate in consensus. Its rates are held constant (no rewards). The operator can later re-enable the validator, which transitions it to **Inactive** (or **Defined** if below the minimum stake threshold).

**4. Displaced by higher-stake validator**

If another validator accumulates more stake and pushes this validator out of the top N, the validator transitions to **Inactive** (or **Defined** if it falls below the minimum stake threshold). It does not participate in consensus and its rates are held constant.

### Slashing and Penalties

When a validator is slashed (either jailed or tombstoned), a penalty is recorded in the chain state for that epoch. This penalty affects delegators through the undelegation mechanism:

- Undelegations produce **unbonding tokens** that must be held for an unbonding period
- During the unbonding period, any penalties applied to the validator accumulate
- When the unbonding period ends and the delegator claims their stake via `UndelegateClaim`, all accumulated penalties are applied

This design ensures that delegators cannot escape slashing by racing to undelegate before evidence is processed.

### Bonding States

In addition to the validator state, each validator has a **bonding state** that tracks whether its stake is bonded:

* **Bonded**: The validator is in the active set. Delegated stake is locked and subject to slashing. Undelegations require waiting through the unbonding period.
* **Unbonding**: The validator has left the active set and its stake is transitioning to unbonded. A specific block height marks when unbonding completes.
* **Unbonded**: The validator is not in the active set and its stake is not locked. Undelegations can be claimed immediately.

The bonding state is managed automatically based on validator state transitions:
- Active validators are always Bonded
- When a validator leaves Active, it enters Unbonding (unless Tombstoned, which goes directly to Unbonded)
- After the unbonding period elapses, the validator becomes Unbonded
