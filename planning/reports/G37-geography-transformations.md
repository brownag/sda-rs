# G37 geography transformations and contract

- **Task:** Plan 13, Task 13.2.1 — research G37 geography transformations and contract.
- **Repository:** `/home/andrew/workspace/sda-rs`, branch `feature/duckdb-feature1`, HEAD
  `ecae0155b8e8d1129570d596fb5ec0db71f5f49d` (clean at analysis start).
- **Date:** 2026-11-20.
- **Status:** research report only. No production source, dependency, contract, or data
  change. No probe was executed: the required local official Spatial artifact is absent
  (Section 9). Every runtime claim below is attributed to a prior recorded probe on a
  pinned scratch build, not to a new measurement.

## 1. Verdict summary

No method is approved for implementation by this report. `STArea` and `STAsText` have
conditional bounded-domain candidates that remain **unproven hypotheses** until the
gates in Sections 6.5, 6.6, and 8 are met; the other four methods are explicit no-gos.

| Method | SQL Server `geography` contract (documented) | Strongest candidate considered | Verdict | Why |
|---|---|---|---|---|
| `STContains(other)` | `bit`/`SqlBoolean`; 1/0; `NULL` when SRIDs differ. | Planar `ST_Contains`, S2 `s2_contains`, margin-bounded planar emulation. | **No-go** | DuckDB documents `ST_Contains` as returning true for a boundary point; the SQL Server method page does not define the boundary-point rule, so no evidence establishes that the two agree. Planar and S2 edges are not ellipsoidal geodesics. Margin emulation has no soundness proof and no oracle. |
| `STWithin(other)` | `bit`/`SqlBoolean`; 1/0; always `NULL` when SRIDs differ. | `ST_Within`; reversed `s2_contains`; `s2_within`. | **No-go** | Same boundary/edge problem as `STContains`. Reversed `s2_contains` returned false for an interior point in the recorded probe; `s2_within` is documented but was absent from the pinned binary. |
| `STIntersection(other)` | `geography`; always `NULL` when SRIDs differ; may return `FullGlobe`, >hemisphere, or curve results. | Planar `ST_Intersection`, S2 `s2_intersection`. | **No-go** | Exact ellipsoidal-geodesic overlay is unavailable; planar and spherical overlays are different operations. Result type/domain includes cases neither candidate represents. |
| `STBuffer(distance)` | `geography`; distance in meters; tolerance-based approximation (`tolerance = abs(distance) * .001`, relative=false); negative buffers; FullGlobe cases; maximum-distance bound. | Planar `ST_Buffer`, S2 `s2_dwithin` (predicate only). | **No-go** | No S2 buffer exists; planar distance is in coordinate units. Reproducing SQL Server's documented tolerance algorithm is not possible with the available evidence, and a geodesic approximation is not the same operation. |
| `STArea()` | `float`/`SqlDouble`; squared units of the SRID (square meters for 4326); 0 for empty or 0-/1-D instances. | `ST_Area_Spheroid` (GeographicLib, WGS84). | **Conditional candidate** | The documented quantity is ellipsoidal surface area. A narrow per-value-guarded 4326 polygon domain (Section 6.5) can make the mathematical semantics unambiguous only after the pin's axis order is established; numeric agreement with SQL Server still requires an explicit acceptance criterion and tests. |
| `STAsText()` | `nvarchar(max)`/`SqlChars`; OGC WKT; omits Z and M; supports FullGlobe and >hemisphere. | `ST_AsText(ST_Force2D(g))`. | **Conditional candidate** | Documented semantics are "OGC WKT without Z/M"; a guarded 2D linear-geometry domain can satisfy that. Byte-for-byte formatting parity with SQL Server is not established and must not be claimed. |

The four no-gos are not a regression from G12/Plan 9; they are the same conclusion after
evaluating the DuckDB-specific candidates. The two candidates exist because the DuckDB
spatial extension now exposes an ellipsoidal area routine and a WKT serializer that can
be combined with per-value dimensional guards. Neither candidate is a planar
substitution: the area candidate computes ellipsoidal area, and the text candidate
serializes coordinates without interpreting topology.

## 2. Evidence standard

Every statement is labeled by kind:

- **Documented** — stated on a cited primary page (Microsoft Learn or DuckDB docs).
- **Observed** — recorded from the prior pinned scratch probe in
  `planning/reports/G33-2-duckdb-spatial-followup.md` (engine v2.0.0, client
  `duckdb = "=1.20000.2"`).
- **Hypothesis** — inference or proposed design not yet verified on the pinned build.

"Exact bounded subset" in this report means: for every input value admitted by
per-value runtime guards, the transformation produces the documented SQL Server result
(including NULL/error/units), and every other input is rejected. It does **not** mean a
small happy-path sample, an AST-shape check, or a planar/approximate operation described
as geography. There is no SQL Server runtime oracle in this project; no numeric or
textual parity may be claimed without the evidence gate named in the relevant section.

## 3. Current seam and baseline

- `src/transpiler/spatial.rs::rewrite_node` (lines ~1027–1122) rejects all six names via
  `unsupported_geography_method` (lines ~1124–1135) before inspecting arity or
  arguments. The rejection is a structured `InvalidInput`:
  `".{method} is unsupported for SQL Server geography compatibility"`.
- Unit tests at lines ~1385–1440 assert rejection for method-call receivers (columns and
  constructed instances), several arities, mixed case, and that free-function spellings
  are not lowered.
- `docs/spatial-sql-contract.md` explicitly rejects all six methods and makes no
  geography, area, or unit guarantee.
- No production `src/` module imports DuckDB; `tests/duckdb_pipeline_contract.rs` is a
  feature-gated T-SQL-to-DuckDB contract test with no spatial extension.
- Consequence: today the six names have **rejection semantics only**. Any future support
  requires new per-value runtime guards; expression shape and GeoPackage column
  metadata cannot prove a value's SRID, dimension, extent, validity, or orientation.
  This matches the approved decision that AST/metadata-only bounds are insufficient.

## 4. SQL Server `geography` contract

### 4.1 Type model

- `geography` is a CLR type representing round-earth/ellipsoidal data with latitude and
  longitude; the default SRID is 4326 (WGS 84). SRIDs determine the unit of measure;
  4326 metric results are in meters/square meters.
- Methods derived from two instances return `NULL` when SRIDs differ (also stated on the
  SRID page for both `geometry` and `geography`).
- Polygon interior follows the **left-hand rule**: walking the ring in listed order, the
  left side is interior. Ring orientation is therefore semantically significant for
  `geography` (unlike planar `geometry`); by the same rule, holes must be ordered
  clockwise.
- SQL Server supports FullGlobe and instances larger than a hemisphere; some methods
  have special handling or throw for FullGlobe.
