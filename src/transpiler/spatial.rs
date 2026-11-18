use polyglot_sql::{
    expressions::{Expression, Function, Literal},
    traversal::{transform_all, ExpressionWalk},
    Dialect, DialectType,
};

use crate::spatial::SpatialCapability;

use super::TranspilerError;

const MUKEY_FUNCTION: &str = "SDA_Get_Mukey_from_intersection_with_WktWgs84";
const MUKEY_WM_FUNCTION: &str = "SDA_Get_Mukey_from_intersection_with_WktWm";
const SAPOLYGONKEY_FUNCTION: &str = "SDA_Get_Sapolygonkey_from_intersection_with_WktWgs84";
const SAPOLYGONKEY_WM_FUNCTION: &str = "SDA_Get_Sapolygonkey_from_intersection_with_WktWm";

#[derive(Clone)]
struct IntersectionTarget {
    function: &'static str,
    table: &'static str,
    key: &'static str,
    input_srid: i32,
    column: String,
    feature_srid: i32,
}

fn intersection_spec(name: &str) -> Option<(&'static str, &'static str, &'static str, i32)> {
    [
        (MUKEY_FUNCTION, "mupolygon", "mukey", 4326),
        (MUKEY_WM_FUNCTION, "mupolygon", "mukey", 3857),
        (SAPOLYGONKEY_FUNCTION, "sapolygon", "sapolygonkey", 4326),
        (SAPOLYGONKEY_WM_FUNCTION, "sapolygon", "sapolygonkey", 3857),
    ]
    .into_iter()
    .find(|(function, _, _, _)| function.eq_ignore_ascii_case(name))
}

pub(super) fn parser_compatibility(sql: &str) -> String {
    const NEEDLE: &str = "geometry::STGeomFromText";

    let mut output = String::with_capacity(sql.len());
    let mut index = 0;
    let mut in_string = false;
    while index < sql.len() {
        let bytes = sql.as_bytes();
        if bytes[index] == b'\'' {
            output.push('\'');
            index += 1;
            if in_string && bytes.get(index) == Some(&b'\'') {
                output.push('\'');
                index += 1;
            } else {
                in_string = !in_string;
            }
            continue;
        }
        let end = index + NEEDLE.len();
        let token_boundary = |byte: Option<&u8>| {
            !byte.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        };
        let is_call = sql
            .get(end..)
            .unwrap_or_default()
            .trim_start()
            .starts_with('(');
        if !in_string
            && sql
                .get(index..end)
                .is_some_and(|part| part.eq_ignore_ascii_case(NEEDLE))
            && token_boundary(index.checked_sub(1).and_then(|i| bytes.get(i)))
            && token_boundary(bytes.get(end))
            && is_call
        {
            output.push_str("geometry.STGeomFromText");
            index = end;
        } else {
            let character = sql[index..].chars().next().unwrap();
            output.push(character);
            index += character.len_utf8();
        }
    }
    output
}

pub(super) fn rewrite(
    statements: Vec<Expression>,
    capability: &SpatialCapability,
) -> Result<Vec<Expression>, TranspilerError> {
    let mut output = Vec::with_capacity(statements.len());
    for statement in statements {
        let spatial_statement = statement.dfs().any(is_spatial_expression);
        let geometry_column = if spatial_statement {
            capability.effective_geometry_column().ok_or_else(|| {
                TranspilerError::Rejected(
                    "spatial rewrite requires GeoPackage geometry metadata or a geometry column override"
                        .into(),
                )
            })?
        } else {
            "geom"
        };
        let mut targets = Vec::new();
        for node in statement.dfs() {
            if let Expression::Function(function) = node {
                if let Some((name, table, key, input_srid)) = intersection_spec(&function.name) {
                    if targets
                        .iter()
                        .any(|target: &IntersectionTarget| target.function == name)
                    {
                        continue;
                    }
                    let metadata = capability.geometry_metadata_for_table(table);
                    if metadata.is_none() && capability.geometry_override().is_none() {
                        return Err(TranspilerError::Rejected(format!(
                            "{name} TVF requires geometry metadata for {table}"
                        )));
                    }
                    targets.push(IntersectionTarget {
                        function: name,
                        table,
                        key,
                        input_srid,
                        column: capability
                            .geometry_override()
                            .or_else(|| metadata.map(|metadata| metadata.geometry_column.as_str()))
                            .or_else(|| capability.effective_geometry_column())
                            .unwrap_or(geometry_column)
                            .to_owned(),
                        feature_srid: metadata.map_or(4326, |metadata| metadata.srs_id),
                    });
                }
            }
        }
        let statement = transform_all(statement, &|node| expand_tables(node, &targets))
            .map_err(classify_error)?;
        output.push(
            transform_all(statement, &|node| rewrite_node(node, capability))
                .map_err(classify_error)?,
        );
    }
    Ok(output)
}

