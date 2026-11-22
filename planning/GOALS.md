# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-22
Source State: STATE_13.md (DuckDB qualification through G42)

## Circled Goal (Active)

- **[G39-EVIDENCE]** DuckDB security controls and bypass resistance have reproducible, pinned-version evidence, with enforceable guarantees distinguished from residual risks.
  - Why now: Security evidence determines what can safely be exercised in G40 and benchmarked in G42. Prior missing documentation is not itself a blocker: run bounded direct-API and adversarial probes, then report positive, negative, or genuinely inconclusive results.
  - Exit criteria: A reproducible matrix covers classifier bypasses, file and network surfaces, writes, extension loading/configuration, and resource-control mechanisms. Each entry records exact pinned versions, setup, commands, observed result, and scope limits. The verdict states which controls are enforced, which are bypassable, and what remains a production-enablement risk. No claim of exhaustive security from finite tests.
  - First action: Inspect the current classifier, denial tests, DuckDB 2.0 security documentation/API, and existing probe harness; create an adversarial test matrix and run direct-Connection probes that bypass the classifier.

## Sequenced Goals (Dependency DAG)

- **[G40-EVIDENCE]** The private Rust-side GeoPackage loader has reproducible evidence for trusted-path handling, read-only behavior, metadata/CRS/SRID checks, geometry decoding, and SDA serialization.
  - Depends on: G39-EVIDENCE (security boundary and residual risks documented; does not require every residual risk to be eliminated to perform isolated internal qualification).
  - Exit criteria: Inspect and run the in-module synthetic tests; add targeted adversarial cases for canonical/trusted path capability, read-only open, metadata ambiguity/mismatch, malformed/truncated GeoPackageBinary/WKB, NULL/empty/dimensional values, and exact result serialization. Distinguish properties proved by the loader from deployment lifecycle properties not wired into it.

- **[G37-EVIDENCE]** All six geography methods have method-specific, reproducible semantic verdicts against SQL Server geography behavior.
  - Depends on: G41-IDENTITY (exact client/engine/extension identity and reproducible local test setup).
  - Exit criteria: For Contains, Within, Intersection, Buffer, Area, and AsText, record authoritative references, pinned observations, SQL Server reference results where available, units/axis/SRID/NULL/error/boundary behavior, tolerances, and runtime guard feasibility. Specifically investigate `.STArea()` spheroid axis order and reference values/tolerance/runtime guards, and `.STAsText()` output/dimensional/NULL behavior. A documented no-go or inconclusive verdict is acceptable only after the bounded investigation is completed and reproducible.

- **[G41-IDENTITY]** The DuckDB and Spatial versions used by qualification are verified as a deliberate pin, with upgrade impact assessed.
  - Depends on: none.
  - Exit criteria: Record current supported release context from authoritative sources; compare at least the current pin and a suitable newer version for client/API, engine/extension compatibility, relevant security controls, loader/result behavior, and geography methods. Research-only comparison may label runtime-dependent dimensions unverified; no alternate-version build or dependency change without separate approval. Preserve the current pin unless evidence supports a separately approved upgrade.

- **[G42-EVIDENCE]** Representative DuckDB workloads and resource behavior are measured reproducibly, with limits recommended from evidence.
  - Depends on: G39-EVIDENCE, G40-EVIDENCE, G41-IDENTITY.
  - Exit criteria: Correctness-checked benchmark results cover scalar/select, captured join, batch/temp-table, serialization, and spatial workloads; report reproducible latency distributions and resource/concurrency observations against the SQLite baseline. The 20% objective and >10% regressions are reported, not gates. Recommend candidate limits with uncertainty; do not set production caps or enable request routing.

## Independent Backlog

- None currently. G37-EVIDENCE is a sequenced workstream depending only on current-pin identity research, not on G39/G40 closure.

## Deferred

- **[G43-ENABLEMENT]** Expose DuckDB to requests or select it at runtime.
  - Trigger: Separate explicit approval after security, compatibility, deployment, and operational evidence is reviewed; outside Plan 13.
- **[G39-REDESIGN]** Replace or supplement controls with parser/authorizer or OS isolation.
  - Trigger: Investigation demonstrates a material bypass that cannot be acceptably bounded in the current internal qualification scope. The investigation must establish the observed bypass and alternatives first; do not stop merely because a stronger design has not yet been chosen.

## Blocked / Pruned

- **[G40-DIRECT-ST-READ]**
  - Status: Pruned
  - Reason: Pinned `ST_Read` direct file access was observed to read outside `allowed_paths`; the plan now uses Rust-side read-only loading and does not rely on that DuckDB control.
- **[G42-OLD-SEQUENCE]**
  - Status: Pruned
  - Reason: Benchmarking is not inherently blocked until every risk is eliminated; it is sequenced after security and loader evidence so results are correctly scoped and the harness is safe to run.

## Ranking Rationale

- G39-EVIDENCE is circled because its results bound how G40 and G42 can safely be executed and what their claims mean. “Not previously verified” is a reason to probe, not a stop condition.
- G41-IDENTITY is a low-cost independent root and gates pinned semantic comparisons. The current DuckDB 1.2.0 / engine 2.0.0 pin was chosen for reproducibility and a matching Spatial artifact, not established as the newest or best pin.
- G40-EVIDENCE follows the security probe but requires only bounded synthetic local inputs and can yield useful qualification evidence despite G39 residual risks.
- G37-SEMANTICS is independent of executor security once the exact Spatial pin is known; parallelize only if test artifacts and files do not overlap.
- G42-EVIDENCE follows G39, G40, and identity verification; performance misses alone are not blockers.
- All goals accept positive, negative, or bounded inconclusive findings when the investigation is completed with reproducible evidence. Implementation, dependency changes, and runtime enablement remain separate approval gates.
