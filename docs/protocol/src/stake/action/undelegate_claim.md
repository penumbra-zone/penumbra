# Undelegate Claim Descriptions

Each undelegate claim contains a UndelegateClaimBody and a zk-SNARK undelegate claim proof.

# Invariants

#### Local Invariants

1. You cannot claim undelegations that have not finishing unbonding.

2. Slashing penalties must be applied when unbonding.

#### Local Justification

1. In the `ActionHandler` for `check_stateful` we check that the undelegations have finished unbonding.

2. The `ConvertCircuit` verifies that the conversion from the unbonding token to the staking token was done using the correct conversion rate calculated from the penalty. We check in the `ActionHandler` for `check_stateful` that the _correct_ penalty rate was used.

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

### Balance Commitment Integrity

The zk-SNARK certifies that the public input balance commitment $cv$ was derived from the witnessed values as:

$cv = v_e + [\widetilde{v}] G_{\widetilde{v}}$

where $G_{\widetilde{v}}$ is a constant generator.

$v_e$ is the expected balance computed from the public conversion rate $p$ and the input
amount $v_i$:

$v_e = [-v_i] G_{v_i} + [p * v_i] G_{v_t}$

where $G_{v_i}$ is the asset-specific generator corresponding to the input
token with asset ID $ID_i$ and $G_{v_t}$ is the asset-specific generator corresponding to the
target token with asset ID $ID_t$. Both these asset-specific bases are derived in-circuit as described in [Assets and Values](../../assets.md).

## Undelegate Claim

The undelegate claim proof uses the convert circuit statements above where:

* The input amount $v_i$ is set to the unbonding amount
* The rate is set to the Penalty $p$
* Asset `ID` $ID_i$ is the unbonding token asset ID
* Asset `ID` $ID_t$ is the staking token asset ID
