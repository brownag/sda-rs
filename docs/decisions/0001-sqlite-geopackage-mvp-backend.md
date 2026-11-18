---
id: "0001"
title: "Use SQLite/GeoPackage as the MVP Backend"
status: "Accepted" # Proposed | Accepted | Deprecated | Superseded
date: 2026-11-17
authors:
  - "Andrew Gene Brown"
supersedes: null
superseded_by: null
source_context: "EXECUTION_PLAN_1 (Dated decisions); commit 22177c6"
---

# 1. Use SQLite/GeoPackage as the MVP Backend

## Context and Problem Statement
The service emulates the USDA-NRCS Soil Data Access endpoint, which is backed upstream by PostGIS. The MVP needs to run locally against OpenSSURGO GeoPackage data without a database server.

## Decision Drivers
* Run from a single local data file.
* Match the spatial data format users already have (OpenSSURGO GeoPackage).

## Considered Options
1. PostgreSQL/PostGIS, matching the upstream SDA backend.
2. SQLite with GeoPackage/SpatiaLite spatial support.

## Decision Outcome
Chosen option: "SQLite with GeoPackage/SpatiaLite spatial support" (option 2). PostgreSQL parity is not a release gate for the MVP.

Rationale recorded in the plan: the MVP prioritizes SQLite/GeoPackage spatial execution, and the end-to-end AOI query is the priority, not full SDA parity. Option 1 was not chosen for the MVP; the plan does not record its specific trade-offs.

### Consequences
* **Positive**: No database server required; one file per dataset.
* **Negative**: Behaviour that differs between T-SQL/PostGIS and SQLite is not guaranteed to match upstream.
