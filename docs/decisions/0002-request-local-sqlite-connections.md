---
id: "0002"
title: "Open a Request-Local SQLite Connection on a Blocking Worker"
status: "Accepted"
date: 2026-11-17
authors:
  - "Andrew Gene Brown"
supersedes: null
superseded_by: null
source_context: "EXECUTION_PLAN_1 (Dated decisions); commits ca7e0e5, 1971f5a"
---

# 2. Open a Request-Local SQLite Connection on a Blocking Worker

## Context and Problem Statement
Batches can create temporary tables and set session state. These must not leak between requests. `rusqlite` is synchronous, so queries must not block the async runtime.

## Decision Drivers
* Batch and temp-table session semantics must be scoped to one request.
* The Axum handler must not block on SQLite I/O.

## Considered Options
The plans record the chosen design but no rejected alternatives. Plan 1 states it preserves "the explicitly selected `rusqlite` plus per-request blocking execution architecture."

## Decision Outcome
Chosen option: `rusqlite`, with each request opening its own connection on a blocking worker thread. The configured database path is passed through Axum state.

### Consequences
* **Positive**: Session state cannot leak between requests.
* **Negative**: A plain `:memory:` database disappears between requests. Tests must use a shared-cache URI (see [0006](0006-shared-cache-in-memory-test-databases.md)). The runnable service must bind to a file path, not an in-memory database.
