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
- [ ] Update Galileo deployment, [following docs](https://github.com/penumbra-zone/galileo).
- [ ] Make GitHub release object and include the announcement
- [ ] Make the announcement to Discord! 🎉🎉🎉
