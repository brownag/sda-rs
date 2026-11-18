---
id: "0004"
title: "Roll Back All Writes When a Multi-Statement Batch Fails"
status: "Accepted"
date: 2026-11-17
authors:
  - "Andrew Gene Brown"
supersedes: null
superseded_by: null
source_context: "EXECUTION_PLAN_3 (Dated decisions); GOALS.md D3; commit 1a3b62d"
---

# 4. Roll Back All Writes When a Multi-Statement Batch Fails

## Context and Problem Statement
A request can contain several statements. If a later statement fails, earlier writes in the same batch must not persist.

## Decision Drivers
* A failed request leaves the database unchanged.
* Successful batches still commit.

## Considered Options
1. Per-statement savepoints. Rejected: plan 3 says a transaction around `execute_on` "matches the batch unit and keeps the change small."
2. One transaction around the whole batch. **Chosen.**
3. Nested transactions or savepoints for user-issued `BEGIN`/`COMMIT`. Out of scope: security guardrails already reject transaction control.

## Decision Outcome
Chosen option: wrap the whole batch in one transaction around `execute_on`. Per-statement savepoints were not used because they add complexity without changing batch behaviour.

### Consequences
* **Positive**: Partial writes from a failed request are impossible. Covered by `router_rolls_back_failed_statement_batch`.
* **Negative**: A long batch holds its transaction for its full duration.
