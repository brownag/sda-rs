endpoint <- Sys.getenv("SOILDB_SDA_URL")
if (!nzchar(endpoint)) {
  stop("SOILDB_SDA_URL must be set to the local /Tabular/post.rest endpoint")
}

suppressPackageStartupMessages(library(soilDB))
options(soilDB.sda.tabular.post.rest.url = endpoint)

testthat::test_dir(
  "tests/soildb/testthat",
  reporter = "summary",
  stop_on_failure = TRUE
)