- Geography method error tolerance can be as large as `1.0e-7 * extents`, where extents
  is the approximate maximal distance between points of the object.
- Curves (`CIRCULARSTRING`/`CURVEPOLYGON`) are part of the geography type and can appear
  in results when present in inputs.

### 4.2 Per-method documented behavior

| Method | Argument/arity | Result | NULL/error | Units/SRID | Special cases |
|---|---|---|---|---|---|
| `STContains` | one `geography` | `bit` (CLR `SqlBoolean`), 1/0 | `NULL` if SRIDs differ; no invalid-input policy stated | SRID-sensitive topology | None stated beyond SRID NULL. |
| `STWithin` | one `geography` | `bit`, 1/0 | Always `NULL` if SRIDs differ | SRID-sensitive topology | None stated. |
| `STIntersection` | one `geography` | `geography` | Always `NULL` if SRIDs differ | Result keeps geography semantics | May return `FullGlobe`; may contain arcs only if inputs do; supports >hemisphere. |
| `STBuffer` | one `float` distance (meters) | `geography` | Throws `ArgumentException` on NaN/infinite distance and on distance beyond the maximum for FullGlobe | Distance in meters; output geography | Tolerance-based (`abs(distance)*.001`, relative=false); negative buffers remove points; may return `FullGlobe`; maximum `0.999*π*minorAxis²/majorAxis` (~half circumference). |
| `STArea` | none | `float` (`SqlDouble`) | Not documented for invalid input | Squared units of the SRID; 4326 = square meters | 0 for empty or only 0-/1-D figures; metric result depends on SRID. |
| `STAsText` | none | `nvarchar(max)` (`SqlChars`) | Not documented for invalid input | SRID not serialized | WKT omits Z and M; supports FullGlobe and >hemisphere. |

### 4.3 What the primary pages do *not* establish

- The exact treatment of boundary points in `STContains`/`STWithin` is not stated on the
  method pages. The OGC Simple Features "Contains" definition (boundary-only points are
  not contained) is a different specification; citing it does not establish SQL Server
  behavior, and no inspected SQL Server page adopts or documents that rule for these
  methods.
- No general invalid-geometry error policy for these methods.
- No numeric algorithm, formatting precision, or canonical WKT text.
- No statement that `STBuffer` output can be reproduced by a particular geodesic
  construction; it is explicitly tolerance-based and "isn't precise".

These gaps matter: without an oracle, "matches SQL Server" can only mean "conforms to
the documented contract for the bounded domain", never "byte/number identical".

## 5. Prior evidence on the available backends

### 5.1 SpatiaLite (G12/Plan 9)

- `ST_Contains`/`ST_Within`/`ST_Intersection`/`ST_Buffer`/`ST_Area` are GEOS/planar at
  the SQLite seam; buffer distance follows coordinate units (a distance-1 buffer at 4326
  spanned about ±1 degree, not 1 meter).
- Mismatched SRIDs returned `1` for a planar predicate where SQL Server returns `NULL`.
- Ellipsoidal `ST_Area(...,1)` uses RTTOPO/PROJ, not SQL Server; the cross-equator case
  emitted `RTTOPO error: ptarray_area_spheroid: cannot handle ptarray that crosses
  equator`. The north-of-equator rerun returned `12304814950.073112` without that
  warning, which establishes only that single case.
- `AsText` retained Z/M ordinates, unlike SQL Server's documented omission.
- No safe input bound was enforceable at that seam; all six were rejected.

### 5.2 DuckDB Spatial (G33-2/Plan 8, pinned scratch probe)

Engine v2.0.0, client `duckdb = "=1.20000.2"`, Spatial extension identifier
`a3dbcd1781`, SHA-256
`8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc`.

| Observation | Recorded result | Relevance |
|---|---|---|
| `ST_Contains` interior/boundary | interior `true`; the probe did not test an ordinary `ST_Contains` boundary point, but DuckDB's page documents boundary-inclusive containment | Direct lowering cannot be shown to match: DuckDB is boundary-inclusive and SQL Server's boundary rule is undocumented in the inspected pages. |
| `ST_Within` | interior `true`; `NULL` → `NULL` | Planar only. |
| `ST_Intersection` | polygon/line `LINESTRING (0 0.5, 1 0.5)`; disjoint points `POINT EMPTY`; `NULL` → `NULL`; mismatched CRS → bind-time CRS error | Planar overlay; CRS mismatch is an error, not SQL Server's `NULL`. |
| `ST_Buffer` | distance 1 around `(0,0)` returned a polygon with coordinates to ±1; `NULL` → `NULL` | Planar coordinate units; documented as ignoring earth curvature. |
| `ST_Area` | 1° square = `1.0`; empty polygon `0.0`; `NULL` → `NULL` | Coordinate-units area. |
| `ST_Area_Spheroid` | north-of-equator polygon = `12305128751.042904` with `geometry_always_xy=true` | Ellipsoidal WGS84 area via GeographicLib; a distinct operation, not ordinary `ST_Area`; the fixture does not establish axis order. |
| `ST_AsText` | WKT; `POINT Z`/`POINT M` retained Z/M | Differs from SQL Server's documented omission for 3D/4D input. |
| S2 community `geography` | `s2_contains` true for interior and a vertex, false for one edge point; reversed `s2_contains` false for interior; `s2_within` missing from the installed binary; `s2_intersection` spherical; `s2_area` perfect-sphere square meters (`12364036567.076418` for the 1° square); no buffer function | Spherical, not ellipsoidal; function availability differs; SQL Server's boundary rule is undocumented. |
| CRS/SRID interfaces | `ST_CRS`/`ST_SetCRS` are documented in DuckDB 2.0; `ST_SRID` and `spatial_version()` were unavailable in the pinned probe; CRS mismatch raises an error | Guards must use the interfaces actually present on the pin; this is unverified. |

The three area figures (`12304814950.073112` SpatiaLite, `12305128751.042904` DuckDB
spheroid, `12364036567.076418` S2) demonstrate that algorithm choice changes the number.
None is evidence of SQL Server numeric parity.

## 6. Candidate evaluation by method

### 6.1 `STContains` — no-go

Candidates considered:

1. **Direct `ST_Contains`.** Rejected. DuckDB documents that its `ST_Contains` returns
   true when the second geometry lies on the boundary of the first. The SQL Server
   `STContains` page defines "spatially contains" but does not state whether a
   boundary-only point is contained; no inspected page establishes that the two agree.
   The risk is a semantic mismatch, not a probe artifact.
2. **S2 `s2_contains`.** Rejected. Spherical great-circle edges differ from SQL Server's
   ellipsoidal geodesic edges; the recorded probe returned true for a polygon vertex.
   OGC-style containment would exclude a boundary-only point, but SQL Server's exact rule
   is not documented, so the vertex result cannot be accepted as SQL Server behavior.
   SRID-mismatch NULL is also unavailable as a native behavior.
