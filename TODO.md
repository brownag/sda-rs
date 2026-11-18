# TODO

## Goal
Bump the package patch version and prepare optional Cargo author metadata.

## Tasks

### 1. Package metadata
- [x] Bump `sda-rs` from `0.1.0` to `0.1.1` in `Cargo.toml` and update the root package entry in `Cargo.lock`.
- [x] Add the requested author string to package metadata.
- [x] `cargo check` and `git diff --check` passed; ready for a focused local commit.

## Notes
- Branch: `feature/sda-readiness`.
- Planning artifacts are tracked and will be committed separately from implementation.
- No remote Git actions.
- Requested author string: `Andrew Gene Brown <brown.andrewg@gmail.com>`.
