# Project Goals (`planning/GOALS.md`)

Last Updated: 2026-11-23
Source State: STATE_14.md (DuckDB security integration assessment)

## Circled Goal (Active)

- **[G50-PARSER-FEASIBILITY]** Parser feasibility for a fail-closed DuckDB SQL authorization gate is established against the pinned executor and its required SQL surface.
  - Why now: Direct probes show that DuckDB accepts statements and setting changes the string classifier rejects. Parser feasibility is the root decision for a defensible authorization design and prevents premature dependency or implementation choices.
  - Exit criteria: Assess `polyglot-sql` AST coverage for DuckDB statements, nested queries, function/table-function calls, identifiers, and temporary-table operations; run offline probes/tests for representative forms; document gaps and recommend reuse, a candidate parser, or a bounded non-parser alternative. No dependency or pin change without separate approval.
  - First action: Map the executor's allowed SQL workflows to the AST and parser API already available through `polyglot-sql`, then add a small offline parser coverage matrix.

## Sequenced Goals (Dependency DAG)

- **[G51-AUTHORIZATION-POLICY]** The DuckDB SQL authorization policy is specified as an explicit fail-closed allowlist over parsed statement and expression forms.
  - Depends on: G50-PARSER-FEASIBILITY.
  - Exit criteria: Define accepted statement types, DDL/temp-table lifecycle, relation and function/table-function policy, nested-query handling, and rejection behavior for settings, extension control, external I/O, and ambiguous syntax. Document supported-query compatibility and known parser limitations before implementation.

- **[G52-PARSER-GATE]** The internal DuckDB executor validates SQL against the approved parsed-AST policy before execution while preserving the tested temporary-table workflow.
  - Depends on: G51-AUTHORIZATION-POLICY.
  - Exit criteria: Replace string-prefix authorization with AST validation; deny unrecognized or unsupported constructs by default; retain request-local execution and batch rollback contracts; add focused tests for allowed and denied statement/AST forms. Keep DuckDB unavailable to HTTP requests.

- **[G53-ADVERSARIAL-QUALIFICATION]** The parser-backed gate is adversarially qualified against the pinned DuckDB engine and documented parser grammar.
  - Depends on: G52-PARSER-GATE.
  - Exit criteria: Test nested/compound statements, comments, quoted identifiers/literals, statement separators, setting/configuration syntax, dangerous functions/table functions, temp-table lifecycle, and parser/runtime disagreement. Record exact versions, reproduction commands, positive/negative findings, and remaining limits; finite tests are not an exhaustiveness proof.

- **[G54-SECURITY-READINESS]** DuckDB integration readiness is reassessed using parser-gate evidence and all remaining security, version, lifecycle, import, and resource risks.
  - Depends on: G53-ADVERSARIAL-QUALIFICATION, G41-IDENTITY, G40-EVIDENCE, G42-EVIDENCE.
  - Exit criteria: Update the security policy and integration assessment with tested guarantees, residual risks, and a go/no-go recommendation. Keep SQLite as default; request-path selection, dependency/pin changes, and production exposure require separate explicit approval.

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

- None currently. G37-EVIDENCE remains independent of the parser authorization series except for exact pin identity.

## Deferred

- **[G43-ENABLEMENT]** Expose DuckDB to requests or select it at runtime.
  - Trigger: Separate explicit approval after security, compatibility, deployment, and operational evidence is reviewed; outside Plan 13.
- **[G55-OS-ISOLATION]** Add process/OS isolation for DuckDB execution.
  - Trigger: A threat-model review after G54 shows parser-backed authorization and DuckDB engine controls leave material risk that requires isolation. No sandbox design or deployment change is authorized by this goal list.

## Blocked / Pruned

- **[G40-DIRECT-ST-READ]**
  - Status: Pruned
  - Reason: Pinned `ST_Read` direct file access was observed to read outside `allowed_paths`; the plan now uses Rust-side read-only loading and does not rely on that DuckDB control.
- **[G42-OLD-SEQUENCE]**
  - Status: Pruned
  - Reason: Benchmarking is not inherently blocked until every risk is eliminated; it is sequenced after security and loader evidence so results are correctly scoped and the harness is safe to run.

## Ranking Rationale

- G50-PARSER-FEASIBILITY is circled because it decides whether existing parser infrastructure can support the authorization boundary; its answer unblocks policy design and avoids premature dependency/implementation work.
- G51 -> G52 -> G53 -> G54 is the critical parser-backed authorization sequence. Each stage depends on evidence from the previous stage; G54 also depends on current identity, loader, and benchmark evidence.
- G39-EVIDENCE is complete as Plan 14 security investigation; its direct probes established that settings and other SQL accepted by DuckDB differ from the interim executor classifier. The new sequence addresses the resulting design gap rather than repeating baseline probing.
- G41-IDENTITY is a low-cost independent root and gates pinned semantic comparisons. The current DuckDB 1.2.0 / engine 2.0.0 pin was chosen for reproducibility and a matching Spatial artifact, not established as the newest or best pin.
- G40-EVIDENCE follows the security probe but requires only bounded synthetic local inputs and can yield useful qualification evidence despite G39 residual risks.
- G37-SEMANTICS is independent of executor security once the exact Spatial pin is known; parallelize only if test artifacts and files do not overlap.
- G42-EVIDENCE follows G39, G40, and identity verification; performance misses alone are not blockers.
- All goals accept positive, negative, or bounded inconclusive findings when the investigation is completed with reproducible evidence. Implementation, dependency changes, and runtime enablement remain separate approval gates.
