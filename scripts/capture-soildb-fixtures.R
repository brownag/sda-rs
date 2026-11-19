#!/usr/bin/env Rscript

args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 1L) {
  stop("usage: Rscript scripts/capture-soildb-fixtures.R /path/to/soilDB")
}
soilDB_path <- normalizePath(args[[1]], mustWork = TRUE)
script_arg <- sub("^--file=", "", grep("^--file=", commandArgs(), value = TRUE))
repo_root <- normalizePath(file.path(dirname(normalizePath(script_arg)), ".."))
fixture_dir <- file.path(repo_root, "tests", "fixtures", "soildb")

if (!requireNamespace("pkgload", quietly = TRUE) ||
    !requireNamespace("sf", quietly = TRUE) ||
    !requireNamespace("wk", quietly = TRUE) ||
    !requireNamespace("jsonlite", quietly = TRUE)) {
  stop("capture requires R packages pkgload, sf, wk, and jsonlite")
}

pkgload::load_all(soilDB_path, quiet = TRUE)
point <- sf::st_sfc(sf::st_point(c(0.5, 0.5)), crs = 4326)

queries <- list(
  list(
    file = "sda-spatial-mukey.sql",
    call = "SDA_spatialQuery(point, what = 'mukey', query_string = TRUE)",
    sql = soilDB::SDA_spatialQuery(point, what = "mukey", query_string = TRUE),
    columns = c("mukey", "muname"),
    key_rows = list(c("1857266", "Inside map unit")),
    prerequisites = "SpatiaLite and deterministic mupolygon/mapunit tables"
  ),
  list(
    file = "sda-spatial-areasymbol.sql",
    call = "SDA_spatialQuery(point, what = 'areasymbol', query_string = TRUE)",
    sql = soilDB::SDA_spatialQuery(point, what = "areasymbol", query_string = TRUE),
    columns = "areasymbol",
    key_rows = list("IA001"),
    prerequisites = "SpatiaLite and deterministic sapolygon table"
  ),
  list(
    file = "get-sda-muaggatt.sql",
    call = "get_SDA_muaggatt(mukeys = 1857266, query_string = TRUE)",
    sql = soilDB::get_SDA_muaggatt(mukeys = 1857266, query_string = TRUE),
    columns = c("mukey", "mukey", "representative_value"),
    key_rows = list(c("1857266", "1857266", "fixture value")),
    prerequisites = "Deterministic legend, mapunit, and muaggatt tables"
  )
)

for (fixture in queries) {
  normalized_sql <- gsub("[ \\t]+(?=\\n|$)", "", fixture$sql, perl = TRUE)
  writeLines(normalized_sql, file.path(fixture_dir, fixture$file), useBytes = TRUE)
}

commit <- system2(
  "git",
  c("-C", shQuote(soilDB_path), "rev-parse", "HEAD"),
  stdout = TRUE
)
if (!identical(attr(commit, "status"), NULL)) {
  stop("could not determine soilDB git commit")
}
version <- as.character(utils::packageVersion("soilDB"))
manifest <- list(
  soilDB = list(version = version, commit = commit[[1L]]),
  format = "JSON+COLUMNNAME+METADATA",
  fixtures = lapply(queries, function(fixture) {
    list(
      file = fixture$file,
      source_call = fixture$call,
      sql = fixture$sql,
      transpiled_sql_file = sub("\\.sql$", ".sqlite.sql", fixture$file),
      expected_columns = fixture$columns,
      expected_key_rows = fixture$key_rows,
      prerequisites = fixture$prerequisites
    )
  })
)
jsonlite::write_json(
  manifest,
  file.path(fixture_dir, "manifest.json"),
  pretty = TRUE,
  auto_unbox = TRUE,
  null = "null"
)
message("Captured ", length(queries), " soilDB query fixtures for soilDB ", version, " (", commit[[1L]], ")")
