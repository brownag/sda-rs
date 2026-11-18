---
id: "0006"
title: "Use Shared-Cache In-Memory SQLite for Integration Tests"
status: "Accepted"
date: 2026-11-17
authors:
  - "Andrew Gene Brown"
supersedes: null
superseded_by: null
source_context: "EXECUTION_PLAN_2 (decision D1); GOALS.md D1; EXECUTION_PLAN_3"
---

# 6. Use Shared-Cache In-Memory SQLite for Integration Tests

## Context and Problem Statement
Each request opens its own connection (see [0002](0002-request-local-sqlite-connections.md)). A plain `:memory:` database disappears when the request's connection closes, so integration tests cannot seed data and then query through the router.

## Decision Drivers
* Tests need no external data files.
* Tests must exercise the real router path.

## Considered Options
1. Plain `:memory:` databases. Rejected: each request's connection gets a fresh database, so seeded data disappears.
2. Named shared-cache URIs with one holder connection per test. **Chosen.** Plan 3 requires a unique URI per test.
3. Test fixtures on disk. Not recorded in the plans as considered.

## Decision Outcome
Chosen option: named shared-cache URIs, `file:<unique>?mode=memory&cache=shared`, with one holder connection kept open for the whole test. Unit tests may still use plain `:memory:`.

### Consequences
* **Positive**: Integration tests run against the real request path with no external data.
* **Negative**: Tests must keep the holder connection alive. Each test needs a unique URI.
