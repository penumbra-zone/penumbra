---
name: Point release
about: Checklist for preparing a point release for tooling
title: Release vX.Y.Z, via point-release
labels: ''
assignees: ''
---

# Tooling Release

<!--
Explain the rationale for this release: did a particular bugfix land that we want to ship quickly?
Is an external consumer blocked on a new RPC?
-->
In order to ship some minor improvements and bug fixes, let's prepare a `vX.Y.Z.` release, flushing out the current contents of the main branch.

## Changes to include
<!--
Explain the rationale for this release: did a particular bugfix land that we want to ship quickly?
Is an external consumer blocked on a new RPC?
-->

- [ ] Everything on current main
- [ ] Feature foo in PR: 
- [ ] Feature bar in PR: 

## Compatibility
As this is a point-release, all changes must be fully compatible for all nodes and clients.
Careful attention should be given to the delta between most recent tag on the main branch:
https://github.com/penumbra-zone/penumbra/compare/v(X.Y.(Z-1)..main
