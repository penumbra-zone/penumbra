---
name: Testnet upgrade
about: Checklist for shipping an upgrade to a testnet
title: Release Testnet X, via chain upgrade
labels: ''
assignees: ''

---

# Testnet upgrade

Testnet chain id: X
Release date: X
Testnet release manager: X

# Testnet Release Manager Checklist

Pre-release:

- [ ] Review all merged PRs with tags [`consensus-breaking` and/or `state-breaking`](https://github.com/penumbra-zone/penumbra/pulls?q=is%3Apr+label%3Astate-breaking%2Cconsensus-breaking+created%3A%3E%3D2024-01-01)
- [ ] Enumerate the specific PRs that should be included in the release:
  - [ ] https://github.com/penumbra-zone/penumbra/pull/xxxx
  - [ ] https://github.com/penumbra-zone/penumbra/pull/xxxx
- [ ] Confirm migrations exist where required for breaking changes already merged
- [ ] Identify specific test cases for before/after behavior, querying chain state, to ensure that migrations are effective
- [ ] Prepare `upgrade-plan` governance proposal
  - [ ] Decide on block height at which to halt
  - [ ] Double-check you've got bonded stake
  - [ ] Submit proposal
  - [ ] Vote for it, notify validators of same
  - [ ] Confirm proposal passed (default is 24h voting period)

On release day:

- [ ] Draft an announcement for peer review to ensure changes included are comprehensive.
- [ ] Disable testnet deploy workflow, so that chain is not reset
- [ ] Bump the version number and push its tag, via [cargo-release](https://crates.io/crates/cargo-release).
    - [ ] Run `cargo release minor` for a new testnet, or `cargo release patch` for a bugfix. For the latter, make sure you're on a dedicated release branch.
    - [ ] Push the commit and newly generated tag, e.g. `v0.51.0`, to the remote.
- [ ] Manually trigger container-build workflow, bc deploy workflow is disabled
- [ ] Wait for the ["Release" workflow](https://github.com/penumbra-zone/penumbra/actions/workflows/release.yml) to complete
- [ ] Edit the newly created release object, and add a note summarizing the intent of the release
- [ ] Close faucet (chain halt will make it inoperative anyway)
- [ ] Run migrations on all validators
- [ ] Run migrations on all fullnodes
- [ ] Update Galileo deployment, [following docs](https://github.com/penumbra-zone/galileo)
- [ ] Make the announcement to Discord! 🎉🎉🎉

Post-release cleanup tasks

- [ ] Ensure faucet is open
- [ ] Perform Hermes maintenance for [genesis restart](https://hermes.informal.systems/advanced/troubleshooting/genesis-restart.html#updating-a-client-after-a-genesis-restart-without-ibc-upgrade-proposal) to get relayer running again
- [ ] Confirm IBC channels are working
