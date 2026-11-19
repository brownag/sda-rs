endpoint <- Sys.getenv("SOILDB_SDA_URL")
if (!nzchar(endpoint)) {
  stop("SOILDB_SDA_URL must be set to the local /Tabular/post.rest endpoint")
}

suppressPackageStartupMessages(library(soilDB))
options(soilDB.sda.tabular.post.rest.url = endpoint)

probe <- suppressMessages(
  soilDB::SDA_query(
    "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'gpkg_contents'"
  )
)
if (!is.data.frame(probe) || !identical(as.character(probe$name), "gpkg_contents")) {
  stop(
    "soilDB::SDA_query did not return the local GeoPackage probe; ",
    "verify that soilDB honors soilDB.sda.tabular.post.rest.url"
  )
}

testthat::test_dir(
  "tests/soildb/testthat",
  reporter = "summary",
  stop_on_failure = TRUE
)
