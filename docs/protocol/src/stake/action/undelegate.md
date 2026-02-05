# Undelegate

The `Undelegate` action withdraws stake from a validator's delegation pool, converting delegation tokens into unbonding tokens.

This is the first step of the two-step undelegation process. After the unbonding period elapses, an [UndelegateClaim](./undelegate_claim.md) action converts unbonding tokens to staking tokens.

## Action Structure

An `Undelegate` action contains:

| Field | Type | Description |
|-------|------|-------------|
| `validator_identity` | `IdentityKey` | The identity key of the validator to undelegate from |
| `from_epoch` | `Epoch` | The epoch in which the undelegation was prepared |
| `unbonded_amount` | `Amount` | The amount of unbonding tokens produced |
| `delegation_amount` | `Amount` | The amount of delegation tokens consumed |

## Balance Effect

The action:
- **Consumes** `delegation_amount` of delegation tokens (`udelegation_<validator_identity>`)
- **Produces** `unbonded_amount` of unbonding tokens (`uunbonding_start_at_<start_height>_<validator_identity>`)

The unbonding token asset ID encodes both the validator identity and the block height at which unbonding started (the start of the `from_epoch`).

## Validation Rules

### Stateless Checks

None.

### Stateful Checks

1. **Epoch match**: The `from_epoch` must match the current epoch. This ensures the exchange rate used to compute `unbonded_amount` is still valid.

2. **Correct unbonded amount**: The `unbonded_amount` must equal the expected amount computed from `delegation_amount` and the validator's current exchange rate:
   ```
   unbonded_amount = delegation_amount * exchange_rate
   ```
   Rounding is applied during this calculation.

3. **Validator exists**: The validator identity must correspond to a known validator with rate data.

## Execution

When executed:

1. The unbonding token denomination is registered in the asset registry
2. The undelegation is queued for processing at the next epoch boundary

At the epoch boundary:
1. The delegation tokens are burned
2. The unbonding tokens are minted
3. The validator's delegation pool size is decreased
4. The validator's voting power is recalculated

## Unbonding Tokens

Unlike delegation tokens (which are fungible for a given validator), unbonding tokens are unique to both a validator and an unbonding start height:

```
uunbonding_start_at_12345_penumbravalid1abc...
```

This design allows the chain to track when each undelegation started, so the correct unbonding period and accumulated penalties can be applied when the tokens are claimed.

## Privacy

Undelegation reveals:
- The validator identity
- The delegation amount being undelegated
- The epoch at which undelegation occurred

This information is necessary on-chain because:
- The validator identity determines which unbonding token is minted
- The amounts must be verified against the exchange rate
- The epoch determines the unbonding token denomination

The subsequent `UndelegateClaim` action uses a zero-knowledge proof to hide the final unbonded amount.
