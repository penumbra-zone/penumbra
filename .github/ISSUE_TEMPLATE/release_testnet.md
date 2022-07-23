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
- [ ] Confirm that all tests pass following any final updates to `main`, and select the commit to tag for the new testnet.
- [ ] Create new git tag e.g. `006-orthosie` on `main` (tags created on any other branches will not transfer when merged in) and push to shared remote: `git tag -a <tag_name>` - must be annotated tag for Vergen build. This will create a `Waiting` GitHub Action for deployment.
- [ ] You must [manually review](https://docs.github.com/en/actions/managing-workflow-runs/reviewing-deployments) the `Waiting` deployment in the GitHub Action UI before the deployment will begin. Monitor the GitHub action to ensure it completes after it is approved.
- [ ] Update Galileo to run against the correct tag: change [the dependencies in the Cargo.toml](https://github.com/penumbra-zone/galileo/blob/main/Cargo.toml#L11) to reference the new git tag and commit to `main`.
- [ ] `ssh root@galileo.penumbra.zone`and bounce Galileo via the following steps:
  - [ ] `git checkout` latest tag in `~/penumbra`
  - [ ] `tmux attach` and navigate to the session running galileo
  - [ ] stop the running galileo process
  - [ ] `git pull origin main` in `~/galileo` to update the checkout
  - [ ] `cargo run --release --bin pcli -- wallet reset` to reset the client state for the new testnet
  - [ ] start Galileo again: `RUST_LOG=galileo=info DISCORD_TOKEN={token} cargo run --release -- serve 100penumbra  --catch-up https://discord.com/channels/824484045370818580/915710851917439060/968238177229897789`
  - [ ] Confirm that Galileo is dispensing tokens, resupply Galileo wallet as needed.
- [ ] Make GitHub release object and draft an announcement for peer review to ensure major changes included are comprehensive.
- [ ] Make the announcement to Discord! 🎉🎉🎉
