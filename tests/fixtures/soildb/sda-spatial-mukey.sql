WITH geom_data (mukey, muname) AS (SELECT mukey, muname
                    FROM mapunit
                    WHERE mapunit.mukey IN (
                    SELECT DISTINCT mukey from SDA_Get_Mukey_from_intersection_with_WktWgs84('GEOMETRYCOLLECTION (POINT (0.5 0.5))')
                    ))

                    SELECT geom_data.mukey, geom_data.muname FROM geom_data
