# Governance

Like the Cosmos Hub, Penumbra supports on-chain governance with delegated
voting.  Unlike the Cosmos Hub, Penumbra's governance mechanism supports
secret ballots.  Penumbra users can anonymously propose votes by escrowing a
deposit of bonded stake.  Stakeholders vote by proving ownership of their
bonded stake prior to the beginning of the voting period and encrypting their
votes to a threshold key controlled by the validator set.  Validators sum
encrypted votes and decrypt only the per-epoch totals.

## TODO

- [ ] emergency vs normal proposals, where emergency proposals are approved as soon as 2/3 validators vote yes
- [ ] is "liquid democracy"-style voting possible? currently the only supported delegation is to validators, but it could be useful to support delegation to third parties.  one way to do this would be to allow the proxy voter to create votes on behalf of their voting delegators, but this would require that the proxy spend and create their notes, which is a huge power.  this could be minimized by creating a special "voting key" capability, but nothing ensures that the proxy voter actually creates valid notes when spending them, so they could still destroy their voting delegators' power.  another 