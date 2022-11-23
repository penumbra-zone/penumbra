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

Preceding Friday (sprint planning day):

- [ ] Create GitHub project column, work with team to populate the milestone with tickets targeted for the release.

Tuesday (or after release of previous testnet):

- [ ] Construct the genesis data for the release:
  - [ ] Update `discord_history.csv` file using the Galileo bot (`cd galileo && cargo run --release history --channel https://discord.com/channels/824484045370818580/915710851917439060 > ../penumbra/testnets/discord_history.csv`, assuming that the galileo and penumbra repos are sitting side-by-side in the file system)
  - [ ] Create new testnet directory with initial genesis allocations for this testnet by running `cd testnets && ./new-testnet.sh`
  - This genesis data will be used for `testnet-preview` with a randomized version of the future testnet's chain ID.

Thursday:

- [ ] Check in with team again in a release meeting and update the GitHub milestone to ensure it represents what will make it into the testnet.
- [ ] Draft an announcement for peer review to ensure major changes included are comprehensive.

Following Monday (release day):

- [ ] Verify that `testnet-preview.penumbra.zone` is operational; it is redeployed on every push to main, and is an exact preview of what is about to be deployed.
- [ ] Create new git tag e.g. `006-orthosie` on `main` (tags created on any other branches will not transfer when merged in) and push to shared remote: `git tag -a <tag_name>` - **must be annotated tag, i.e. `git tag -a`** for Vergen build. This will create a `Waiting` GitHub Action for deployment.
- [ ] You must [manually review](https://docs.github.com/en/actions/managing-workflow-runs/reviewing-deployments) the `Waiting` deployment in the GitHub Action UI before the deployment will begin. Monitor the GitHub action to ensure it completes after it is approved.
- [ ] Update the User Guide to mention the newly created git tag.
- [ ] Delegate to the Penumbra Labs CI validator
- [ ] Update and redeploy Galileo:
  - [ ] Change it to run against the correct tag: change [the dependencies in the Cargo.toml](https://github.com/penumbra-zone/galileo/blob/main/Cargo.toml#L11) to reference the new git tag and commit to `main`.
  - [ ] `ssh root@galileo.penumbra.zone`and bounce Galileo via the following steps:
    - [ ] `git checkout` latest tag in `~/penumbra`
    - [ ] `cargo run --release --bin pcli view reset` in `~/penumbra` to reset the client state for the new testnet
    - [ ] `killall galileo` to stop the running galileo process
    - [ ] `git pull origin main` in `~/galileo` to update the checkout
    - [ ] `screen -r` to start a screen session
    - [ ] start Galileo again: `RUST_LOG=galileo=info DISCORD_TOKEN={token} cargo run --release -- serve 100penumbra 10pizza 10gm 10gn --catch-up {URL of latest unserved request}`
    - [ ] exit `screen` (`^A d`) without stopping `galileo`
    - [ ] **confirm that Galileo is dispensing tokens** by testing the faucet channel with your own address
    - [ ] resupply Galileo wallet as needed
- [ ] Make GitHub release object and include the announcement
- [ ] Make the announcement to Discord! 🎉🎉🎉