/// Unsupported input shapes are validation failures; everything else is a generation failure.
fn classify_error(error: polyglot_sql::Error) -> TranspilerError {
    match error {
        polyglot_sql::Error::InvalidInput(message) => TranspilerError::Rejected(message),
        other => TranspilerError::Generate(other),
    }
}

fn require_geom_from_text_arity(arg_count: usize) -> Result<(), polyglot_sql::Error> {
    if arg_count != 2 {
        return Err(polyglot_sql::Error::InvalidInput(
            "geometry::STGeomFromText requires WKT and SRID arguments".into(),
        ));
    }
    Ok(())
}

fn expand_tables(
    node: Expression,
    targets: &[IntersectionTarget],
) -> Result<Expression, polyglot_sql::Error> {
    let Expression::Select(mut select) = node else {
        return Ok(node);
    };
    if select.from.is_none() && select.expressions.len() == 1 {
        if matches!(&select.expressions[0], Expression::Function(function) if intersection_spec(&function.name).is_some())
        {
            let derived = expand_source(select.expressions[0].clone(), targets)?;
            let mut parsed = Dialect::get(DialectType::TSQL).parse("SELECT * FROM placeholder")?;
            let Expression::Select(mut template) = parsed.remove(0) else {
                unreachable!()
            };
            select.expressions = template.expressions;
            let mut from = template.from.take().unwrap();
            from.expressions[0] = derived;
            select.from = Some(from);
        }
    }
    if let Some(from) = &mut select.from {
        for source in &mut from.expressions {
            *source = expand_source(source.clone(), targets)?;
        }
    }
    for join in &mut select.joins {
        join.this = expand_source(join.this.clone(), targets)?;
    }
    Ok(Expression::Select(select))
}

fn expand_source(
    source: Expression,
    targets: &[IntersectionTarget],
) -> Result<Expression, polyglot_sql::Error> {
    let (function, alias) = match &source {
        Expression::Function(function) if intersection_spec(&function.name).is_some() => {
            (function, None)
        }
        Expression::Alias(alias) => match &alias.this {
            Expression::Function(function)
                if intersection_spec(&function.name).is_some()
                    && alias.column_aliases.is_empty() =>
            {
                (function, Some(alias.alias.clone()))
            }
            _ => return Ok(source),
        },
        _ => return Ok(source),
    };
    let target = targets
        .iter()
        .find(|target| target.function.eq_ignore_ascii_case(&function.name))
        .ok_or_else(|| {
            polyglot_sql::Error::InvalidInput("missing intersection target metadata".into())
        })?;
    if function.args.len() != 1
        || !matches!(&function.args[0], Expression::Literal(literal) if matches!(literal.as_ref(), Literal::String(_)))
    {
        return Err(polyglot_sql::Error::InvalidInput(format!("{} TVF requires one literal WKT argument; correlated and other expressions are unsupported", target.function)));
    }
    let argument = function.args[0].clone();
    let predicate = if target.function == MUKEY_FUNCTION {
        format!("ST_Intersects(ST_Transform(GeomFromGPB(mp.__geom__), {}), GeomFromText(__wkt__, 4326))", target.feature_srid)
    } else {
        format!(
            "ST_Intersects(ST_Transform(GeomFromGPB(mp.__geom__), {}), GeomFromText(__wkt__, {}))",
            target.input_srid, target.input_srid
        )
    };
    let filter = if target.table == "mupolygon" {
        "mp.areasymbol <> 'US' AND "
    } else {
        ""
    };
    let mut parsed = Dialect::get(DialectType::TSQL).parse(&format!(
        "SELECT * FROM (SELECT DISTINCT mp.{} FROM {} AS mp WHERE {}{}) AS __sda",
        target.key, target.table, filter, predicate
    ))?;
    let Expression::Select(mut outer) = parsed.remove(0) else {
        unreachable!()
    };
    let mut derived = outer.from.take().unwrap().expressions.remove(0);
    if let Expression::Subquery(subquery) = &mut derived {
        if let Some(alias) = alias {
            subquery.alias = Some(alias);
        }
    } else {
        return Err(polyglot_sql::Error::InvalidInput(
            "unexpected derived table AST".into(),
        ));
    }
    transform_all(derived, &|node| match node {
        Expression::Column(mut column) if column.name.name == "__geom__" => {
            column.name.name = target.column.clone();
            column.name.quoted = true;
            Ok(Expression::Column(column))
        }
        Expression::Column(column) if column.name.name == "__wkt__" => Ok(argument.clone()),
        other => Ok(other),
    })
}

