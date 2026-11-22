# DuckDB private GeoPackage loader audit

**Task:** Plan 14, task 14.2.2
**Audit date:** 2026-11-22
**Scope:** Revalidation of the private Rust-side synthetic GeoPackage loader and result contract. No loader implementation, fixtures, dependencies, deployment, or request-path integration changed.

## Summary

The loader has bounded structural validation and a focused set of positive and negative synthetic tests. It canonicalizes an internally supplied path, requires a regular file, opens SQLite read-only, checks a unique geometry metadata row and CRS/table registrations, validates GeoPackageBinary and WKB structure/SRID/dimensions, then copies SQLite scalar/blob values into an in-memory DuckDB table. A module test confirms a synthetic geometry survives SDA JSON serialization and the source file remains unchanged.

This evidence does not establish caller authorization or a production provisioning lifecycle: `TrustedGeoPackagePath` and `load_geopackage` are private, marked `allow(dead_code)`, and not wired to deployment or HTTP. Canonicalization resolves the path at setup time but does not by itself establish who may provide that path, prevent later filesystem replacement/races, or prove the same object remains open after canonicalization. No symlink-specific test was found. The no-mutex SQLite flag is used with a local connection; concurrent sharing behavior is not tested.

## Evidence map

| Concern | Source behavior | Existing evidence and limits |
|---|---|---|
| Path capability and provenance | `TrustedGeoPackagePath::from_internal_setup` canonicalizes and checks `is_file`; private tuple capability is required by `load_geopackage`. `load_geopackage` opens the canonical path with `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX`. | `rejects_untrusted_or_inconsistent_geo_packages` checks a nonexistent path is rejected. `loads_synthetic_gpkg_read_only_and_preserves_validated_values` verifies the fixture is unchanged and a write through a read-only connection fails. No symlink, replacement race, permission, or production caller/provisioning test. |
| Metadata ambiguity and mismatch | `load_geopackage_connection` requires exactly one row from `gpkg_geometry_columns`; checks z/m values, identifier syntax, one matching `gpkg_spatial_ref_sys` row, one matching feature-table registration in `gpkg_contents`, and presence of the declared geometry column. SQL metadata values are bound where used as values; table/column identifiers are validated and quoted. | `rejects_untrusted_or_inconsistent_geo_packages` tests an SRID mismatch. No tests create multiple geometry metadata rows, duplicate/missing CRS registrations, table-registration mismatches, invalid z/m values, or metadata identifier adversarial cases. These branches are implemented but their rejection behavior is not directly asserted by the current tests. |
| GeoPackageBinary and WKB validation | `gpkg_geometry::decode` parses the GeoPackage header; `validate_geometry_value` checks blob-or-NULL, embedded SRID, WKB, empty flag, and declared z/m compatibility. `validate_wkb`/`parse_wkb_geometry` check byte order, supported type/dimensions, truncation, bounded nesting/count, finite ordinates, collection dimension consistency, and trailing bytes. | Tests cover embedded SRID mismatch, a one-byte-truncated Point payload, XYZ accepted with z metadata, dimensional mismatch rejected, and empty Point accepted. Structural checks are not full OGC geometry/topology validity checks. No suite of malformed/truncated GeoPackage header variants, collection adversarial cases, M/ZM cases, or every metadata/payload ambiguity is present. |
| NULL, empty, dimensional values | NULL geometry is returned as NULL; empty flag must agree with the WKB coordinate-presence result; dimension tags are compared with metadata (z/m value 2 is accepted as “may vary” by the predicate). | `validates_null_empty_and_dimensional_geometries` asserts NULL, empty 2D Point, valid XYZ, and XYZ/2D-metadata mismatch. ZM and mixed collection dimensions are not specifically exercised in this test. |
| Copying and output | `load_geopackage_connection` materializes SQLite values into `ResultSet`. `copy_validated_values_to_duckdb` infers one DuckDB scalar type per column, rejects mixed SQLite types and row-width mismatch, inserts bound values into an in-memory temporary table with external access/autoload disabled, reads rows back in rowid order, and retains original column names. | `loads_synthetic_gpkg_read_only_and_preserves_validated_values` checks one row's integer, geometry blob, and text values plus exact JSON output. It also checks the source bytes are unchanged. This is exact equality for that row, not broad type/large-value/ordering coverage. |
| Generic result serialization | Generic DuckDB `map_value` rejects unvalidated BLOB/GEOMETRY outputs; validated loader blobs use a distinct copying path. Serializer consumes the resulting `ResultSet`. | `duckdb_result_contract::supported_scalars_preserve_values_and_sda_serialization` verifies scalar values and JSON/column-name/metadata output. `gpkg_metadata_is_readable_but_blob_payloads_are_rejected` verifies arbitrary BLOB rejection. The result contract does not invoke the private loader. |
| Resource and lifecycle boundary | Loader materializes the whole SQLite result in Rust and then copies it into DuckDB; no row, blob, total-byte, memory, or deadline bounds appear in this path. | No dedicated limits, cancellation, concurrent use, deployment lifecycle, or resource exhaustion test. Passing the synthetic tests is not evidence of bounded resource use. |

## Test results

Commands were run serially, offline, as required because Cargo's build cache is shared with task 14.2.1.

1. `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb executor::duckdb::tests` — **passed**: 4 unit tests passed, 0 failed; remaining test binaries were invoked with zero matching tests.
   - `geometry_values_are_rejected_without_geo_package_validation`
   - `rejects_untrusted_or_inconsistent_geo_packages`
   - `loads_synthetic_gpkg_read_only_and_preserves_validated_values`
   - `validates_null_empty_and_dimensional_geometries`
2. `CARGO_NET_OFFLINE=true cargo test --offline --features duckdb --test duckdb_result_contract` — **passed**: 4 tests passed, 0 failed.
   - `result_contract_remains_request_path_independent`
   - `supported_scalars_preserve_values_and_sda_serialization`
   - `gpkg_metadata_is_readable_but_blob_payloads_are_rejected`
   - `unsupported_values_fail_instead_of_being_coerced`

These tests used existing synthetic fixtures/tests only. No downloads, network access, or user/Iowa data were used.

## Proposed follow-ups requiring separate approval

No source or test changes were made. To directly establish current gaps, consider approval for synthetic-only tests/fixtures covering: duplicate geometry metadata rows; missing/duplicate CRS and mismatched feature registration; invalid dimensions/identifiers; malformed/truncated GeoPackageBinary header; truncated WKB for each relevant structure and trailing bytes; ZM and mixed-dimensional collections; symlink/canonicalization behavior; and multiple rows/SQLite scalar types copied and serialized exactly. If the loader is later wired into deployment, separately specify and test trusted-path provisioning, filesystem race/symlink policy, lifecycle, and resource limits. These are proposed work items, not claims that current behavior fails.

## Boundary

The loader remains private qualification code and is not an HTTP/request-selected file reader. Results establish only the tested synthetic observations and source-level checks described above; they do not establish production trust provisioning, exhaustive GeoPackage validity, geometry topology validity, or resource safety.
