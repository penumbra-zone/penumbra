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
Testnet release manager: X

# Testnet Release Manager Checklist

Monday (week before release):

- [ ] Create GitHub project column, work with team to populate the milestone with tickets targeted for the release.

Thursday:

- [ ] Check in with team again in a release meeting and update the GitHub milestone to ensure it represents what will make it into the testnet.

Monday (release day):

- [ ] Update `discord_history.csv` file using the Galileo bot (`cd galileo && cargo run --release history
  --channel https://discord.com/channels/824484045370818580/915710851917439060 >
  ../penumbra/testnets/discord_history.csv`, assuming that the galileo and penumbra repos are
  sitting side-by-side in the file system)
- [ ] Create new testnet directory with initial genesis allocations for this testnet (make sure all
  current team members and the Galileo bot have some assets in the `base_allocations.csv` file!) by
  running `cd testnets && ./new-testnet.sh`
- [ ] Check for any [tendermint updates](https://github.com/tendermint/tendermint/releases) and update the Dockerfiles, documentation, and relay deployments with the latest desired version
- [ ] Update the User Guide to mention the git tag
- [ ] Confirm that all tests pass following any final updates to `main`, and select the commit to tag for the new testnet.
- [ ] Create new git tag e.g. `006-orthosie` on `main` (tags created on any other branches will not transfer when merged in) and push to shared remote: `git tag -a <tag_name>` - **must be annotated tag, i.e. `git tag -a`** for Vergen build. This will create a `Waiting` GitHub Action for deployment.
- [ ] You must [manually review](https://docs.github.com/en/actions/managing-workflow-runs/reviewing-deployments) the `Waiting` deployment in the GitHub Action UI before the deployment will begin. Monitor the GitHub action to ensure it completes after it is approved.
- [ ] Delegate to the Penumbra Labs CI validator
- [ ] Update Galileo to run against the correct tag: change [the dependencies in the Cargo.toml](https://github.com/penumbra-zone/galileo/blob/main/Cargo.toml#L11) to reference the new git tag and commit to `main`.
- [ ] `ssh root@galileo.penumbra.zone`and bounce Galileo via the following steps:
  - [ ] `git checkout` latest tag in `~/penumbra`
  - [ ] `cargo run --release --bin pcli view reset` in `~/penumbra` to reset the client state for the new testnet
  - [ ] `killall galileo` to stop the running galileo process
  - [ ] `git pull origin main` in `~/galileo` to update the checkout
  - [ ] `screen -r` to start a screen session
  - [ ] start Galileo again: `RUST_LOG=galileo=info DISCORD_TOKEN={token} cargo run --release -- serve 100penumbra 10pizza 10gm 10gn 1cube --catch-up https://discord.com/channels/824484045370818580/915710851917439060/968238177229897789`
  - [ ] exit `screen` (`^A d`) without stopping `galileo`
  - [ ] **confirm that Galileo is dispensing tokens** by testing the faucet channel with your own address
  - [ ] resupply Galileo wallet as needed
- [ ] Make GitHub release object and draft an announcement for peer review to ensure major changes included are comprehensive.
- [ ] Make the announcement to Discord! 🎉🎉🎉
