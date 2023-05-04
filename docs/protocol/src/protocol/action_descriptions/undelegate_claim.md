# Undelegate Claim Descriptions

Each undelegate claim contains a UndelegateClaimBody and a zk-SNARK undelegate claim proof.

## Undelegate Claim zk-SNARK Statements

The undelegate claim proof demonstrates the properties enumerated below for the private witnesses known by the prover:

* Unbonding amount $v_u$ interpreted as an $\mathbb F_q$
* Balance blinding factor $\tilde v \isin \mathbb F_r$ used to blind the balance commitment

And the corresponding public inputs:

* Balance commitment $cv \isin G$ to the value balance
* Penalty $p$ interpreted as an $\mathbb F_q$
* Unbonding asset ID $G_u \isin \mathbb G$

### Balance Commitment Integrity

The zk-SNARK certifies that the public input balance commitment $cv$ was derived from the witnessed values as:

$cv = v_e + [\tilde v] G_{\tilde v}$

where $G_{\tilde v}$ is a constant generator.

$v_e$ is the expected balance computed from the penalty $p$, the unbonding
amount $v_u$ and the unbonding asset ID $G_u$:

$v_e = [-v_u] G_{v_u} + v_p G_{v_s}$

where $G_{v_u}$ is the asset-specific generator corresponding to the unbonding
token and $G_{v_s}$ is the asset-specific generator corresponding to the
staking token. $v_u$ is the unbonding amount, and $v_p$ is the penalized amount,
computed by applying the penalty $p$ to the unbonding amount:

$v_p = v_u * (100,000,000 - p) / 100,000,000$
