# Orchestration State 1

Plan: `planning/EXECUTION_PLAN_1.md`

## Baseline checklist

- [ ] `cargo fmt --check` — not runnable before crate scaffold; run after task 1.1.1.
- [ ] `cargo test` — not runnable before crate scaffold; run after task 1.1.1.
- [ ] In-process Axum tests use no fixed/shared TCP ports.
- [ ] SpatiaLite extension availability recorded; missing extension has deterministic unavailable-capability tests.
- Git baseline: repository on `master`, no commits at plan authoring; planning sources untracked. Do not switch branch or initialize Git.
- Conventions/ADR baseline: no repository `AGENTS.md`, `CONTRIBUTING.md`, accepted ADRs, prior plans, or test suites found.

## Wave board

| Wave | Tasks | Status |
|---|---|---|
| 1 | 1.1.1 | done |
| 2 | 1.2.1 | pending |
| 3 | 1.2.2 | pending |
| 4 | 1.3.1, 1.3.2 | pending |
| 5 | 1.4.1, 1.4.2 | pending |
| 6 | 1.5.1 | pending |

## Task reports

<!-- Empty until worker tasks are executed. -->

## 1.1.1
- Status: implemented and verified on `feature/sqlite-mvp`.
- Planning commit: `0133d85` (`docs(planning): record task 1.1.1 micro-plan`).
- Files changed: `Cargo.toml`, `Cargo.lock`, `src/config.rs`, `src/lib.rs`, `src/main.rs`.
- Added CLI/env config, default/override unit test, router endpoints, CORS/tracing/body limit, and runnable server; no spatial initialization.
- `cargo check`: passed.
- `cargo test`: 1 passed, 0 failed (plus empty binary/doc test suites).
- `cargo fmt --check`: passed after formatting.
- Deviations: none.
- Gotchas: route handlers are intentional 501 stubs; later route task replaces them.
