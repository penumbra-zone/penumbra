# Validators

Validators in Penumbra undergo various transitions depending on chain activity.

```
                                       ┌ ─ ─ ─ ─ ┐             
                                         Genesis               
                                       │Validator│             
                                        ─ ─ ─ ─ ─              
                                            │                  
                                            │                  
┌ ─ ─ ─ ─ ─ ─ ─ ─                           ▼                  
    Validator    │      ┏━━━━━━━━┓      ╔══════╗      ┏━━━━━━━┓
│   Definition    ─────▶┃Inactive┃─────▶║Active║─────▶┃Slashed┃
 (in transaction)│      ┗━━━━━━━━┛      ╚══════╝      ┗━━━━━━━┛
└ ─ ─ ─ ─ ─ ─ ─ ─            ▲              ▲             ▲    
                             │              │             │    
                             │              ▼             │    
                             │         ╔═════════╗        │    
                             └─────────║Unbonding║────────┘    
                                       ╚═════════╝             
```

Single lines represent unbonded stake, and double lines represent bonded stake.

Validators become known to the chain either at genesis, or by means of a transaction with a `ValidatorDefinition` action in them. Validators transition through four states:

* **Inactive**, where the validator is not part of the consensus set, the stake in the validator's delegation pool is not bonded;
* **Active**, where the validator is part of the consensus set, and the stake in the validator's delegation pool is bonded;
* **Unbonding**, where the validator is not part of the consensus set, but the stake in the validator's delegation pool is still bonded;
* **Slashed**, where the validator is not part of the consensus set, and the stake in the validator's delegation pool is not bonded.

Validators specified in the genesis config begin in the active state, with whatever stake was allocated to their delegation pool at genesis. Otherwise, validators begin in the inactive state, with no stake in their delegation pool.  At this point, the validator is known to the chain, and stake can be contributed to its delegation pool.  Stake contributed to an inactive validator's delegation pool does not earn rewards (the validator's rates are held constant), but it is also not bonded, so undelegations are effective immediately, with no unbonding period and no output quarantine.

The chain chooses a validator limit N as a consensus parameter. When a validator's delegation pool (a) has a nonzero balance and (b) its (voting-power-adjusted) size is in the top N validators, it moves into the active state during the next epoch transition.  Active validators participate in consensus, and are communicated to Tendermint. Stake contributed to an active validator's delegation pool earns rewards (the validator's rates are updated at each epoch to track the rewards accruing to the pool). That stake is bonded, so undelegations have an unbonding period and an output quarantine. An active validator can exit the consensus set in two ways.

First, the validator could be slashed.  This can happen in any block, not just at an epoch transition.  Slashed validators are immediately removed from the consensus set. Any pending undelegations from a slashed validator are cancelled: the quarantined output notes are deleted, and the quarantined nullifiers are removed from the nullifier set.  The validator's rates are updated to price in the slashing penalty, and are then held constant. Slashed validators are jailed, and permanently prohibited from participation in consensus (though their operators can create new identity keys, if they'd like to). Stake cannot be delegated to a slashed validator. Stake already contributed to a slashed validator's delegation pool is not bonded (the validator has already been slashed and jailed), so undelegations are effective immediately, with no unbonding period and no quarantine.

Second, the validator could be displaced from the validator set by another validator with more stake in its delegation pool. The validator is then in the unbonding state.  It does not participate in consensus, and the stake in its delegation pool does not earn rewards (the validator's rates are held constant).  However, the stake in its delegation pool is still bonded.  Undelegations from an unbonding validator are quarantined with an unbonding period that starts when the undelegation was performed, *not* when the validator began unbonding.  Unbonding validators have three possible state transitions:

1. they can become active again, if new delegations boost its weight back into the top N;
2. they can be slashed, if evidence of misbehavior arises during the unbending period;
3. they can become inactive, if neither (1) nor (2) occurs before the unbonding period passes.

If (2) occurs, the same state transitions as in regular slashing occur: all pending undelegations are cancelled, etc.
If (3) occurs, all pending undelegations are immediately removed from quarantine, short-circuiting the unbonding period that began when the undelegation was performed.  If (1) occurs, the validator stops unbonding, but this has no effect on pending undelegations, since they were quarantined with an unbonding period that started when the undelegation was performed (i.e., as if they were undelegations from an active validator).