WITH geom_data (areasymbol) AS (SELECT areasymbol
                  FROM sapolygon
                  WHERE sapolygonkey IN (
                  SELECT DISTINCT sapolygonkey from SDA_Get_Sapolygonkey_from_intersection_with_WktWgs84('GEOMETRYCOLLECTION (POINT (0.5 0.5))')
                  ))

                  SELECT geom_data.areasymbol FROM geom_data
