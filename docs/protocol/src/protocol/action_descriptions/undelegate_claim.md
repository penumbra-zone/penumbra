# Undelegate Claim Descriptions

Each undelegate claim contains a UndelegateClaimBody and a zk-SNARK undelegate claim proof. The undelegate claim proof is implemented as an instance of a generic convert circuit which converts a private amount of one input asset into a target asset, given a public conversion rate.

First we describe the convert circuit, and then the undelegate claim proof.

## Convert zk-SNARK Statements

The convert circuit demonstrates the properties enumerated below for the private witnesses known by the prover:

* Input amount $v_i$ interpreted as an $\mathbb F_q$
* Balance blinding factor $\widetilde{v} \isin \mathbb F_r$ used to blind the balance commitment

And the corresponding public inputs:

* Balance commitment $cv \isin G$ to the value balance
* Rate $p$, a 128-bit fixed point value, represented in circuit as four 64-bit (Boolean constraint) limbs
* Asset ID $ID_i \isin \mathbb G$ of the input (source) amount
* Asset ID $ID_t \isin \mathbb G$ of the target amount

### Balance Commitment Integrity

The zk-SNARK certifies that the public input balance commitment $cv$ was derived from the witnessed values as:

$cv = v_e + [\widetilde{v}] G_{\widetilde{v}}$

where $G_{\widetilde{v}}$ is a constant generator.

$v_e$ is the expected balance computed from the public conversion rate $p$ and the input
amount $v_i$:

$v_e = [-v_i] G_{v_i} + [p * v_i] G_{v_t}$

where $G_{v_i}$ is the asset-specific generator corresponding to the input
token with asset ID $ID_i$ and $G_{v_t}$ is the asset-specific generator corresponding to the
target token with asset ID $ID_t$. Both these asset-specific bases are derived in-circuit as described in [Value Commitments](../../protocol/value_commitments.md).

## Undelegate Claim

The undelegate claim proof uses the convert circuit statements above where:

* The input amount $v_i$ is set to the unbonding amount
* The rate is set to the Penalty $p$
* Asset `ID` $G_i$ is the unbonding token asset ID
* Asset `ID` $G_t$ is the staking token asset ID