fn is_spatial_expression(expression: &Expression) -> bool {
    match expression {
        Expression::Function(function) => intersection_spec(&function.name).is_some(),
        Expression::MethodCall(call) => {
            call.method.name.eq_ignore_ascii_case("STIntersects")
                || call.method.name.eq_ignore_ascii_case("STGeomFromText")
                || intersection_spec(&call.method.name).is_some()
        }
        _ => false,
    }
}

fn rewrite_node(
    node: Expression,
    capability: &SpatialCapability,
) -> Result<Expression, polyglot_sql::Error> {
    Ok(match node {
        Expression::MethodCall(call) if intersection_spec(&call.method.name).is_some() => {
            return Err(polyglot_sql::Error::InvalidInput(
                "qualified mukey TVF syntax is unsupported".into(),
            ));
        }
        Expression::Function(function) if intersection_spec(&function.name).is_some() => {
            return Err(polyglot_sql::Error::InvalidInput(
                "SDA_Get_Mukey_from_intersection_with_WktWgs84 is unsupported in this AST context"
                    .into(),
            ));
        }
        Expression::Function(mut function)
            if function.name.eq_ignore_ascii_case("STGeomFromText")
                && function.qualified_name.len() == 2
                && function.qualified_name[0]
                    .name
                    .eq_ignore_ascii_case("geometry")
                && function.qualified_name[1]
                    .name
                    .eq_ignore_ascii_case("STGeomFromText") =>
        {
            require_geom_from_text_arity(function.args.len())?;
            function.name = "GeomFromText".into();
            function.qualified_name.clear();
            Expression::Function(function)
        }
        Expression::MethodCall(call) if call.method.name.eq_ignore_ascii_case("STIntersects") => {
            if call.args.len() != 1 {
                return Err(polyglot_sql::Error::InvalidInput(
                    ".STIntersects requires exactly one geometry argument".into(),
                ));
            }
            let wrap = |expression| match &expression {
                Expression::Column(column) if capability.is_geometry_column(&column.name.name) => {
                    Expression::Function(Box::new(Function::new("GeomFromGPB", vec![expression])))
                }
                _ => expression,
            };
            Expression::Function(Box::new(Function::new(
                "ST_Intersects",
                vec![wrap(call.this), wrap(call.args.into_iter().next().unwrap())],
            )))
        }
        Expression::MethodCall(call) if call.method.name.eq_ignore_ascii_case("STGeomFromText") => {
            require_geom_from_text_arity(call.args.len())?;
            Expression::Function(Box::new(Function::new("GeomFromText", call.args)))
        }
        other => other,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial::initialize;
    use polyglot_sql::{Dialect, DialectType};

    fn capability() -> SpatialCapability {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection.execute_batch(
            "CREATE TABLE gpkg_geometry_columns (table_name TEXT, column_name TEXT, srs_id INTEGER);
             INSERT INTO gpkg_geometry_columns VALUES ('soil_map', 'soil_shape', 26910);
             INSERT INTO gpkg_geometry_columns VALUES ('mupolygon', 'shape_dynamic', 4326);",
        ).unwrap();
        initialize(&connection, None, None).unwrap()
    }

    #[test]
    fn rewrites_tvf_in_from_and_in_subquery() {
        for sql in [
            "SELECT mukey FROM mapunit WHERE mukey IN (SELECT * FROM SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)'))",
            "SELECT mukey FROM mapunit WHERE mukey IN (SELECT SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)'))",
            "SELECT k.mukey FROM SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)') AS k",
            "SELECT k.mukey FROM mapunit m JOIN SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)') k ON k.mukey = m.mukey",
        ] {
            let output = crate::transpiler::transpile_with_spatial(sql, Some(&capability())).unwrap();
            let generated = &output[0];
            assert!(generated.contains("shape_dynamic"), "{generated}");
            assert!(generated.contains("GEOMFROMGPB"), "{generated}");
            assert!(generated.contains("ST_TRANSFORM"), "{generated}");
            assert!(generated.contains("areasymbol <> 'US'"), "{generated}");
            assert!(!generated.contains(MUKEY_FUNCTION), "{generated}");
        }
    }

    #[test]
    fn folds_aoi_and_wraps_geopackage_geometry() {
        let sql = "DECLARE @aoi geometry; SET @aoi = geometry::STGeomFromText('POINT(0 0)', 4326); SELECT mukey FROM mupolygon WHERE shape_dynamic.STIntersects(@aoi) = 1";
        let output = crate::transpiler::transpile_with_spatial(sql, Some(&capability())).unwrap();
        assert!(
            output[0].contains("GEOMFROMGPB(shape_dynamic)"),
            "{output:?}"
        );
        let sql = "SELECT mp.shape_dynamic.STIntersects(geometry::STGeomFromText('POINT(0 0)', 4326)) FROM mupolygon mp";
        let output = crate::transpiler::transpile_with_spatial(sql, Some(&capability())).unwrap();
        assert!(
            output[0].contains("GEOMFROMGPB(mp.shape_dynamic)"),
            "{output:?}"
        );
        assert!(
            output[0].contains("GEOMFROMTEXT('POINT(0 0)', 4326)"),
            "{output:?}"
        );
        let sql = "DECLARE @aoi geometry; SET @aoi = geometry::STGeomFromText('POINT(0 0)', 4326); SELECT @aoi.STIntersects(shape_dynamic) FROM mupolygon";
        let output = crate::transpiler::transpile_with_spatial(sql, Some(&capability())).unwrap();
        assert!(!output[0].contains("@aoi"), "{output:?}");
        assert!(
            output[0].contains("GEOMFROMGPB(shape_dynamic)"),
            "{output:?}"
        );
    }

    #[test]
    fn uses_override_and_rejects_unsupported_arguments() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        let capability = initialize(&connection, None, Some("override shape")).unwrap();
        let query = "SELECT * FROM SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)')";
        let output = crate::transpiler::transpile_with_spatial(query, Some(&capability)).unwrap();
        assert!(output[0].contains("mp.\"override shape\""), "{output:?}");
        for query in [
            "SELECT * FROM SDA_Get_Mukey_from_intersection_with_WktWgs84(m.wkt)",
            "SELECT * FROM SDA_Get_Mukey_from_intersection_with_WktWgs84()",
        ] {
            assert!(
                crate::transpiler::transpile_with_spatial(query, Some(&capability))
                    .unwrap_err()
                    .to_string()
                    .contains("requires one literal WKT")
            );
        }
    }

    #[test]
    fn mukey_requires_mupolygon_metadata_or_explicit_override() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE gpkg_geometry_columns (table_name TEXT, column_name TEXT, srs_id INTEGER);
                 INSERT INTO gpkg_geometry_columns VALUES ('soil_map', 'soil_shape', 26910);",
            )
            .unwrap();
        let capability = initialize(&connection, None, None).unwrap();
        let query = "SELECT * FROM SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)')";

        let error =
            crate::transpiler::transpile_with_spatial(query, Some(&capability)).unwrap_err();
        assert!(matches!(&error, TranspilerError::Rejected(_)), "{error:?}");
        assert!(
            error
                .to_string()
                .contains("requires geometry metadata for mupolygon"),
            "{error}"
        );

        let capability = initialize(&connection, None, Some("geom_override")).unwrap();
        let output = crate::transpiler::transpile_with_spatial(query, Some(&capability)).unwrap();
        let generated = output[0].to_ascii_uppercase();
        assert!(generated.contains("MP.\"GEOM_OVERRIDE\""), "{generated}");
        assert!(
            generated.contains("ST_TRANSFORM(GEOMFROMGPB(MP.\"GEOM_OVERRIDE\"), 4326)"),
            "{generated}"
        );
        assert!(!generated.contains("SOIL_SHAPE"), "{generated}");
    }

    #[test]
    fn mukey_override_column_uses_mupolygon_metadata_srid() {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE gpkg_geometry_columns (table_name TEXT, column_name TEXT, srs_id INTEGER);
                 INSERT INTO gpkg_geometry_columns VALUES ('soil_map', 'soil_shape', 26910);
                 INSERT INTO gpkg_geometry_columns VALUES ('mupolygon', 'geom_custom', 3857);",
            )
            .unwrap();
        let capability = initialize(&connection, None, Some("geom_override")).unwrap();
        let output = crate::transpiler::transpile_with_spatial(
            "SELECT * FROM SDA_Get_Mukey_from_intersection_with_WktWgs84('POINT(0 0)')",
            Some(&capability),
        )
        .unwrap();
        let generated = output[0].to_ascii_uppercase();
        assert!(generated.contains("MP.\"GEOM_OVERRIDE\""), "{generated}");
        assert!(
            generated.contains("ST_TRANSFORM(GEOMFROMGPB(MP.\"GEOM_OVERRIDE\"), 3857)"),
            "{generated}"
        );
        assert!(!generated.contains("GEOM_CUSTOM"), "{generated}");
        assert!(!generated.contains("SOIL_SHAPE"), "{generated}");
    }

    #[test]
    fn parser_ast_rewrites_qualified_geometry_constructor_and_intersects_without_runtime_extension()
    {
        let ast = Dialect::get(DialectType::TSQL)
            .parse(&parser_compatibility(
                "SELECT geom.STIntersects(geometry::STGeomFromText('POINT(0 0)', 4326))",
            ))
            .unwrap();
        let output = rewrite(ast, &capability()).unwrap();
        let sql = Dialect::get(DialectType::TSQL)
            .generate(&output[0])
            .unwrap();
        assert!(sql.to_ascii_lowercase().contains("st_intersects"), "{sql}");
        assert!(sql.to_ascii_lowercase().contains("geomfromtext"), "{sql}");
        assert_eq!(capability().effective_geometry_column(), Some("soil_shape"));
        assert!(capability().is_geometry_column("shape_dynamic"));
    }

    #[test]
    fn rewrites_geometry_constructor_in_aoi_but_preserves_string_and_non_spatial_calls() {
        let output = crate::transpiler::transpile_with_spatial(
            "DECLARE @aoi geometry; SET @aoi = geometry::STGeomFromText('POINT(0 0)', 4326); SELECT 'geometry::STGeomFromText' AS text_value, STGeomFromText('POINT(1 1)', 4326) AS ordinary_call",
            Some(&capability()),
        )
        .unwrap();
        assert!(
            output[0].contains("'geometry::STGeomFromText'"),
            "{output:?}"
        );
        assert!(
            output[0].contains("STGEOMFROMTEXT('POINT(1 1)', 4326)"),
            "{output:?}"
        );
    }

    #[test]
    fn parser_compatibility_rewrites_only_constructor_syntax_outside_literals() {
        assert_eq!(
            parser_compatibility(
                "SELECT geometry::STGeomFromText('geometry::STGeomFromText', 4326), 'x'' geometry::STGeomFromText', xgeometry::STGeomFromText, geometry::STGeomFromText"
            ),
            "SELECT geometry.STGeomFromText('geometry::STGeomFromText', 4326), 'x'' geometry::STGeomFromText', xgeometry::STGeomFromText, geometry::STGeomFromText"
        );
    }

    #[test]
    fn does_not_lower_unqualified_function_with_same_terminal_name() {
        let ast = Dialect::get(DialectType::TSQL)
            .parse("SELECT STGeomFromText('POINT(0 0)', 4326)")
            .unwrap();
        let output = rewrite(ast, &capability()).unwrap();
        let sql = Dialect::get(DialectType::TSQL)
            .generate(&output[0])
            .unwrap();
        assert!(sql.contains("STGEOMFROMTEXT("), "{sql}");
        assert!(!sql.contains(" GEOMFROMTEXT("), "{sql}");
    }

    #[test]
    fn unsupported_mukey_function_is_explicitly_rejected() {
        let ast = Dialect::get(DialectType::TSQL)
            .parse(&format!("SELECT 1 + {MUKEY_FUNCTION}('POINT(0 0)')"))
            .unwrap();
        let error = rewrite(ast, &capability()).unwrap_err();
        assert!(error
            .to_string()
            .contains("unsupported in this AST context"));
    }

    #[test]
    fn validation_failures_are_rejected_not_generate() {
        let ast = Dialect::get(DialectType::TSQL)
            .parse(&format!("SELECT * FROM {MUKEY_FUNCTION}(m.wkt)"))
            .unwrap();
        let error = rewrite(ast, &capability()).unwrap_err();
        assert!(matches!(error, TranspilerError::Rejected(_)), "{error:?}");
        let ast = Dialect::get(DialectType::TSQL)
            .parse("SELECT geom.STIntersects(geometry.STGeomFromText('POINT(0 0)'))")
            .unwrap();
        let error = rewrite(ast, &capability()).unwrap_err();
        assert!(matches!(error, TranspilerError::Rejected(_)), "{error:?}");
    }
}
