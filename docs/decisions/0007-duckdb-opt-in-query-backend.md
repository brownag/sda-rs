---
id: "0007"
title: "Add DuckDB as an Opt-In Query Backend"
status: "Accepted"
date: 2026-11-19
authors:
  - "Andrew Gene Brown"
supersedes: null
superseded_by: null
source_context: "EXECUTION_PLAN_10 (Dated decisions D6-D11 and approved 10.2.1 performance target)"
---

# 7. Add DuckDB as an Opt-In Query Backend

## Context and Problem Statement

SQLite/GeoPackage is the supported default backend. Plan 8 evaluated DuckDB
without establishing production service integration or authorizing repository
dependency changes. Plan 10 sets an opt-in production qualification path that
seeks performance while preserving existing behavior and explicit limits.

## Decision Drivers

* Keep existing SQLite deployments and behavior supported and unchanged by
  default.
* Permit explicit selection of a DuckDB query backend, initially targeting
  Linux x86_64 only.
* Preserve the documented SQL support and rejection surface; do not imply
  SQL Server `geography` parity.
* Keep request state isolated and spatial capability optional.
* Ensure requests do not access the network to obtain extensions.

## Considered Options

1. Replace SQLite with DuckDB. Rejected: SQLite remains the supported default
   and DuckDB is an opt-in alternative.
2. Add DuckDB as an opt-in alternative while retaining SQLite. **Chosen.**
3. Require SpatiaLite for DuckDB spatial queries or silently fall back to
   SQLite/SpatiaLite. Rejected: DuckDB must not require SpatiaLite, and backend
   selection must not silently change.

## Decision Outcome

Chosen option: qualify DuckDB as an additive, opt-in query backend. SQLite
remains the default and supported backend. The initial DuckDB target is Linux
x86_64; this does not promise support on other platforms.

The DuckDB path preserves the existing documented query contract, including
explicit rejection of unsupported SQL and spatial methods. In particular,
rejected CLR geography methods are not implemented using planar substitutes,
and this decision makes no SQL Server `geography` compatibility or parity
claim.

Each request uses request-local execution state, including temporary tables, so
state does not leak between requests. A failed multi-statement request does not
leave partial writes. Persistent writes remain outside the accepted query
surface, consistent with the existing execution security boundary.

DuckDB operation does not require SpatiaLite and never silently falls back to
SpatiaLite. Non-spatial queries remain available when DuckDB Spatial is absent
or cannot be provisioned. A spatial request in that condition fails explicitly
with a clear error. Required Spatial artifacts are provisioned and verified
during deployment; request handling performs no extension network access.

This ADR records the backend contract only. It does not authorize adding or
updating dependencies, implement the backend, or approve a performance result.
The approved qualification objective is at least 20% improvement in
geometric-mean median latency across scalar/select, captured join,
batch/temp-table, and serialization workloads, with no individual workload
regressing by more than 10%; spatial results are reported separately. Measure
and report these results, but missing the 20% objective alone is not a
qualification or release blocker. This performance objective does not waive
security qualification requirements. Numeric resource limits remain separate
and require measurement and explicit approval.

## Consequences

* **Positive**: Users can opt into qualification of DuckDB without changing the
  SQLite default or requiring SpatiaLite for non-spatial operation.
* **Negative**: DuckDB support is initially limited to Linux x86_64 and its
  supported SQL surface remains bounded by documented behavior and explicit
  rejections.
* **Neutral**: Dependency authorization, security/resource/deployment policy,
  implementation, and final qualification remain separate work.
