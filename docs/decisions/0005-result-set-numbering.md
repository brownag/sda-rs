---
id: "0005"
title: "Number Result Sets by Non-Empty Position"
status: "Accepted"
date: 2026-11-17
authors:
  - "Andrew Gene Brown"
supersedes: null
superseded_by: null
source_context: "EXECUTION_PLAN_2 (decision D4); GOALS.md G21; commit 5193bd1"
---

# 5. Number Result Sets by Non-Empty Position

## Context and Problem Statement
A batch can return several result sets, and some may be empty. Result-set keys must be predictable for clients.

## Decision Drivers
* Keys must not depend on how many empty sets came before.

## Considered Options
1. Absolute index keying, where the key reflects the set's position in the batch. Replaced: plan 2 says to replace "the current absolute-index keying."
2. Number only non-empty sets: `Table`, `Table1`, `Table2`, and so on. **Chosen.**

## Decision Outcome
Chosen option: the first non-empty result set is keyed `Table`, and later non-empty sets are `Table1`, `Table2`, and so on. Empty result sets are skipped when numbering.

### Consequences
* **Positive**: Key numbering is stable regardless of empty sets earlier in the batch.
* **Negative**: The key for a given set depends on the other sets in the batch. Test: an empty first set followed by a non-empty second set yields `Table`.
