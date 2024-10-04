---
name: Release checklist
about: '[for maintainer use]'
title: 'Release 0.3.1'
labels: ''
assignees: 'joaander'

---

- [ ] Run *bumpversion*.
- [ ] Run `cargo update`
- [ ] Run `cargo bundle-licenses --format yaml --output THIRDPARTY.yaml`
- [ ] Check for new or duplicate contributors since the last release:
  `comm -13 (git log $(git describe --tags --abbrev=0) --format="%aN <%aE>" | sort | uniq | psub) (git log --format="%aN <%aE>" | sort | uniq | psub)`.
  Add entries to `.mailmap` to remove duplicates.
- [ ] Add release date and highlights to release notes.
- [ ] Check readthedocs build, especially release notes formatting.
- [ ] Tag and push.
- [ ] Update conda-forge recipe.
