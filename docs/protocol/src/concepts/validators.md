# Validators

Validators in Penumbra undergo various transitions depending on chain activity.

```
                                 ┌────────────────────────────────────────────────────────────────────────────┐
                                 │                                                                            │
                                 │            ┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─                                           │
                                 │              Genesis Validator  │                                          │
                                 │            │                             ┏━━━━━━━━━━━━━━━━━━━━━━━┓         │
                                 │             ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┘        ┃      Tombstoned       ┃         │
                                 │                       │          ┌──────▶┃     (Misbehavior)     ┃         │
                                 │                       │          │       ┗━━━━━━━━━━━━━━━━━━━━━━━┛         │
                                 │                       │          │                                         │
┌ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─            │                       ▼          │                                         ▼
      Validator      │      ┏━━━━━━━━━┓              ╔══════╗       │                                   ┏━━━━━━━━━━━┓
│     Definition     ──────▶┃Inactive ┃─────────────▶║Active║───────┼────────────────────────────────┬─▶┃ Disabled  ┃
   (in transaction)  │      ┗━━━━━━━━━┛              ╚══════╝       │                                │  ┗━━━━━━━━━━━┛
└ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─            ▲                                  │       ┏━━━━━━━━━━━━━━━━━━━━━┓  │        │
                                 │                                  └──────▶┃ Jailed (Inactivity) ┃──┘        │
                                 │                                          ┗━━━━━━━━━━━━━━━━━━━━━┛           │
                                 │                                                     │                      │
                                 └─────────────────────────────────────────────────────┴──────────────────────┘
```

Single lines represent unbonded stake, and double lines represent bonded stake.

Validators become known to the chain either at genesis, or by means of a transaction with a `ValidatorDefinition` action in them. Validators transition through five states:

* **Inactive**, where the validator is not part of the consensus set, and the stake in the validator's delegation pool is not bonded;
* **Active**, where the validator is part of the consensus set, and the stake in the validator's delegation pool is bonded;
* **Jailed**, where the validator is not part of the consensus set, and the stake in the validator's delegation pool is not bonded;
* **Tombstoned**, where the validator is not part of the consensus set, and the stake in the validator's delegation pool is not bonded;
* **Disabled**, where the validator is not part of the consensus set, and the stake in the validator's delegation pool is not bonded.

Validators specified in the genesis config begin in the active state, with whatever stake was allocated to their delegation pool at genesis. Otherwise, new validators begin in the inactive state, with no stake in their delegation pool.  At this point, the validator is known to the chain, and stake can be contributed to its delegation pool.  Stake contributed to an inactive validator's delegation pool does not earn rewards (the validator's rates are held constant), but it is also not bonded, so undelegations are effective immediately, with no unbonding period and no output quarantine.

The chain chooses a validator limit N as a consensus parameter. When a validator's delegation pool (a) has a nonzero balance and (b) its (voting-power-adjusted) size is in the top N validators, it moves into the active state during the next epoch transition.  Active validators participate in consensus, and are communicated to Tendermint. Stake contributed to an active validator's delegation pool earns rewards (the validator's rates are updated at each epoch to track the rewards accruing to the pool). That stake is bonded, so undelegations have an unbonding period and an output quarantine. An active validator can exit the consensus set in four ways.

First, the validator could be jailed and slashed for inactivity.  This can happen in any block, not just at an epoch transition.  Jailed validators are immediately removed from the consensus set. The validator's rates are updated to price in the slashing penalty, and are then held constant. Validators jailed for inactivity are not permanently prohibited from participation in consensus, and their operators can re-activate them by re-uploading the validator definition. Stake cannot be delegated to a slashed validator. Stake already contributed to a slashed validator's delegation pool will enter an unbonding period to hold the validator accountable for any byzantine behavior during the unbonding period. Re-delegations may occur after the validator enters the "Inactive" state.

Second, the validator could be tombstoned and slashed for byzantine misbehavior.  This can happen in any block, not just at an epoch transition.  Tombstoned validators are immediately removed from the consensus set. Any pending undelegations from a slashed validator are cancelled: the quarantined output notes are deleted, and the quarantined nullifiers are removed from the nullifier set.  The validator's rates are updated to price in the slashing penalty, and are then held constant. Tombstoned validators are permanently prohibited from participation in consensus (though their operators can create new identity keys, if they'd like to). Stake cannot be delegated to a tombstoned validator. Stake already contributed to a tombstoned validator's delegation pool is not bonded (the validator has already been slashed and tombstoned), so undelegations are effective immediately, with no unbonding period and no quarantine.

Third, the validator could be manually disabled by the operator. The validator is then in the disabled state.  It does not participate in consensus, and the stake in its delegation pool does not earn rewards (the validator's rates are held constant).  The stake in its delegation pool will enter an unbonding period at the time the validator becomes disabled. The only valid state a disabled validator may enter into is "inactive", if the operator re-activates it by updating the validator definition.

Fourth, the validator could be displaced from the validator set by another validator with more stake in its delegation pool. The validator is then in the inactive state.  It does not participate in consensus, and the stake in its delegation pool does not earn rewards (the validator's rates are held constant).  The stake in its delegation pool will enter an unbonding period at the time the validator becomes inactive.  Inactive validators have three possible state transitions:

1. they can become active again, if new delegations boost its weight back into the top N;
2. they can be tombstoned, if evidence of misbehavior arises during the unbonding period;
3. they can be disabled, if the operator chooses.

If (2) occurs, the same state transitions as in regular tombstoning occur: all pending undelegations are cancelled, etc.
If (3) occurs, the unbonding period continues and the validator enters the disabled state.
If (1) occurs, the validator stops unbonding, and all delegations are no longer subject to unbonding restrictions.
