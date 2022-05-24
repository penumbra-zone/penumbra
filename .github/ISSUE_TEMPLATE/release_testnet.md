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

Thursday:
- [ ] Check in with team again in a release meeting and update the GitHub milestone to ensure it represents what will make it into the testnet.

Friday: Prepare a PR (for merge on Monday) with the following:
- [ ] Update `discord-addresses` file using the Galileo bot (`cargo run --release history --channel <channel>`)
- [ ] Update `testnet/<moon>/allocations.csv` file with the above Discord faucet addresses
- [ ] Update the User Guide to mention the git tag

Monday (release day):
- [ ] Create new git tag e.g. `006-orthosie`, push to shared remote: `git tag -a <tag_name>` - must be annotated tag for Vergen build. This will begin the release process. Monitor the GitHub action to ensure it completes.
- [ ] Update peer configuration on `testnet.penumbra.zone`: in Tendermint's `config.toml`, update the `bootstrap-peers` and `persistent-peers` fields to contain the IPs of the Penumbra-operated full nodes.
- [ ] On the peers themselves, stop `pd` and `tendermint`, then update `pd` from latest `main` (or the tag), clear rocksdb and existing tendermint state, and copy the new `genesis.json` file in place. Update the `config.toml` to point to `testnet.penumbra.zone` as described [here](https://guide.penumbra.zone/main/pd/join-testnet/fullnode.html). Then start `pd` and `tendermint`. Verify that they both sync with the running testnet.
- [ ] Bounce galileo: git checkout latest tag, stop galileo, `pcli wallet reset`, start Galileo.
- [ ] Make GitHub release object.
- [ ] Make an announcement to Discord 🎉
