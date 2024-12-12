# Undelegate Claim Descriptions

Each undelegate claim contains a UndelegateClaimBody and a zk-SNARK undelegate claim proof.

## [UndelegateClaim](#undelegateclaim-body)

The body of a `UndelegateClaim` has four parts:

1. The validator `IdentityKey` being undelegated from;
2. The penalty to apply to the undelegation;
3. The balance contribution which commits to the value balance of the action;
4. The height at which unbonding started.

# Invariants

#### Local Invariants

1. You cannot claim undelegations that have not finished unbonding.

2. Slashing penalties must be applied when unbonding.

3. The UndelegateClaim reveals the validator identity, but not the unbonding amount.

4. The balance contribution of the value of the undelegation is private.

#### Local Justification

1. In the `ActionHandler` we check that the undelegations have finished unbonding.

2. The `ConvertCircuit` verifies that the conversion from the unbonding token to the staking token was done using the correct conversion rate calculated from the penalty. We check in the `ActionHandler` that the _correct_ penalty rate was used.

3. The `UndelegateClaim` performs the above [conversion check in 2 in zero-knowledge](#balance-commitment-integrity) using the private unbonding amount.

4. The balance contribution of the value of the undelegation is hidden via the hiding property of the balance commitment scheme. Knowledge of the opening of the [balance commitment is done in zero-knowledge](#balance-commitment-integrity).

#### Global Justification

1.1. This action consumes the amount of the unbonding tokens and contributes the unbonded amount of the staking tokens to the transaction's value balance. Value is not created due to [system level invariant 1](../../transactions/invariants.md), which ensures that transactions contribute a 0 value balance.

# zk-SNARK Statements

The undelegate claim proof is implemented as an instance of a generic convert circuit which converts a private amount of one input asset into a target asset, given a public conversion rate.

First we describe the convert circuit, and then the undelegate claim proof.

## Convert zk-SNARK Statements

The convert circuit demonstrates the properties enumerated below for the private witnesses known by the prover:

* Input amount $v_i$ interpreted as an $\mathbb F_q$ and constrained to fit in 128 bits
* Balance blinding factor $\widetilde{v} \isin \mathbb F_r$ used to blind the balance commitment

And the corresponding public inputs:

* Balance commitment $cv \isin G$ to the value balance
* Rate $p$, a 128-bit fixed point value, represented in circuit as four 64-bit (Boolean constraint) limbs
* Asset ID $ID_i \isin \mathbb F_q$ of the input (source) amount
* Asset ID $ID_t \isin \mathbb F_q$ of the target amount

### [Balance Commitment Integrity](#balance-commitment-integrity)

The zk-SNARK certifies that the public input balance commitment $cv$ was derived from the witnessed values as:

$cv = [-v_i] G_{v_i} + [v_e] G_{v_t} + [\widetilde{v}] G_{\widetilde{v}}$

where:
* $G_{\widetilde{v}}$ is a constant generator
* $v_e$ is the expected balance computed from the public conversion rate $p$ and the input
amount $v_i$, i.e. $v_e = p \cdot v_i$
* $G_{v_i}$ is the asset-specific generator corresponding to the input
token with asset ID $ID_i$
* $G_{v_t}$ is the asset-specific generator corresponding to the
target token with asset ID $ID_t$.

Both these asset-specific bases $G_{v_t}$ and $G_{v_i}$ are derived in-circuit as described in [Assets and Values](../../assets.md).

## Undelegate Claim

The undelegate claim proof uses the convert circuit statements above where:

* The input amount $v_i$ is set to the unbonding amount
* The rate is set to the Penalty $p$
* Asset `ID` $ID_i$ is the unbonding token asset ID
* Asset `ID` $ID_t$ is the staking token asset ID
