---
name: Testnet release
about: Checklist for releasing a testnet
title: ''
labels: ''
assignees: ''

---

# Testnet Release

Testnet name: X
Release date: X
Milestone link: X
Testnet release manager: X

# Testnet Release Manager Checklist

Monday (week before release):
- [ ] Create GitHub milestone, work with team to populate the milestone with tickets targeted for the release.
- [ ] Create new testnet directory with initial genesis allocations for this testnet (make sure all current team members and the Galileo bot have some assets!)
- [ ] Update `discord-addresses` file using the Galileo bot (`cargo run --release history --channel <channel>`)
- [ ] Update `testnet/<moon>/allocations.csv` file with the above Discord faucet addresses

Thursday:
- [ ] Check in with team again in a release meeting and update the GitHub milestone to ensure it represents what will make it into the testnet.

Monday (release day):
- [ ] Check for any [tendermint updates](https://github.com/tendermint/tendermint/releases) and update the Dockerfiles, documentation, and relay deployments with the latest desired version
- [ ] Update the User Guide to mention the git tag
- [ ] Create new git tag e.g. `006-orthosie`, push to shared remote: `git tag -a <tag_name>` - must be annotated tag for Vergen build. This will begin the release process. Monitor the GitHub action to ensure it completes.
- [ ] Update peer configuration on our Penumbra validator running on `testnet.penumbra.zone`: in Tendermint's `config.toml`, update the `bootstrap-peers` and `persistent-peers` fields to contain the IPs of the Penumbra-operated full nodes.
- [ ] On the peers themselves, stop `pd` and `tendermint`, then update `pd` from latest `main` (or the tag), clear rocksdb and existing tendermint state, and copy the new `genesis.json` file in place. Update the `config.toml` to point to `testnet.penumbra.zone` as described [here](https://guide.penumbra.zone/main/pd/join-testnet/fullnode.html). Then start `pd` and `tendermint`. Verify that they both sync with the running testnet.
- [ ] Update Galileo to run against the correct tag: change [the dependencies in the Cargo.toml](https://github.com/penumbra-zone/galileo/blob/main/Cargo.toml#L11) to reference the new git tag and commit to `main`.
- [ ] Bounce galileo: git checkout latest tag in `~/penumbra`, stop galileo, git pull main in `~/galileo`, `pcli wallet reset`, start Galileo.
- [ ] Make GitHub release object.
- [ ] Make an announcement to Discord 🎉
