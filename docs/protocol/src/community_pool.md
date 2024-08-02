# Community Pool

As with the Cosmos SDK, Penumbra also has a similar Community Pool feature.

## Making A Community Pool Spend Transaction Plan

Token holders can submit a governance community pool spend proposal. This proposal contains a _transaction plan_ containing a description of the spends to be performed if the proposal passes. This is described fully in the [governance section of the Penumbra protocol spec](./governance.md).

## Contributing To The Community Pool

Anyone can contribute any amount of any denomination to the Penumbra Community Pool. To do this, use the
command `pcli tx community-pool-deposit`, like so:

```bash
pcli tx community-pool-deposit 100penumbra
```

Funds contributed to the Community Pool cannot be withdrawn except by a successful Community Pool spend governance
proposal.

To query the current Community Pool balance, use `pcli query community-pool balance` with the **base denomination** of an
asset or its asset ID (display denominations are not currently accepted). For example:

```bash
pcli query community-pool balance upenumbra
```

Community Pool spend proposals are only accepted for voting if they would not overdraw the current funds in the
Community Pool at the time the proposal is submitted, so it's worth checking this information before submitting
such a proposal.

### Sending Validator Funding Streams To The Community Pool

A validator may non-custodially send funds to the Community Pool, similarly to any other funding stream. To do
this, add a `[[funding_stream]]` section to your validator definition TOML file that declares the
Community Pool as a recipient for a funding stream. For example, your definition might look like this:

```toml
sequence_number = 0
enabled = true
name = "My Validator"
website = "https://example.com"
description = "An example validator"
identity_key = "penumbravalid1s6kgjgnphs99udwvyplwceh7phwt95dyn849je0jl0nptw78lcqqvcd65j"
governance_key = "penumbragovern1s6kgjgnphs99udwvyplwceh7phwt95dyn849je0jl0nptw78lcqqhknap5"

[consensus_key]
type = "tendermint/PubKeyEd25519"
value = "tDk3/k8zjEyDQjQC1jUyv8nJ1cC1B/MgrDzeWvBTGDM="

# Send a 1% commission to this address:
[[funding_stream]]
recipient = "penumbrav2t1hum845ches70c8kp8zfx7nerjwfe653hxsrpgwepwtspcp4jy6ytnxhe5kwn56sku684x6zzqcwp5ycrkee5mmg9kdl3jkr5lqn2xq3kqxvp4d7gwqdue5jznk2ter2t66mk4n"
rate_bps = 100

# Send another 1% commission to the Community Pool:
[[funding_stream]]
recipient = "CommunityPool"
rate_bps = 100
```
