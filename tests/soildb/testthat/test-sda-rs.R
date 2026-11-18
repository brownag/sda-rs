test_that("SDA_query reads the stable Iowa survey row", {
  result <- suppressMessages(
    soilDB::SDA_query(
      "SELECT areasymbol FROM legend WHERE areasymbol = 'IA001'"
    )
  )

  expect_s3_class(result, "data.frame")
  expect_true("areasymbol" %in% names(result))
  expect_identical(as.character(result$areasymbol), "IA001")
})

test_that("SDA_spatialQuery finds the known Iowa mukey", {
  geometry <- suppressMessages(
    soilDB::SDA_query(
      "SELECT * FROM SDA_Get_MupolygonWktWgs84_from_Mukey(1857266)"
    )
  )
  expect_s3_class(geometry, "data.frame")
  expect_true("MupolygonWktWgs84" %in% names(geometry))
  valid_wkt <- geometry$MupolygonWktWgs84[!is.na(geometry$MupolygonWktWgs84)]
  expect_gt(length(valid_wkt), 0)

  polygon <- sf::st_as_sfc(valid_wkt[[1]], crs = 4326)
  result <- suppressMessages(
    soilDB::SDA_spatialQuery(polygon, what = "mukey", geomAcres = FALSE)
  )

  expect_s3_class(result, "data.frame")
  expect_true("mukey" %in% names(result))
  expect_true("1857266" %in% as.character(result$mukey))
})

test_that("get_SDA_muaggatt returns the known Iowa mukey", {
  result <- suppressMessages(
    soilDB::get_SDA_muaggatt(mukeys = 1857266)
  )

  expect_s3_class(result, "data.frame")
  expect_gt(nrow(result), 0)
  expect_true("mukey" %in% names(result))
  expect_true("1857266" %in% as.character(result$mukey))
})
