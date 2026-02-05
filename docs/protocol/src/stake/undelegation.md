# Undelegation

The undelegation process unbonds stake from a validator's delegation pool. Unlike delegation (which is a single action), undelegation is a two-step process:

1. **Undelegate**: Burns delegation tokens and mints unbonding tokens
2. **UndelegateClaim**: After the unbonding period, burns unbonding tokens and produces staking tokens

This two-step design ensures that delegators cannot escape slashing by racing to undelegate before misbehavior evidence is processed.

## Unbonding Tokens

When a user undelegates, they receive **unbonding tokens** instead of staking tokens directly. Unbonding tokens are unique per validator and per epoch:

```
uunbonding_start_at_<start_height>_<validator_identity>
```

For example: `uunbonding_start_at_12345_penumbravalid1abc...`

The unbonding token encodes:
- The validator identity being undelegated from
- The block height at which unbonding started (the start of the epoch when the `Undelegate` action was processed)

This allows the chain to track the unbonding period and apply any penalties that occur during it.

## Unbonding Period

The length of the unbonding period is determined by the validator's bonding state:

- **Bonded validators**: Unbonding takes `unbonding_delay` blocks (a chain parameter)
- **Unbonding validators**: Unbonding completes when the validator's unbonding completes
- **Unbonded validators**: No unbonding period; tokens can be claimed immediately

During the unbonding period, if the validator is slashed, a penalty is recorded. This penalty will be applied when the unbonding tokens are claimed.

## Claiming Unbonded Stake

After the unbonding period elapses, the user submits an `UndelegateClaim` action to convert their unbonding tokens to staking tokens.

The claim:
1. Burns the unbonding tokens
2. Computes the accumulated penalty over the unbonding window
3. Produces staking tokens equal to the unbonded amount minus the penalty
4. Uses a zero-knowledge proof to hide the unbonding amount while proving correct penalty application

See [Undelegate Claim](./action/undelegate_claim.md) for the detailed specification.

## Penalty Accumulation

Penalties are stored per-validator per-epoch. When claiming, the total penalty is computed by compounding all penalties from the unbonding start epoch through the current epoch:

$$\text{total\_penalty} = 1 - \prod_{e=\text{start}}^{\text{current}} (1 - p_e)$$

where $p_e$ is the penalty applied in epoch $e$.

For example, if a validator was slashed 1% in epoch 10 and 2% in epoch 15, a claim covering epochs 5-20 would apply a combined penalty of approximately 2.98%:

$$1 - (1 - 0.01)(1 - 0.02) = 1 - (0.99)(0.98) \approx 0.0298$$

## Privacy Considerations

Unlike delegation (where the amount can be hidden), undelegation reveals:
- The validator identity
- The amount of delegation tokens being unbonded
- The epoch at which unbonding started

This information is visible on-chain because the unbonding period calculation requires knowing when unbonding started.

However, the `UndelegateClaim` action hides the final unbonded amount using a zero-knowledge proof. An observer can see that someone claimed unbonding tokens from a validator, but cannot determine the exact amount received.
