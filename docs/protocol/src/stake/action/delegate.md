# Delegate

The `Delegate` action adds stake to a validator's delegation pool, converting staking tokens into delegation tokens.

## Action Structure

A `Delegate` action contains:

| Field | Type | Description |
|-------|------|-------------|
| `validator_identity` | `IdentityKey` | The identity key of the validator to delegate to |
| `epoch_index` | `u64` | The epoch in which the delegation was prepared |
| `unbonded_amount` | `Amount` | The amount of staking tokens to delegate |
| `delegation_amount` | `Amount` | The amount of delegation tokens produced |

## Balance Effect

The action:
- **Consumes** `unbonded_amount` of staking tokens (`upenumbra`)
- **Produces** `delegation_amount` of delegation tokens (`udelegation_<validator_identity>`)

## Validation Rules

### Stateless Checks

None.

### Stateful Checks

1. **Epoch match**: The `epoch_index` must match the current epoch. This ensures the exchange rate used to compute `delegation_amount` is still valid.

2. **Correct delegation amount**: The `delegation_amount` must equal the expected amount computed from `unbonded_amount` and the validator's current exchange rate:
   ```
   delegation_amount = unbonded_amount / exchange_rate
   ```
   Rounding is applied during this calculation.

3. **Validator enabled**: The validator definition must have `enabled: true`.

4. **Valid validator state**: The validator must be in one of these states: `Defined`, `Inactive`, or `Active`. Delegations to `Jailed`, `Tombstoned`, or `Disabled` validators are rejected.

5. **Minimum stake for Defined validators**: If the validator is in the `Defined` state with an empty delegation pool, the delegation must meet the `min_validator_stake` threshold. This ensures new validators start with sufficient stake.

## Execution

When executed, the delegation is queued to take effect at the next epoch boundary. At that time:

1. The delegation tokens are minted to the delegator
2. The validator's delegation pool size is increased
3. The validator's voting power is recalculated
4. If the validator was in `Defined` state and now meets the minimum stake threshold, it transitions to `Inactive`

## Exchange Rate

The exchange rate between staking tokens and delegation tokens varies over time as the validator earns rewards:

- At genesis, the exchange rate is 1:1
- Each epoch, if the validator is active, the rate increases by the validator's reward rate
- The reward rate is the base chain reward rate minus the validator's commission

A higher exchange rate means more staking tokens are needed to purchase the same amount of delegation tokens. This reflects the accumulated rewards in the delegation pool.

## Privacy

Currently, delegation amounts are revealed on-chain. The struct includes a TODO to add flow encryption to hide the `unbonded_amount`.