3. **Margin-bounded planar emulation** (accept a result only when the argument is farther
   from the receiver's boundary than the maximum planar-vs-geodesic edge deviation).
   Rejected for this plan: it needs a rigorous, latitude/extent-dependent deviation bound,
   a conservative per-value distance-to-boundary guard, and a documented SQL Server
   boundary rule; none is established, and a wrong margin silently changes the answer.

No bounded exact domain is established. Retain explicit rejection.

### 6.2 `STWithin` — no-go

1. **Direct `ST_Within`.** Rejected for the same unestablished boundary rule and
   planar-edge reasons.
2. **Reversed `s2_contains`.** Rejected: the recorded probe returned false for a point
   that was interior to the polygon, and the semantics of reversing arguments are not
   established.
3. **`s2_within`.** Rejected on the pin: documented in the maintainer reference but
   absent from the installed binary (`Catalog Error: Scalar Function with name s2_within
   does not exist`). Even if present, it is spherical, not ellipsoidal.

Retain explicit rejection.

### 6.3 `STIntersection` — no-go

1. **Planar `ST_Intersection`.** Rejected: a planar overlay of lon/lat edges is not an
   ellipsoidal overlay; result geometry, arcs, FullGlobe, and >hemisphere cases are not
   represented. The recorded polygon/line result (`LINESTRING (0 0.5, 1 0.5)`) is a
   planar artifact.
2. **S2 `s2_intersection`.** Rejected: spherical great-circle overlay returned
   `LINESTRING (0 0.5001523435130507, 1 0.5001523435130508)` for the same inputs,
   proving the result differs from the planar one and is still not the ellipsoidal
   operation. SRID-mismatch NULL and FullGlobe semantics are unavailable.

No exact transformation exists with the available operations. Retain explicit rejection.

### 6.4 `STBuffer` — no-go

1. **Planar `ST_Buffer`.** Rejected: distance is in coordinate units, explicitly planar.
2. **S2.** Rejected: no buffer operation exists (`s2_dwithin` is a predicate, not a
   buffer geometry), and it is spherical anyway.
3. **Geodesic approximation.** Rejected by the approved decision: SQL Server documents a
   tolerance-based construction with `tolerance = abs(distance)*.001`, relative=false,
   an error bound `max(tolerance, extents*1e-7)`, negative-buffer behavior, and FullGlobe
   cases. Reproducing that algorithm is not possible from the available evidence, and a
   geodesic offset curve would produce different geometry even if it looked plausible.

Retain explicit rejection.

### 6.5 `STArea` — conditional bounded candidate (not proven)

**Documented target.** Total surface area of the geography instance in the SRID's squared
units; for 4326, square meters; 0 for empty or only 0-/1-D figures.

**Proposed domain (hypothesis).** A single `POLYGON` value that satisfies all of:

```text
ST_GeometryType(g) = 'POLYGON'
ST_HasZ(g) = false AND ST_HasM(g) = false
ST_IsEmpty(g) = false AND ST_IsValid(g) = true
ST_NInteriorRings(g) = 0
ST_CRS(g) is the WGS84 geographic CRS fixed for the pin
ST_XMin(g) >= -170 AND ST_XMax(g) <= 170
ST_YMin(g) >= -80 AND ST_YMax(g) <= 80
ST_XMax(g) - ST_XMin(g) <= 10 and ST_YMax(g) - ST_YMin(g) <= 10
shell orientation is counter-clockwise (positive shoelace sum in lon/lat)
```

Then transform to `ST_Area_Spheroid(g)`. The bbox guards and the shoelace test are written
in the service's X=longitude, Y=latitude convention, so they are valid only if the
transform is proven to consume that same order.

**Axis order is an unresolved precondition, not an established setting.** The DuckDB 2.0
`ST_Area_Spheroid` page states that input is assumed to be EPSG:4326 with
`[latitude, longitude]` axis order. The G33-2 probe instead set
`geometry_always_xy = true` (the DuckDB 2.0 geometry page's example sets it to assume
`[longitude, latitude]` axis order) and returned `12305128751.042904` for a synthetic
north-of-equator polygon. The recorded fixture is not detailed enough to distinguish the
two interpretations, and a polygon of that size at low latitude is nearly symmetric in
longitude/latitude, so the observation does **not** establish which order the function
consumed. This report asserts
no axis setting. A non-symmetric fixture must establish the pin's behavior before
implementation; if the function consumes `[latitude, longitude]`, the rewrite would need a
verified coordinate swap or session setting, which is itself unverified. Until then the
candidate is blocked. Note also that EPSG:4326 and OGC:CRS84 are not interchangeable here:
their axis orders differ.

**Why each guard exists:**

- *Single polygon, no holes:* keeps the orientation guard and the surface definition
  tractable; multipolygons and holes are rejected rather than approximated.
- *2D only:* avoids unverified Z/M handling for a surface-area method; SQL Server's
  documented result is a 2D surface area.
- *Valid, non-empty:* the method pages do not define invalid-input behavior; rejection is
  explicit.
- *WGS84 CRS:* SQL Server 4326 is WGS84; DuckDB's spheroid routine is documented for
  EPSG:4326 and returns square meters, subject to the axis-order precondition above.
- *Bounding box away from poles and the antimeridian, extent ≤10°:* keeps the polygon
  well inside a hemisphere, keeps equirectangular orientation (shoelace sign) aligned
  with the geodesic interior, and avoids the curvilinear degeneracies at the poles and
  dateline. The 10° figure is a conservative proposal to be tightened or justified when
  the guard tests run; these are domain restrictions, not approximations of the area
  within the domain.
- *CCW shell:* SQL Server's left-hand rule makes CCW the small interior; a clockwise
  ring would mean the complement and a completely different area. The shoelace guard is
  an orientation test, not an area calculation.

**Result/NULL/error/units.** For admitted values: one `DOUBLE` square-meter value,
matching SQL Server's `float`. For `NULL` input: `NULL` (SQL propagation). For a value
outside the domain: a structured rejection, never a planar area and never a silent 0.
Empty/0-/1-D values are outside the proposed domain; SQL Server returns 0 for them, so a
future extension could admit them only after verifying the pinned build's behavior.

**Why this is not yet proven.** `ST_Area_Spheroid` is documented as an ellipsoidal
GeographicLib computation, but (a) the pin's axis order is unresolved: the documentation
says `[latitude, longitude]`, the probe set `geometry_always_xy=true`, and the recorded
fixture cannot disambiguate them; (b) the pinned binary's exact behavior was observed on
only one synthetic polygon; (c) `ST_CRS`/orientation guards have not been exercised on the
pin; (d) SQL Server's own implementation is tolerance-based, so "same number" needs an
explicit acceptance criterion. The area *quantity* is mathematically defined, which is why
this is a candidate rather than an automatic no-go. Wave 13.5.1 may implement it only if
it can show, with recorded independent reference values (e.g. GeographicLib geodesic
polygon area), an asymmetric axis-order fixture, and adversarial guard tests, that the
admitted domain yields the documented quantity within a stated criterion; otherwise it
must retain rejection.

### 6.6 `STAsText` — conditional bounded candidate (semantic WKT only)

**Documented target.** OGC WKT text, `nvarchar(max)`, with no Z or M values; FullGlobe
and >hemisphere supported by SQL Server.

**Proposed domain (hypothesis).** Any non-empty, non-curve (linear-interpolated)
geometry value with a verified WGS84 longitude/latitude CRS:

```text
ST_IsEmpty(g) = false
ST_GeometryType(g) IN ('POINT','LINESTRING','POLYGON',
                       'MULTIPOINT','MULTILINESTRING','MULTIPOLYGON',
                       'GEOMETRYCOLLECTION')
ST_CRS(g) is the verified WGS84 longitude/latitude identifier
```

Transform to `ST_AsText(ST_Force2D(g))`. Z/M are allowed on input and dropped by
`ST_Force2D`, matching SQL Server's documented omission. Curves and FullGlobe cannot be
represented by DuckDB's linear `GEOMETRY` type, so those inputs fail at parse/bind time
and are explicit rejections. Empty values are excluded because the exact empty-geometry
WKT text was not verified.

**Claim and limits.** The claim is *semantic*: the output is valid OGC WKT for the same
2D coordinates, with Z/M absent. It is **not** a byte-for-byte formatting claim. SQL
Server's pages do not specify coordinate precision, whitespace, or canonical number
formatting; DuckDB uses shortest round-trip formatting. If the contract requires
byte-identical text, this candidate is a no-go. The WKT parser/serializer round trip and
the exact fixtures in Section 8 must be tested on the pinned build.

**Result/NULL/error.** `VARCHAR`/`Text`; `NULL` input propagates to `NULL`; out-of-domain
values (empty, non-WGS84 CRS, curve/full-globe input) are rejected explicitly. SRID is
not serialized, matching SQL Server.

## 7. Runtime guard design

Per-value enforcement is mandatory. The available layers are:

| Layer | Can enforce | Cannot enforce |
|---|---|---|
| Transpiler/AST | method name, arity, argument shape, presence of a geometry column, rejection | SRID/CRS, dimension, emptiness, validity, type, extent, orientation, value content |
| GeoPackage metadata | declared column type/SRID for a referenced relation | the actual value in a row, constructor expressions, arbitrary WKT |
| Per-value SQL predicate | `ST_CRS`, `ST_HasZ`, `ST_HasM`, `ST_IsEmpty`, `ST_IsValid`, `ST_GeometryType`, `ST_XMin/XMax/YMin/YMax`, `ST_NPoints`, `ST_NInteriorRings`, `ST_Extent`, `ST_Dimension` | guarantees about functions not present on the pin; orientation unless computed |
| Executor/bind validation | pre-scanning referenced values, rejecting before execution; whole-batch rollback | values produced mid-query by arbitrary expressions |

Design requirements for any future supported subset:

1. Guard every operand per value, before the operation. A `CASE` that returns the
   operation only when all guards hold and otherwise raises/rejects is acceptable only if
   the pinned build exposes a reliable error-raising function (for example `error()`;
   availability and behavior on the pin are unverified), otherwise the executor must
   validate first.
2. Never convert an out-of-domain value to `NULL` or `0` silently. SQL Server's `NULL` is
   primarily for its documented cases (e.g. SRID mismatch, SQL NULL propagation).
3. Preserve `NULL` propagation: guards must evaluate to `NULL`, not false/reject, for a
   SQL `NULL` receiver so the method returns `NULL` like SQL Server.
4. Reject at the whole-request boundary with the existing structured unsupported/
   validation error; keep batch all-or-nothing semantics.
5. Verify guard functions on the exact extension/engine pin, not just the docs. The
   G33-2 probe found `ST_SRID` and `spatial_version()` unavailable; `ST_CRS` is a
   different, built-in interface whose pin behavior is unverified.
6. For two-operand methods (not supported here), SRID equality must be checked
   explicitly because DuckDB raises a CRS error where SQL Server returns `NULL`; any
   future lowering must not leak the DuckDB error shape.

## 8. Adversarial and boundary test proposals

Common cases for every method: SQL `NULL` receiver/argument; wrong arity; non-geometry
receiver; free-function spelling; mixed case; constructor receiver with arbitrary WKT;
SRID/CRS mismatch and missing CRS; Z, M, ZM; empty; invalid WKT/WKB; antimeridian
crossing; poles; >hemisphere; FullGlobe; curve inputs; very large vertex counts.

### 8.1 No-go methods (waves 13.3.1 and 13.4.1)

- Assert the structured rejection for `STContains`, `STWithin`, `STIntersection`,
  `STBuffer` across column, constructor, and alias receivers, and for arities 0, 1, 2+.
- Assert the rejection message does not contain a lowered function name (`ST_Contains`,
  `ST_Within`, `ST_Intersection`, `ST_Buffer`).
- Assert free-function spellings are not rewritten.
- If a future change adds an accepted domain, add in-domain fixtures and out-of-domain
  rejections (SRID mismatch, boundary point, Z/M, empty, antimeridian, pole, FullGlobe,
  curve) — no test may treat a planar result as geography evidence.

### 8.2 `STArea` candidate (wave 13.5.1)

Accepted fixtures (synthetic, 4326): small CCW polygon near the equator; small CCW
polygon at mid-latitude; triangle; polygon with many vertices but no holes. Before any
accepted fixture is used, run a non-symmetric axis-order fixture (for example a
20°-longitude by 2°-latitude rectangle at 60°N) whose area differs materially under
swapped axes, and record which interpretation the pin uses. For each fixture, record the
independent ellipsoidal reference value and its provenance, and assert the transformed
result within a stated criterion.

Rejected fixtures: clockwise shell; polygon with an interior ring; multipolygon;
geometry collection; point/line/empty; Z or M value; invalid (self-crossing) polygon;
CRS 3857 or missing CRS; bbox touching/crossing ±180; |lat| > 80; extent > 10°;
`NULL` (must remain `NULL`, not rejected); string/non-geometry argument; wrong arity.

### 8.3 `STAsText` candidate (wave 13.5.1)

Accepted fixtures: point, line, polygon, multipoint, multilinestring, multipolygon,
collection; point with Z, M, ZM (assert output is 2D and contains no Z/M); large vertex
count within the request limits. For each, assert the output parses as OGC WKT, has the
same 2D coordinate sequence/type, and contains no dimensional tags.

Rejected fixtures: empty geometry; curve input; FullGlobe input; non-WGS84 CRS (if the
domain is restricted); `NULL` (must remain `NULL`); non-geometry receiver; wrong arity.
Explicitly assert that byte formatting is not claimed: a formatting difference alone
must not be reported as SQL Server parity.

## 9. Probe availability blocker

The prior probe artifacts are not present in this checkout, so no new runtime evidence
could be produced under the task's no-network constraint. Exact read-only checks:

```text
$ git rev-parse --abbrev-ref HEAD
feature/duckdb-feature1
$ git rev-parse HEAD
ecae0155b8e8d1129570d596fb5ec0db71f5f49d
$ ls -d /tmp/opencode/sda-rs-plan8-duckdb
ls: cannot access '/tmp/opencode/sda-rs-plan8-duckdb': No such file or directory
$ find /tmp/opencode -name '*.duckdb_extension' -print | wc -l
0
$ ls -d /tmp/opencode/sda-rs-plan13-geography
ls: cannot access '/tmp/opencode/sda-rs-plan13-geography': No such file or directory
```

What is missing: the official Spatial artifact recorded in Plan 8 (engine v2.0.0,
extension `a3dbcd1781`, SHA-256
`8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc`), and the community
`geography` artifact (version `f777314ad0`, SHA-256
`f0c5172314e86530d7680b34c62bc092acd64ae8440b43428a5be7ecbe1ad7ad`). Neither may be
downloaded or installed by this task.

If an authorized local official artifact is supplied later, the reproduction is:

```sh
mkdir -p /tmp/opencode/sda-rs-plan13-geography
# copy the verified artifact into that directory; record source/version/SHA-256
sha256sum /tmp/opencode/sda-rs-plan13-geography/spatial.duckdb_extension
# offline scratch manifest pinned to duckdb = "=1.20000.2"; load by absolute path,
# disable autoload/autoinstall; run synthetic WKT only and record the observed axis
# behavior with an asymmetric fixture before assuming or setting geometry_always_xy
```

The probes that would be required are: version/extension metadata before lockdown;
`ST_Area_Spheroid` on the accepted fixtures with independent reference values; a
non-symmetric axis-order fixture that distinguishes `[latitude, longitude]` from
`[longitude, latitude]` (a `geometry_always_xy` on/off comparison alone does not, because
the recorded low-latitude fixture is nearly symmetric in lon/lat); `ST_CRS` and
CRS-mismatch behavior;
`ST_HasZ`/`ST_HasM`/`ST_IsValid`/`ST_GeometryType`/`ST_NInteriorRings` guard expressions;
`ST_AsText(ST_Force2D(...))` for all types; and the availability/shape of an error-raising
function for guard rejection. Until then, the candidates remain hypotheses and the
extension-backed tests in waves 13.3.1–13.5.1 are blocked.

## 10. Candidate dependencies

No dependency is added, required, or recommended. Research options only, all unapproved:

- **GEOS** (already behind SpatiaLite/DuckDB planar operations): planar only; cannot
  supply geography semantics.
- **S2** (already in the community `geography` extension): spherical great-circle
  operations; S2 documents spherical geodesic edges, left-interior polygons, and that
  exact ellipsoidal predicates are not implemented; no buffer; boundary behavior differs.
- **GeographicLib**: already used inside DuckDB's `ST_Area_Spheroid`; useful for
  independent area reference values in tests, but a new crate/binding would require
  separate approval.
- **PROJ**: CRS definitions/transformations; does not provide geodesic predicates or
  overlay.

The proposed `STArea` candidate relies on the official Spatial extension's existing
GeographicLib-based function, not on a new repository dependency. Any future decision to
add a crate must stop and request explicit user approval before use.

## 11. Risks and open questions

1. **No oracle.** All candidates are contract-conformance arguments, not numeric/textual
   parity. SQL Server's documented tolerances may make "equal" undefined.
2. **Docs vs pin.** Current DuckDB 2.0 docs list functions (`s2_within`, possibly
   `ST_CRS`/`error()`) that may be absent from the pinned binary. Verify on the exact pin.
3. **Axis order.** The DuckDB 2.0 `ST_Area_Spheroid` page says `[latitude, longitude]`,
   while the G33-2 probe set `geometry_always_xy=true` (the geometry page's example uses
   it to assume `[longitude, latitude]` axis order); the recorded low-latitude fixture
   cannot distinguish them. A
   transposed input would silently produce a plausible but wrong area; the pin must be
   tested with an asymmetric fixture before the candidate is accepted.
4. **Orientation.** SQL Server's left-hand rule makes CW vs CCW a different surface.
   The shoelace guard is only valid within the bounded, non-polar, non-antimeridian
   domain and needs verification.
5. **Formatting.** `STAsText` byte parity is not established; semantic WKT agreement must
   be an explicit contract decision if that candidate is pursued.
6. **Extension blocker.** Without a locally supplied verified artifact, no G37 wave can
   run extension-backed tests; the report cannot be upgraded from hypothesis to evidence.

## 12. Primary references

All accessed 2026-11-20.

Microsoft Learn — SQL Server `geography`:

- [STContains (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stcontains-geography-data-type?view=sql-server-ver17)
- [STWithin (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stwithin-geography-data-type?view=sql-server-ver17)
- [STIntersection (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stintersection-geography-data-type?view=sql-server-ver17)
- [STBuffer (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stbuffer-geography-data-type?view=sql-server-ver17)
- [BufferWithTolerance (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/bufferwithtolerance-geography-data-type?view=sql-server-ver17)
- [STArea (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/starea-geography-data-type?view=sql-server-ver17)
- [STAsText (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stastext-geography-data-type?view=sql-server-ver17)
- [geography (Transact-SQL)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/spatial-types-geography-transact-sql?view=sql-server-ver17)
- [Polygon — orientation and left-hand rule](https://learn.microsoft.com/en-us/sql/relational-databases/spatial/polygon?view=sql-server-ver17)
- [Spatial Reference Identifiers (SRIDs)](https://learn.microsoft.com/en-us/sql/relational-databases/spatial/spatial-reference-identifiers-srids?view=sql-server-ver17)
- [Create, Construct, and Query geography instances](https://learn.microsoft.com/en-us/sql/relational-databases/spatial/create-construct-and-query-geography-instances?view=sql-server-ver17)
- [MakeValid (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/makevalid-geography-data-type?view=sql-server-ver17)

DuckDB:

- [DuckDB 2.0 Spatial functions](https://duckdb.org/docs/2.0/core_extensions/spatial/functions.html)
- [DuckDB 2.0 Spatial overview](https://duckdb.org/docs/2.0/core_extensions/spatial/overview)
- [DuckDB 2.0 Geometry data type](https://duckdb.org/docs/2.0/sql/data_types/geometry)
- [DuckDB community geography catalog page](https://duckdb.org/community_extensions/extensions/geography)
- [duckdb-geography function reference](https://github.com/paleolimbot/duckdb-geography/blob/main/docs/function-reference.md)
- [DuckDB 2.0 Securing DuckDB](https://duckdb.org/docs/2.0/operations_manual/securing_duckdb/overview)
- [S2 geometry overview](https://s2geometry.io/about/overview) — spherical geodesic
  edges, left-interior rule, and the stated limits of ellipsoidal operations

Project evidence:

- `planning/reports/G12-geography-method-feasibility.md`
- `planning/reports/G33-2-duckdb-spatial-followup.md`
- `docs/spatial-sql-contract.md`
- `docs/security/duckdb-execution-policy.md`
- `src/transpiler/spatial.rs` (`rewrite_node`, `unsupported_geography_method`, tests)
- `tests/spatial_methods_contract.rs`

## 13. Acceptance status

- [x] All six methods addressed individually with documented contract rows.
- [x] Primary Microsoft Learn and DuckDB references cited; prior project evidence cited.
- [x] Bounded-domain candidates (`STArea`, `STAsText`) defined with per-value guards; the
      other four are explicit no-gos.
- [x] No planar/approximate operation is described as geography-compatible.
- [x] No dependency added or required; candidate libraries are research options only.
- [x] Probe availability blocker recorded with exact commands and artifact identities;
      no probe result is presented as SQL Server parity, and no new runtime claim is made.

## 14. Refreshed source-contract review (2026-11-22; Task 14.3.2)

This section records a source review against the current Microsoft Learn SQL Server
17 documentation and versioned DuckDB 2.0 Spatial documentation, accessed 2026-11-22.
“Documented” below means the cited page states the behavior; it does not imply an
independent runtime oracle. “Prior observation” refers only to the pinned G33-2
record in Section 5.2; no runtime probe was run for this refresh. “Unknown” means
the reviewed primary pages do not define the behavior or that behavior remains
unverified on the pinned artifact.

### 14.1 Method-by-method source contract and evidence plan

| Method | Microsoft Learn documented contract | DuckDB 2.0 documented operation / mismatch | Unknowns and evidence plan |
|---|---|---|---|
| `STContains(other)` | Returns `bit` / CLR `SqlBoolean`: 1 if the calling geography spatially contains the other, else 0; `NULL` for different SRIDs. The method page does not state boundary-point treatment, invalid-input behavior, or dimensional rules. | `ST_Contains(GEOMETRY, GEOMETRY)` returns Boolean; its page explicitly says a geometry on the boundary is contained. This is a documented boundary-inclusive behavior of the planar operation, not SQL Server geography evidence. | Boundary rule/oracle, geodesic edge semantics, SRID mismatch, SQL NULL, empty/invalid and Z/M behavior remain unknown or non-equivalent. Probe interior, exterior, boundary point/edge, vertex, reversed operands, mismatched CRS, NULL, empty, invalid, and dimensional inputs; require explicit rejection unless exact documented-domain equivalence is established. |
| `STWithin(other)` | Returns `bit` / CLR `SqlBoolean`: 1 if the receiver is spatially within the argument, else 0; always `NULL` for differing SRIDs. Boundary, invalid-input, and dimensional rules are not stated on this method page. | `ST_Within(GEOMETRY, GEOMETRY)` returns Boolean in DuckDB Spatial 2.0; it is a geometry operation, not documented geography/ellipsoidal processing. | Boundary and edge rules and NULL/empty/invalid behavior are not established as SQL Server parity. Probe inside/outside/boundary/vertex, argument reversal, CRS match/mismatch, SQL NULL, empty/invalid and Z/M; do not infer parity from an interior-only result. |
| `STIntersection(other)` | Returns `geography` / CLR `SqlGeography`; returns `NULL` for differing SRIDs. Supports larger-than-hemisphere instances and `FullGlobe`; result may contain circular arcs only when inputs do. | `ST_Intersection(GEOMETRY, GEOMETRY)` returns geometry. DuckDB Spatial 2.0 documentation describes a geometry operation; it does not claim SQL Server ellipsoidal geography overlay. | Exact boundary, empty/disjoint output typing, invalid-input errors, arcs, FullGlobe, >hemisphere, dimensional preservation, and SRID NULL remain unestablished for any candidate. Probe crossing/touching/disjoint inputs, CRS mismatch, NULL, empty, invalid, Z/M, and representability limits; acceptance requires result-domain and SRID semantics, not merely similar WKT. |
| `STBuffer(distance)` | Returns `geography` / CLR `SqlGeography`; distance is a floating-point distance. Same construction as `BufferWithTolerance` with tolerance `abs(distance)*.001`, relative=false. Negative distance removes points near the boundary; some cases return FullGlobe. The page documents the maximum-distance restriction and `ArgumentException` condition, plus error bound `max(tolerance, extents*1e-7)`. | `ST_Buffer(GEOMETRY, distance)` is planar, uses input-coordinate units, and ignores Earth curvature. It is not a geography-meter buffer. | SQL NULL, empty/invalid, exact zero/negative output, and representation of FullGlobe remain unverified by the reviewed method page. Probe positive/zero/negative distances, near and beyond the documented maximum, polar/large extents, NULL/empty/invalid; an approximate-looking polygon is not acceptance evidence. |
| `STArea()` | Returns `float` / CLR `SqlDouble`, total surface area in squared units for the instance SRID (4326 uses square meters); returns 0 for empty or instances containing only zero- and one-dimensional figures. | `ST_Area` is planar and uses CRS coordinate units. `ST_Area_Spheroid` returns square meters using an ellipsoidal model and GeographicLib; DuckDB 2.0 docs explicitly assume EPSG:4326 with `[latitude, longitude]` axis order and return 0 for non-polygonal inputs. | SQL Server numeric tolerance/algorithm and invalid-input behavior are not specified by the method page. Prior observation (Section 5.2) set `geometry_always_xy=true` on a near-symmetric low-latitude fixture; it does not settle axis order or parity. Probe an asymmetric polygon (e.g. 20° longitude × 2° latitude near 60°N), compare coordinate interpretations, and record independent ellipsoidal reference values/provenance. Before acceptance, state a justified numeric tolerance and test empty/0D/1D, NULL, invalid, orientation, CRS, and guards; no tolerance can be claimed from documentation alone. |
| `STAsText()` | Returns OGC WKT as `nvarchar(max)` / CLR `SqlChars`; output omits Z and M. Supports FullGlobe and instances larger than a hemisphere. The page does not specify formatting precision/canonicalization or invalid-input behavior. | `ST_AsText(GEOMETRY)` returns `VARCHAR` WKT. `ST_Force2D` drops both Z and M. DuckDB docs do not establish SQL Server formatting parity or representation of geography-only values such as FullGlobe. | Probe POINT, LINESTRING, POLYGON, multi-types and collection in 2D, Z, M, ZM; empty and NULL; parse output back and verify type and 2D coordinate sequence with Z/M absent. Record text differences without treating formatting as parity. Test CRS/curve/FullGlobe capability and guard behavior; nonrepresentable values must remain rejected. |

Across all six methods, SQL NULL propagation and error behavior outside the explicitly
documented cases are not comprehensively specified by these method pages. Invalid
geography handling must not be invented from OGC or DuckDB behavior. “Boundary” is
especially material for predicates: Microsoft’s method pages define the result in
terms of spatial containment but do not state the boundary-only rule. Do not use an
OGC predicate definition or DuckDB’s geometry contract as a substitute for a
SQL Server geography statement.

### 14.2 Area axis, reference, and tolerance requirements

The DuckDB 2.0 `ST_Area_Spheroid` page states `[latitude, longitude]` input order.
That is the documentation contract for that function; it does not establish what the
existing pinned binary did under the prior probe's `geometry_always_xy=true` setting.
Keep those facts separate. The earlier fixture/result in Section 5.2 is an observed
DuckDB value only and cannot discriminate axis order or establish SQL Server equality.

Any future area evidence must include (1) an asymmetric fixture whose area differs
materially when axes are swapped, such as a 20°-by-2° rectangle near 60°N; (2) both
interpretations and explicit coordinates; (3) an independent ellipsoidal reference
with named algorithm/library, version, ellipsoid, input order, and reproducible
method; and (4) a stated acceptance tolerance with rationale. Microsoft documents
the result quantity and units but gives no numeric tolerance for `STArea`; therefore
the tolerance is an explicit project decision, not a fact inferred from the pages.
Also assess orientation and bounded-domain guards, CRS identity, SQL NULL, empty,
zero-/one-dimensional, invalid geometry, and whether each guard is available on the
exact pin. Until those checks pass, retain unconditional rejection.

### 14.3 Text cases and claim limits

For `.STAsText()`, the documented semantic requirements are OGC WKT and omission of
Z/M; Microsoft documents FullGlobe and >hemisphere support. It does not promise a
canonical spelling, whitespace, decimal precision, or byte-for-byte output. DuckDB
documents `ST_AsText` as WKT serialization and `ST_Force2D` as dropping Z/M, but
those pages do not prove the composition's runtime behavior on the locked artifact.
The evidence plan therefore checks 2D, Z, M, and ZM for each supported geometry
type; empty and SQL NULL separately; parseability, geometry type, 2D coordinate
sequence, and absence of Z/M. Record lexical differences as observations only.
FullGlobe, curves, empty-text spelling, CRS enforcement, and exact output typing
remain unknown for the proposed lowering and must be excluded or explicitly tested
before any domain can be admitted.

### 14.4 Source references and provenance

Microsoft Learn, SQL Server 17 view, accessed 2026-11-22:

- [STContains (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stcontains-geography-data-type?view=sql-server-ver17)
- [STWithin (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stwithin-geography-data-type?view=sql-server-ver17)
- [STIntersection (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stintersection-geography-data-type?view=sql-server-ver17) — page metadata reports update 2026-08-24.
- [STBuffer (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stbuffer-geography-data-type?view=sql-server-ver17) — page metadata reports update 2026-08-24.
- [STArea (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/starea-geography-data-type?view=sql-server-ver17) — page metadata reports update 2026-08-24.
- [STAsText (geography)](https://learn.microsoft.com/en-us/sql/t-sql/spatial-geography/stastext-geography-data-type?view=sql-server-ver17) — page metadata reports update 2026-08-24.

Version-matched DuckDB documentation, accessed 2026-11-22:

- [DuckDB 2.0 Spatial functions](https://duckdb.org/docs/2.0/core_extensions/spatial/functions.html) — `ST_Area`, `ST_Area_Spheroid`, `ST_AsText`, `ST_Buffer`, `ST_Contains`, `ST_Intersection`, `ST_Within`, `ST_Force2D`, and geometry guard functions.
- Prior pinned runtime observations remain those explicitly recorded in Section 5.2 / G33-2; this source refresh ran no tests or probes and does not upgrade any observation to a runtime fact.

## Post-report artifact update

After this research task completed, the user authorized retrieving the official DuckDB
v2.0.0 Linux x86_64 Spatial artifact. It was downloaded from
`https://extensions.duckdb.org/v2.0.0/linux_amd64/spatial.duckdb_extension.gz` to
`/tmp/opencode/sda-rs-plan13-geography/` and decompressed as
`spatial.duckdb_extension`. Its SHA-256 is
`8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc`, matching the
Plan 8 record. This resolves the artifact-availability blocker only; extension identity,
load behavior, and all candidate semantics remain to be verified by the scoped tests.

## Post-report follow-up: spheroidal primitives and local projections

At the user's direction, the available geography and spheroidal operations were
checked as candidates for `STContains`/`STWithin`:

- The community [`geography` extension function reference](https://github.com/paleolimbot/duckdb-geography/blob/main/docs/function-reference.md)
  describes S2 predicates (`s2_contains`, `s2_within`, `s2_intersects`, and related
  functions). The [official community catalog entry](https://duckdb.org/community_extensions/extensions/geography)
  characterizes the extension as global processing **on the sphere**. Its `s2_area`
  documentation explicitly uses a perfect sphere; S2 containment is therefore not an
  ellipsoidal SQL Server predicate.
- DuckDB Spatial's [2.0 function reference](https://duckdb.org/docs/2.0/core_extensions/spatial/functions.html)
  documents `ST_Area_Spheroid`, `ST_Length_Spheroid`, `ST_Perimeter_Spheroid`,
  `ST_Distance_Spheroid(POINT_2D, POINT_2D)`, and
  `ST_DWithin_Spheroid(POINT_2D, POINT_2D, DOUBLE)`. The distance/dwithin signatures
  are point-to-point; the documentation does not describe a spheroidal point-to-line or
  point-in-polygon predicate. These measurement/proximity primitives do not by themselves
  establish `STContains` or `STWithin`.
- A local projection remains a **research candidate**, not an approved approximation.
  Projecting a bounded neighborhood may reduce distortion, but straight projected
  segments do not automatically reproduce SQL Server's ellipsoidal edges. To satisfy the
  unchanged exact-compatibility bar, a future candidate would need a specified bounded
  domain, a rigorous projected/geodesic boundary error bound, a runtime guard that rejects
  classifications within that error margin, and adversarial evidence that every admitted
  result matches the documented SQL Server contract. `ST_DWithin_Spheroid` is point-only;
  using it on sampled vertices is not a nearest-distance proof for an entire boundary.
  If only an approximate answer can be supported, keep the CLR methods rejected; do not
  label the result geography-compatible.

This follow-up does not change any verdict. No projection or spheroidal-predicate probe
was run; the extension-backed task test must first confirm which documented primitives
are present on the pinned binary.

## 15. Pinned synthetic probes (2026-11-22; Task 14.4.1)

### 15.1 Setup and commands

The local artifact was verified before probing:

```text
$ sha256sum /tmp/opencode/sda-rs-plan13-geography/spatial.duckdb_extension
8ec99d875355200316c743ede176e2cb50d2b712d2fa192ccbf19cc245a8d0bc  .../spatial.duckdb_extension
```

This matches the deployment pin. The artifact loaded with autoload/autoinstall
disabled; `duckdb_extensions()` reported `(spatial, a3dbcd1781)`. The repository's
locked client is `duckdb = "=1.20000.2"` (bundled engine v2.0.0). The contract test
also verifies the extension hash itself.

The exact prescribed command was attempted with a task-local target directory, but
the `/tmp` filesystem filled while compiling bundled DuckDB (`ar: ... No space left
on device`). It was rerun serially using the existing repository build cache:

```sh
SDA_DUCKDB_SPATIAL_EXTENSION=/tmp/opencode/sda-rs-plan13-geography/spatial.duckdb_extension CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/home/andrew/workspace/sda-rs/target cargo test --offline --features duckdb --test duckdb_geography_contract -- --nocapture
```

Result: 7 passed, 0 failed. The test observes boundary-exclusive planar
`ST_Contains` for one edge point and one vertex, interior/exterior planar
predicates, SQL NULL propagation, CRS mismatch bind errors, one planar
intersection/buffer, spheroidal point-pair distance, planar area, and limited WKT.
It also asserts that all six transpiler method spellings remain rejected. It does
**not** by itself establish the candidate-specific area guards, independent area
accuracy, or full WKT matrix.

To extend observations without editing repository code/tests or manifests, a
temporary standalone runner was created under
`/tmp/opencode/sda-rs-plan14-geography/` and run offline against the same artifact.
Its manifest pins the same Rust crate. Reproducer:

```sh
SDA_DUCKDB_SPATIAL_EXTENSION=/tmp/opencode/sda-rs-plan13-geography/spatial.duckdb_extension CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/home/andrew/workspace/sda-rs/target cargo run --offline --manifest-path /tmp/opencode/sda-rs-plan14-geography/Cargo.toml
```

The runner loads the artifact by absolute path with autoload/autoinstall disabled,
sets `geometry_always_xy=false`, and prints each query result or error. It exercised
asymmetric 20°×2° polygons near 60°N in both coordinate orders, a mid-latitude
rectangle, triangle, reversed ring, hole, invalid self-crossing polygon, Z/M
polygons, NULL/empty, `ST_CRS`, guard-function availability, and WKT serialization
for seven linear geometry classes plus ZM, collection, empty, and NULL. No SQL Server
oracle or independent GeographicLib executable/library was available locally;
therefore no numeric tolerance or area-accuracy claim is made.

### 15.2 Results and method verdicts

| Method | Pinned synthetic observation | Evidence verdict |
|---|---|---|
| `STContains` | Existing contract test: planar interior `true`, exterior `false`, tested edge and vertex `false`; NULL propagation; CRS mismatch errors at bind time. | **No-go.** These are planar observations, not SQL Server geography semantics; SQL Server's boundary rule remains undocumented in reviewed sources and mismatch behavior differs. |
| `STWithin` | Existing contract test: planar interior `true`, exterior `false`; NULL propagation; CRS mismatch errors at bind time. No synthetic boundary classification for this method was directly recorded. | **No-go.** Planar operation and CRS mismatch error do not establish SQL Server geography parity. Boundary/invalid/empty semantics remain unqualified. |
| `STIntersection` | Existing test: crossing polygon/line returns `LINESTRING`; disjoint points return `POINT EMPTY`; NULL propagates; CRS mismatch errors. | **No-go.** These characterize planar overlay only; no ellipsoidal, FullGlobe, >hemisphere, or arc behavior established. |
| `STBuffer` | Existing test: planar point buffer at distance 1 returns polygon reaching coordinate scale ±1; NULL propagates. | **No-go.** Coordinate-unit planar buffer does not implement documented geography meters/tolerance/negative-buffer semantics. |
| `STArea()` | `ST_Area_Spheroid` returned `489981499011.9553` for the explicit `POLYGON((-10 59,10 59,10 61,-10 61,-10 59))` with EPSG:4326 and `geometry_always_xy=false`; transposed coordinates returned `245493000698.74805`. A 10°×2° polygon near 40° returned `245902379922.79245`; triangle `12307884643.353682`; reversed ring returned the same as original; hole polygon `184564470319.76242`; invalid self-crossing polygon returned a number (`931211.9734077454`); Z and M polygons returned numbers; empty returned `0.0`; NULL returned NULL; `ST_CRS` returned `EPSG:4326`. `ST_HasZ`, `ST_HasM`, `ST_IsValid`, `ST_GeometryType`, and `ST_NInteriorRings` resolved in a guard expression; `error('guard')` raised `Invalid Input Error: guard`. | **Bounded inconclusive; candidate not established.** The asymmetric fixture distinguishes coordinate order numerically, and documented setting was exercised, but no independent reference was available to determine correctness/tolerance. Observed invalid, Z/M, hole and reversed-orientation values show why guards are needed, not that a valid guard expression is complete or SQL Server-equivalent. Keep method rejected. |
| `STAsText()` | `ST_AsText(ST_Force2D(...))` returned parse-shaped WKT with matching type/coordinates for POINT, LINESTRING, POLYGON, MULTIPOINT, MULTILINESTRING, MULTIPOLYGON, and GEOMETRYCOLLECTION; observed POINT ZM dropped to `POINT (1 2)`; empty yielded `POINT EMPTY`; NULL yielded NULL. | **Bounded semantic candidate only; not parity.** The sampled serialized output is consistent with 2D WKT and coordinate preservation, but the runner did not parse every output back independently; no SQL Server textual oracle exists, formatting parity is explicitly not claimed, and CRS/error guards plus all dimensional combinations per geometry class were not exhausted. |

The scratch runner additionally observed NULL from planar `ST_Intersection(NULL, …)`
and `ST_Buffer(NULL, …)`. These do not add geography support. The required six-method
coverage is therefore a matrix of observed candidate operations and specific gaps,
not a claim that every proposed adversarial case passed. In particular, no synthetic
runtime operation can resolve undocumented SQL Server predicate boundaries, and the
area output has no independent reference/tolerance. The two conditional candidates
remain unapproved; all six methods remain rejected by the transpiler.

The first scratch build attempt exhausted `/tmp` space; after removing only that
task-created target directory, the offline runner reused the repository's existing
DuckDB build cache. No repository source, manifest, lockfile, or test was changed.
