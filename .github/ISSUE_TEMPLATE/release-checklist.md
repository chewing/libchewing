---
name: Release checklist
about: Prepare a release for libchewing
title: ''
labels: ''
assignees: ''

---

- [ ] Review issues list
- [ ] Make sure NEWS is updated
- [ ] Update version numbers in
  - [ ] CMakeLists.txt
  - [ ] Cargo.toml, */Cargo.toml
- [ ] Update Cargo.lock
- [ ] Prepare pre-release/release tarball
  ```sh
  cmake --preset rust
  cmake --build build -t package_source
  ```
  - [ ] Verify the content of the tarball
- [ ] Git tag and sign tag
- [ ] cargo publish
- [ ] Tag new release on GitHub
- [ ] Update website
- [ ] Send announcement
