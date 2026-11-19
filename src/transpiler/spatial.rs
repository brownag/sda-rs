use polyglot_sql::{
    expressions::{Expression, Function, Identifier, Literal},
    traversal::{transform_all, ExpressionWalk},
    Dialect, DialectType,
};

use crate::spatial::SpatialCapability;

use super::TranspilerError;

const MUKEY_FUNCTION: &str = "SDA_Get_Mukey_from_intersection_with_WktWgs84";
const MUKEY_WM_FUNCTION: &str = "SDA_Get_Mukey_from_intersection_with_WktWm";
const MUPOLYGONKEY_FUNCTION: &str = "SDA_Get_Mupolygonkey_from_intersection_with_WktWgs84";
const MUPOLYGONKEY_WM_FUNCTION: &str = "SDA_Get_Mupolygonkey_from_intersection_with_WktWm";
const SAPOLYGONKEY_FUNCTION: &str = "SDA_Get_Sapolygonkey_from_intersection_with_WktWgs84";
const SAPOLYGONKEY_WM_FUNCTION: &str = "SDA_Get_Sapolygonkey_from_intersection_with_WktWm";
const AREASYMBOL_FUNCTION: &str = "SDA_Get_Areasymbol_from_intersection_with_WktWgs84";
const AREASYMBOL_WM_FUNCTION: &str = "SDA_Get_Areasymbol_from_intersection_with_WktWm";
const MUPOINTKEY_FUNCTION: &str = "SDA_Get_Mupointkey_from_intersection_with_WktWgs84";
const MUPOINTKEY_WM_FUNCTION: &str = "SDA_Get_Mupointkey_from_intersection_with_WktWm";
const MULINEKEY_FUNCTION: &str = "SDA_Get_Mulinekey_from_intersection_with_WktWgs84";
const MULINEKEY_WM_FUNCTION: &str = "SDA_Get_Mulinekey_from_intersection_with_WktWm";
const FEATLINEKEY_FUNCTION: &str = "SDA_Get_Featlinekey_from_intersection_with_WktWgs84";
const FEATLINEKEY_WM_FUNCTION: &str = "SDA_Get_Featlinekey_from_intersection_with_WktWm";
const FEATPOINTKEY_FUNCTION: &str = "SDA_Get_Featpointkey_from_intersection_with_WktWgs84";
const FEATPOINTKEY_WM_FUNCTION: &str = "SDA_Get_Featpointkey_from_intersection_with_WktWm";
const SAPOLYGONKEY_STATUS_FUNCTION: &str =
    "SDA_Get_Sapolygonkey_from_intersection_with_sastatusmap_WktWgs84";
const SAPOLYGONKEY_STATUS_WM_FUNCTION: &str =
    "SDA_Get_Sapolygonkey_from_intersection_with_sastatusmap_WktWm";
const AREASYMBOL_WKT_84_FUNCTION: &str = "SDA_Get_AreasymbolWKTWgs84_from_Areasymbol";
const AREASYMBOL_WKT_WM_FUNCTION: &str = "SDA_Get_AreasymbolWKTWm_from_Areasymbol";
const MUPOLYGON_WKT_84_FUNCTION: &str = "SDA_Get_MupolygonWktWgs84_from_Mukey";
const MUPOLYGON_WKT_WM_FUNCTION: &str = "SDA_Get_MupolygonWktWm_from_Mukey";
const MUPOINT_WKT_84_FUNCTION: &str = "SDA_Get_MupointWktWgs84_from_Mukey";
const MUPOINT_WKT_WM_FUNCTION: &str = "SDA_Get_MupointWktWm_from_Mukey";
const MULINE_WKT_84_FUNCTION: &str = "SDA_Get_MulineWktWgs84_from_Mukey";
const MULINE_WKT_WM_FUNCTION: &str = "SDA_Get_MulineWktWm_from_Mukey";
const AREASYMBOL_TABLE_WKT_84_FUNCTION: &str = "SDA_Get_AreasymbolWktWgs84_from_AreasymbolTable";
const AREASYMBOL_TABLE_WKT_WM_FUNCTION: &str = "SDA_Get_AreasymbolWktWm_from_AreasymbolTable";
const MUPOLYGON_TABLE_WKT_84_FUNCTION: &str = "SDA_Get_MupolygonWktWgs84_from_MukeyTable";
const MUPOLYGON_TABLE_WKT_WM_FUNCTION: &str = "SDA_Get_MupolygonWktWm_from_MukeyTable";
const MUPOINT_TABLE_WKT_84_FUNCTION: &str = "SDA_Get_MupointWktWgs84_from_MukeyTable";
const MUPOINT_TABLE_WKT_WM_FUNCTION: &str = "SDA_Get_MupointWktWm_from_MukeyTable";
const MULINE_TABLE_WKT_84_FUNCTION: &str = "SDA_Get_MulineWktWgs84_from_MukeyTable";
const MULINE_TABLE_WKT_WM_FUNCTION: &str = "SDA_Get_MulineWktWm_from_MukeyTable";

#[derive(Clone)]
struct IntersectionTarget {
    function: &'static str,
    table: &'static str,
    key: &'static str,
    join_statusmap: bool,
    input_srid: i32,
    geometry_srid: Option<i32>,
    column: String,
}

#[derive(Clone)]
struct ScalarWktTarget {
    function: &'static str,
    table: &'static str,
    key: &'static str,
    output: &'static str,
    srid: i32,
    column: String,
}

#[derive(Clone)]
struct TableWktTarget {
    function: &'static str,
    table: &'static str,
    key: &'static str,
    input_key: &'static str,
    output: &'static str,
    srid: i32,
    column: String,
}

fn intersection_spec(name: &str) -> Option<(&'static str, &'static str, &'static str, bool, i32)> {
    [
        (MUKEY_FUNCTION, "mupolygon", "mukey", false, 4326),
        (MUKEY_WM_FUNCTION, "mupolygon", "mukey", false, 3857),
        (
            MUPOLYGONKEY_FUNCTION,
            "mupolygon",
            "mupolygonkey",
            false,
            4326,
        ),
        (
            MUPOLYGONKEY_WM_FUNCTION,
            "mupolygon",
            "mupolygonkey",
            false,
            3857,
        ),
        (
            SAPOLYGONKEY_FUNCTION,
            "sapolygon",
            "sapolygonkey",
            false,
            4326,
        ),
        (
            SAPOLYGONKEY_WM_FUNCTION,
            "sapolygon",
            "sapolygonkey",
            false,
            3857,
        ),
        (AREASYMBOL_FUNCTION, "sapolygon", "areasymbol", false, 4326),
        (
            AREASYMBOL_WM_FUNCTION,
            "sapolygon",
            "areasymbol",
            false,
            3857,
        ),
        (MUPOINTKEY_FUNCTION, "mupoint", "mupointkey", false, 4326),
        (MUPOINTKEY_WM_FUNCTION, "mupoint", "mupointkey", false, 3857),
        (MULINEKEY_FUNCTION, "muline", "mulinekey", false, 4326),
        (MULINEKEY_WM_FUNCTION, "muline", "mulinekey", false, 3857),
        (FEATLINEKEY_FUNCTION, "featline", "featlinekey", false, 4326),
        (
            FEATLINEKEY_WM_FUNCTION,
            "featline",
            "featlinekey",
            false,
            3857,
        ),
        (
            FEATPOINTKEY_FUNCTION,
            "featpoint",
            "featpointkey",
            false,
            4326,
        ),
        (
            FEATPOINTKEY_WM_FUNCTION,
            "featpoint",
            "featpointkey",
            false,
            3857,
        ),
        (
            SAPOLYGONKEY_STATUS_FUNCTION,
            "sapolygon",
            "sapolygonkey",
            true,
            4326,
        ),
        (
            SAPOLYGONKEY_STATUS_WM_FUNCTION,
            "sapolygon",
            "sapolygonkey",
            true,
            3857,
        ),
    ]
    .into_iter()
    .find(|(function, _, _, _, _)| function.eq_ignore_ascii_case(name))
}

fn scalar_wkt_spec(
    name: &str,
) -> Option<(&'static str, &'static str, &'static str, &'static str, i32)> {
    [
        (
            AREASYMBOL_WKT_84_FUNCTION,
            "sapolygon",
            "areasymbol",
            "AreasymbolWkt84",
            4326,
        ),
        (
            AREASYMBOL_WKT_WM_FUNCTION,
            "sapolygon",
            "areasymbol",
            "AreasymbolWktWm",
            3857,
        ),
        (
            MUPOLYGON_WKT_84_FUNCTION,
            "mupolygon",
            "mukey",
            "MupolygonWktWgs84",
            4326,
        ),
        (
            MUPOLYGON_WKT_WM_FUNCTION,
            "mupolygon",
            "mukey",
            "MupolygonWktWm",
            3857,
        ),
        (
            MUPOINT_WKT_84_FUNCTION,
            "mupoint",
            "mukey",
            "MupointWktWgs84",
            4326,
        ),
        (
            MUPOINT_WKT_WM_FUNCTION,
            "mupoint",
            "mukey",
            "MupointWktWm",
            3857,
        ),
        (
            MULINE_WKT_84_FUNCTION,
            "muline",
            "mukey",
            "MulineWktWgs84",
            4326,
        ),
        (
            MULINE_WKT_WM_FUNCTION,
            "muline",
            "mukey",
            "MulineWktWm",
            3857,
        ),
    ]
    .into_iter()
    .find(|(function, ..)| function.eq_ignore_ascii_case(name))
}

fn table_wkt_spec(
    name: &str,
) -> Option<(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    i32,
)> {
    [
        (
            AREASYMBOL_TABLE_WKT_84_FUNCTION,
            "sapolygon",
            "areasymbol",
            "v",
            "AreasymbolWkt84",
            4326,
        ),
        (
            AREASYMBOL_TABLE_WKT_WM_FUNCTION,
            "sapolygon",
            "areasymbol",
            "v",
            "AreasymbolWktWm",
            3857,
        ),
        (
            MUPOLYGON_TABLE_WKT_84_FUNCTION,
            "mupolygon",
            "mukey",
            "i",
            "MupolygonWktWgs84",
            4326,
        ),
        (
            MUPOLYGON_TABLE_WKT_WM_FUNCTION,
            "mupolygon",
            "mukey",
            "i",
            "MupolygonWktWm",
            3857,
        ),
        (
            MUPOINT_TABLE_WKT_84_FUNCTION,
            "mupoint",
            "mukey",
            "i",
            "MupointWktWgs84",
            4326,
        ),
        (
            MUPOINT_TABLE_WKT_WM_FUNCTION,
            "mupoint",
            "mukey",
            "i",
            "MupointWktWm",
            3857,
        ),
        (
            MULINE_TABLE_WKT_84_FUNCTION,
            "muline",
            "mukey",
            "i",
            "MulineWktWgs84",
            4326,
        ),
        (
            MULINE_TABLE_WKT_WM_FUNCTION,
            "muline",
            "mukey",
            "i",
            "MulineWktWm",
            3857,
        ),
    ]
    .into_iter()
    .find(|(function, ..)| function.eq_ignore_ascii_case(name))
}

pub(super) fn parser_compatibility(sql: &str) -> String {
    let sql = expand_table_macros(sql);
    const NEEDLE: &str = "geometry::STGeomFromText";

    let mut output = String::with_capacity(sql.len());
    let mut index = 0;
    while index < sql.len() {
        let bytes = sql.as_bytes();
        if let Some(end) = copy_protected_sql(&sql, index, &mut output) {
            index = end;
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
        if sql
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

fn expand_table_macros(sql: &str) -> String {
    let mut output = String::with_capacity(sql.len());
    let mut declarations = Vec::new();
    let mut index = 0;

    while index < sql.len() {
        if let Some(end) = copy_protected_sql(sql, index, &mut output) {
            index = end;
            continue;
        }
        if sql.as_bytes()[index] == b'~' {
            if let Some(end_offset) = sql[index + 1..].find('~') {
                let end = index + 1 + end_offset;
                let macro_text = &sql[index + 1..end];
                if let Some((name, declaration)) = parse_table_macro(macro_text) {
                    declarations.push(name.clone());
                    output.push_str(&declaration);
                    output.push(';');
                    index = end + 1;
                    continue;
                }
            }
        }
        let character = sql[index..].chars().next().unwrap();
        output.push(character);
        index += character.len_utf8();
    }

    if declarations.is_empty() {
        return output;
    }
    replace_table_variables(&output, &declarations)
}

fn parse_table_macro(input: &str) -> Option<(String, String)> {
    let open = input.find('(')?;
    let close = input.rfind(')');
    if close != Some(input.len() - 1) {
        return None;
    }
    let macro_name = input[..open].trim();
    let variable = input[open + 1..close?].trim();
    if !variable.starts_with('@')
        || !variable[1..]
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return None;
    }
    let (column, data_type) = if macro_name.eq_ignore_ascii_case("DeclareIntTable") {
        ("i", "INT")
    } else if macro_name.eq_ignore_ascii_case("DeclareVarcharTable")
        || macro_name.eq_ignore_ascii_case("DeclareVarchar255Table")
    {
        ("v", "VARCHAR(255)")
    } else {
        return None;
    };
    let table = format!("#{}", &variable[1..]);
    Some((
        variable.to_ascii_lowercase(),
        format!("CREATE TABLE {table} ({column} {data_type})"),
    ))
}

fn replace_table_variables(sql: &str, variables: &[String]) -> String {
    let mut output = String::with_capacity(sql.len());
    let mut index = 0;
    while index < sql.len() {
        if let Some(end) = copy_protected_sql(sql, index, &mut output) {
            index = end;
            continue;
        }
        if sql.as_bytes()[index] == b'@' {
            let end = sql[index..]
                .find(|character: char| {
                    !(character == '@' || character.is_ascii_alphanumeric() || character == '_')
                })
                .map(|offset| index + offset)
                .unwrap_or(sql.len());
            let variable = sql[index..end].to_ascii_lowercase();
            if variables.iter().any(|known| known == &variable) {
                output.push('#');
                output.push_str(&sql[index + 1..end]);
                index = end;
                continue;
            }
        }
        let character = sql[index..].chars().next().unwrap();
        output.push(character);
        index += character.len_utf8();
    }
    output
}

fn copy_protected_sql(sql: &str, start: usize, output: &mut String) -> Option<usize> {
    let bytes = sql.as_bytes();
    let first = *bytes.get(start)?;
    let (closing, doubled) = match first {
        b'\'' | b'"' | b'`' => (first, true),
        b'[' => (b']', true),
        b'-' if bytes.get(start + 1) == Some(&b'-') => {
            let end = sql[start..]
                .find('\n')
                .map_or(sql.len(), |offset| start + offset + 1);
            output.push_str(&sql[start..end]);
            return Some(end);
        }
        b'/' if bytes.get(start + 1) == Some(&b'*') => {
            let end = sql[start + 2..]
                .find("*/")
                .map_or(sql.len(), |offset| start + offset + 4);
            output.push_str(&sql[start..end]);
            return Some(end);
        }
        _ => return None,
    };

    let mut end = start + 1;
    while end < bytes.len() {
        if bytes[end] == closing {
            if doubled && bytes.get(end + 1) == Some(&closing) {
                end += 2;
                continue;
            }
            end += 1;
            break;
        }
        end += 1;
    }
    output.push_str(&sql[start..end]);
    Some(end)
}

pub(super) fn rewrite(
    statements: Vec<Expression>,
    capability: &SpatialCapability,
) -> Result<Vec<Expression>, TranspilerError> {
    let mut output = Vec::with_capacity(statements.len());
    for statement in statements {
        let spatial_statement = statement.dfs().any(is_spatial_expression);
        let geometry_column = if spatial_statement {
            capability.effective_geometry_column().unwrap_or("geom")
        } else {
            "geom"
        };
        let geometry_relations = resolve_relations(&statement, capability, spatial_statement)?;
        let mut targets = Vec::new();
        let mut wkt_targets = Vec::new();
        let mut table_wkt_targets = Vec::new();
        for node in statement.dfs() {
            if let Expression::Function(function) = node {
                if let Some((name, table, key, input_key, output, srid)) =
                    table_wkt_spec(&function.name)
                {
                    if table_wkt_targets
                        .iter()
                        .any(|target: &TableWktTarget| target.function == name)
                    {
                        continue;
                    }
                    let metadata = capability.geometry_metadata_for_table(table);
                    if metadata.is_none() && capability.geometry_override().is_none() {
                        return Err(TranspilerError::Rejected(format!(
                            "{name} TVF requires geometry metadata for {table}"
                        )));
                    }
                    table_wkt_targets.push(TableWktTarget {
                        function: name,
                        table,
                        key,
                        input_key,
                        output,
                        srid,
                        column: capability
                            .geometry_override()
                            .or_else(|| metadata.map(|metadata| metadata.geometry_column.as_str()))
                            .or_else(|| capability.effective_geometry_column())
                            .unwrap_or(geometry_column)
                            .to_owned(),
                    });
                    continue;
                }
                if let Some((name, table, key, output, srid)) = scalar_wkt_spec(&function.name) {
                    if wkt_targets
                        .iter()
                        .any(|target: &ScalarWktTarget| target.function == name)
                    {
                        continue;
                    }
                    let metadata = capability.geometry_metadata_for_table(table);
                    if metadata.is_none() && capability.geometry_override().is_none() {
                        return Err(TranspilerError::Rejected(format!(
                            "{name} TVF requires geometry metadata for {table}"
                        )));
                    }
                    wkt_targets.push(ScalarWktTarget {
                        function: name,
                        table,
                        key,
                        output,
                        srid,
                        column: capability
                            .geometry_override()
                            .or_else(|| metadata.map(|metadata| metadata.geometry_column.as_str()))
                            .or_else(|| capability.effective_geometry_column())
                            .unwrap_or(geometry_column)
                            .to_owned(),
                    });
                    continue;
                }
                if let Some((name, table, key, join_statusmap, input_srid)) =
                    intersection_spec(&function.name)
                {
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
                        join_statusmap,
                        input_srid,
                        geometry_srid: if name == MUKEY_FUNCTION {
                            metadata.map(|metadata| metadata.srs_id)
                        } else {
                            None
                        },
                        column: capability
                            .geometry_override()
                            .or_else(|| metadata.map(|metadata| metadata.geometry_column.as_str()))
                            .or_else(|| capability.effective_geometry_column())
                            .unwrap_or(geometry_column)
                            .to_owned(),
                    });
                }
            }
        }
        let statement = transform_all(statement, &|node| {
            let node = lift_intersection_select(node, &targets, &wkt_targets, &table_wkt_targets)?;
            expand_tables(node, &targets, &wkt_targets, &table_wkt_targets)
        })
        .map_err(classify_error)?;
        output.push(
            transform_all(statement, &|node| rewrite_node(node, &geometry_relations))
                .map_err(classify_error)?,
        );
    }
    Ok(output)
}

fn resolve_relations(
    statement: &Expression,
    capability: &SpatialCapability,
    spatial_statement: bool,
) -> Result<Vec<(String, String, String)>, TranspilerError> {
    if !spatial_statement {
        return Ok(Vec::new());
    }
    let mut relations = Vec::new();
    for node in statement.dfs() {
        let Expression::Select(select) = node else {
            continue;
        };
        let sources = select
            .from
            .iter()
            .flat_map(|from| from.expressions.iter())
            .chain(select.joins.iter().map(|join| &join.this));
        for source in sources {
            let (table, source_alias) = match source {
                Expression::Table(table) => (table.as_ref(), None),
                Expression::Alias(alias) => match &alias.this {
                    Expression::Table(table) => (table.as_ref(), Some(&alias.alias)),
                    _ => continue,
                },
                _ => continue,
            };
            let name = table.name.name.as_str();
            let metadata = capability
                .resolve_geometry_metadata_for_table(name)
                .map_err(TranspilerError::Rejected)?;
            let qualifier = source_alias
                .or(table.alias.as_ref())
                .map(|alias| alias.name.as_str())
                .unwrap_or(name);
            relations.push((
                qualifier.to_ascii_lowercase(),
                metadata
                    .map(|metadata| metadata.geometry_column.to_ascii_lowercase())
                    .or_else(|| capability.geometry_override().map(str::to_ascii_lowercase))
                    .unwrap_or_default(),
                name.to_ascii_lowercase(),
            ));
        }
    }
    for node in statement.dfs() {
        let Expression::MethodCall(call) = node else {
            continue;
        };
        if !call.method.name.eq_ignore_ascii_case("STIntersects") {
            continue;
        }
        for operand in std::iter::once(&call.this).chain(call.args.iter()) {
            let Expression::Column(column) = operand else {
                continue;
            };
            let found = relations.iter().any(|(qualifier, geometry, _)| {
                (column
                    .table
                    .as_ref()
                    .is_none_or(|table| table.name.eq_ignore_ascii_case(qualifier)))
                    && !geometry.is_empty()
                    && column.name.name.eq_ignore_ascii_case(geometry)
            });
            let relation_missing = column.table.as_ref().is_some_and(|table| {
                relations
                    .iter()
                    .any(|(qualifier, _, _)| table.name.eq_ignore_ascii_case(qualifier))
                    && !relations.iter().any(|(qualifier, geometry, _)| {
                        table.name.eq_ignore_ascii_case(qualifier) && !geometry.is_empty()
                    })
            });
            if relation_missing || (!found && column.table.is_some()) {
                let table = column
                    .table
                    .as_ref()
                    .map(|table| table.name.as_str())
                    .unwrap_or("referenced");
                return Err(TranspilerError::Rejected(format!(
                    "spatial reference to table {table} has no geometry metadata"
                )));
            }
        }
    }
    Ok(relations)
}

fn lift_intersection_select(
    statement: Expression,
    targets: &[IntersectionTarget],
    wkt_targets: &[ScalarWktTarget],
    table_wkt_targets: &[TableWktTarget],
) -> Result<Expression, polyglot_sql::Error> {
    let Expression::Select(mut select) = statement else {
        return Ok(statement);
    };
    if select.from.is_some()
        || select.expressions.len() != 1
        || !matches!(&select.expressions[0], Expression::Function(function) if intersection_spec(&function.name).is_some())
    {
        return Ok(Expression::Select(select));
    }
    let derived = expand_source(
        select.expressions[0].clone(),
        targets,
        wkt_targets,
        table_wkt_targets,
    )?;
    let mut parsed = Dialect::get(DialectType::TSQL).parse("SELECT * FROM placeholder")?;
    let Expression::Select(mut template) = parsed.remove(0) else {
        unreachable!()
    };
    select.expressions = template.expressions;
    let mut from = template.from.take().unwrap();
    from.expressions[0] = derived;
    select.from = Some(from);
    Ok(Expression::Select(select))
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
    wkt_targets: &[ScalarWktTarget],
    table_wkt_targets: &[TableWktTarget],
) -> Result<Expression, polyglot_sql::Error> {
    let Expression::Select(mut select) = node else {
        return Ok(node);
    };
    if let Some(from) = &mut select.from {
        for source in &mut from.expressions {
            *source = expand_source(source.clone(), targets, wkt_targets, table_wkt_targets)?;
        }
    }
    for join in &mut select.joins {
        join.this = expand_source(join.this.clone(), targets, wkt_targets, table_wkt_targets)?;
    }
    Ok(Expression::Select(select))
}

fn expand_source(
    source: Expression,
    targets: &[IntersectionTarget],
    wkt_targets: &[ScalarWktTarget],
    table_wkt_targets: &[TableWktTarget],
) -> Result<Expression, polyglot_sql::Error> {
    let table_match = match &source {
        Expression::Function(function) if table_wkt_spec(&function.name).is_some() => {
            Some((function, None))
        }
        Expression::Alias(alias) => match &alias.this {
            Expression::Function(function) if table_wkt_spec(&function.name).is_some() => {
                if !alias.column_aliases.is_empty() {
                    return Err(polyglot_sql::Error::InvalidInput(format!(
                        "{} TVF does not support column alias lists",
                        function.name
                    )));
                }
                Some((function, Some(alias.alias.clone())))
            }
            _ => None,
        },
        _ => None,
    };
    if let Some((function, alias)) = table_match {
        return expand_table_wkt_source(function, alias, table_wkt_targets);
    }
    let (wkt_function, wkt_alias) = match &source {
        Expression::Function(function) if scalar_wkt_spec(&function.name).is_some() => {
            (function, None)
        }
        Expression::Alias(alias) => match &alias.this {
            Expression::Function(function) if scalar_wkt_spec(&function.name).is_some() => {
                if !alias.column_aliases.is_empty() {
                    return Err(polyglot_sql::Error::InvalidInput(format!(
                        "{} TVF does not support column alias lists",
                        function.name
                    )));
                }
                (function, Some(alias.alias.clone()))
            }
            _ => return expand_intersection_source(source, targets),
        },
        _ => return expand_intersection_source(source, targets),
    };
    expand_scalar_wkt_source(wkt_function, wkt_alias, wkt_targets)
}

fn expand_intersection_source(
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
    let geometry_srid = target.geometry_srid.unwrap_or(target.input_srid);
    let predicate = format!(
        "ST_Intersects(ST_Transform(GeomFromGPB(mp.__geom__), {0}), ST_Transform(GeomFromText(__wkt__, {1}), {0})) = 1",
        geometry_srid, target.input_srid
    );
    let filter = if target.table == "mupolygon" {
        "mp.areasymbol <> 'US' AND "
    } else {
        ""
    };
    let query = if target.join_statusmap {
        format!(
            "SELECT * FROM (SELECT DISTINCT mp.{} FROM {} AS mp INNER JOIN sastatusmap AS sm ON mp.areasymbol = sm.areasymbol WHERE {}{}) AS __sda",
            target.key, target.table, filter, predicate
        )
    } else {
        format!(
            "SELECT * FROM (SELECT DISTINCT mp.{} FROM {} AS mp WHERE {}{}) AS __sda",
            target.key, target.table, filter, predicate
        )
    };
    let mut parsed = Dialect::get(DialectType::TSQL).parse(&query)?;
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

fn expand_scalar_wkt_source(
    function: &Function,
    alias: Option<Identifier>,
    targets: &[ScalarWktTarget],
) -> Result<Expression, polyglot_sql::Error> {
    let target = targets
        .iter()
        .find(|target| target.function.eq_ignore_ascii_case(&function.name))
        .ok_or_else(|| {
            polyglot_sql::Error::InvalidInput("missing scalar WKT target metadata".into())
        })?;
    if function.args.len() != 1
        || !matches!(
            &function.args[0],
            Expression::Literal(_) | Expression::Null(_)
        )
    {
        return Err(polyglot_sql::Error::InvalidInput(format!(
            "{} TVF requires one literal key argument; correlated and other expressions are unsupported",
            target.function
        )));
    }
    let argument = function.args[0].clone();
    let key_filter = "mp.__key__ = __arg__";
    let query = format!(
        "SELECT * FROM (SELECT AsText(ST_Transform(GeomFromGPB(mp.__geom__), {})) AS {} FROM {} AS mp WHERE {}) AS __sda",
        target.srid, target.output, target.table, key_filter
    );
    let mut parsed = Dialect::get(DialectType::TSQL).parse(&query)?;
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
            "unexpected scalar WKT derived table AST".into(),
        ));
    }
    transform_all(derived, &|node| match node {
        Expression::Column(mut column) if column.name.name == "__geom__" => {
            column.name.name = target.column.clone();
            column.name.quoted = true;
            Ok(Expression::Column(column))
        }
        Expression::Column(column) if column.name.name == "__key__" => {
            let mut column = column;
            column.name.name = target.key.to_owned();
            Ok(Expression::Column(column))
        }
        Expression::Column(column) if column.name.name == "__arg__" => Ok(argument.clone()),
        other => Ok(other),
    })
}

fn expand_table_wkt_source(
    function: &Function,
    alias: Option<Identifier>,
    targets: &[TableWktTarget],
) -> Result<Expression, polyglot_sql::Error> {
    let target = targets
        .iter()
        .find(|target| target.function.eq_ignore_ascii_case(&function.name))
        .ok_or_else(|| {
            polyglot_sql::Error::InvalidInput("missing table WKT target metadata".into())
        })?;
    let [Expression::Column(input_table)] = function.args.as_slice() else {
        return Err(polyglot_sql::Error::InvalidInput(format!(
            "{} TVF requires one declared table argument",
            target.function
        )));
    };
    let input_table = &input_table.name.name;
    if !input_table.starts_with("__sda_temp_")
        || !input_table
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err(polyglot_sql::Error::InvalidInput(format!(
            "{} TVF requires a table declared by an SDA input-table macro",
            target.function
        )));
    }
    let join = format!("mp.{} = input.{}", target.key, target.input_key);
    let query = format!(
        "SELECT * FROM (SELECT mp.{key}, AsText(ST_Transform(GeomFromGPB(mp.__geom__), {srid})) AS {output} FROM {table} AS mp INNER JOIN {input_table} AS input ON {join}) AS __sda",
        key = target.key,
        srid = target.srid,
        output = target.output,
        table = target.table,
    );
    let mut parsed = Dialect::get(DialectType::TSQL).parse(&query)?;
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
            "unexpected table WKT derived table AST".into(),
        ));
    }
    transform_all(derived, &|node| match node {
        Expression::Column(mut column) if column.name.name == "__geom__" => {
            column.name.name = target.column.clone();
            column.name.quoted = true;
            Ok(Expression::Column(column))
        }
        other => Ok(other),
    })
}

fn is_spatial_expression(expression: &Expression) -> bool {
    match expression {
        Expression::Function(function) => {
            intersection_spec(&function.name).is_some()
                || scalar_wkt_spec(&function.name).is_some()
                || table_wkt_spec(&function.name).is_some()
        }
        Expression::MethodCall(call) => {
            call.method.name.eq_ignore_ascii_case("STIntersects")
                || call.method.name.eq_ignore_ascii_case("STGeomFromText")
                || intersection_spec(&call.method.name).is_some()
                || scalar_wkt_spec(&call.method.name).is_some()
                || table_wkt_spec(&call.method.name).is_some()
        }
        _ => false,
    }
}

fn rewrite_node(
    node: Expression,
    geometry_relations: &[(String, String, String)],
) -> Result<Expression, polyglot_sql::Error> {
    Ok(match node {
        Expression::MethodCall(call) if scalar_wkt_spec(&call.method.name).is_some() => {
            return Err(polyglot_sql::Error::InvalidInput(
                "qualified scalar WKT TVF syntax is unsupported".into(),
            ));
        }
        Expression::MethodCall(call) if table_wkt_spec(&call.method.name).is_some() => {
            return Err(polyglot_sql::Error::InvalidInput(
                "qualified table WKT TVF syntax is unsupported".into(),
            ));
        }
        Expression::Function(function) if table_wkt_spec(&function.name).is_some() => {
            return Err(polyglot_sql::Error::InvalidInput(
                "table WKT TVFs must be used as a FROM source".into(),
            ));
        }
        Expression::Function(function) if scalar_wkt_spec(&function.name).is_some() => {
            return Err(polyglot_sql::Error::InvalidInput(
                "scalar WKT TVFs must be used as a FROM source".into(),
            ));
        }
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
                Expression::Column(column)
                    if geometry_relations.iter().any(|(qualifier, geometry, _)| {
                        column.name.name.eq_ignore_ascii_case(geometry)
                            && column
                                .table
                                .as_ref()
                                .is_some_and(|table| table.name.eq_ignore_ascii_case(qualifier))
                    }) || (column.table.is_none()
                        && geometry_relations
                            .iter()
                            .filter(|(_, geometry, _)| {
                                column.name.name.eq_ignore_ascii_case(geometry)
                            })
                            .count()
                            == 1) =>
                {
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
            assert!(generated.contains("= 1"), "{generated}");
            assert!(generated.contains("areasymbol <> 'US'"), "{generated}");
            assert!(!generated.contains(MUKEY_FUNCTION), "{generated}");
        }
    }

    #[test]
    fn scalar_mukey_wkt_uses_sqlite_compatible_key_comparison() {
        for argument in ["'inside'", "11", "'11'"] {
            let sql = format!("SELECT * FROM SDA_Get_MupolygonWktWgs84_from_Mukey({argument})");
            let generated =
                crate::transpiler::transpile_with_spatial(&sql, Some(&capability())).unwrap();
            let generated = &generated[0];
            assert!(
                generated.to_ascii_lowercase().contains("mp.mukey ="),
                "{generated}"
            );
            assert!(
                !generated.to_ascii_uppercase().contains("VARCHAR"),
                "{generated}"
            );
            assert!(
                !generated.to_ascii_uppercase().contains("MAX"),
                "{generated}"
            );
            assert!(!generated.contains("__arg__"), "{generated}");
        }
    }

    #[test]
    fn wkt_tvfs_reject_column_alias_lists() {
        for sql in [
            "SELECT * FROM SDA_Get_MupolygonWktWgs84_from_Mukey(11) AS w(shape)",
            "~DeclareIntTable(@mu)~ SELECT * FROM \
             SDA_Get_MupolygonWktWgs84_from_MukeyTable(@mu) AS w(key, shape)",
        ] {
            let error =
                crate::transpiler::transpile_with_spatial(sql, Some(&capability())).unwrap_err();
            assert!(error.to_string().contains("column alias lists"), "{error}");
        }
    }

    #[test]
    fn table_input_macros_expand_and_join_through_request_temp_tables() {
        let sql = "~DeclareIntTable(@mu)~ INSERT INTO @mu VALUES (11); \
                   SELECT mukey, MupolygonWktWgs84 \
                   FROM SDA_Get_MupolygonWktWgs84_from_MukeyTable(@mu)";
        let generated =
            crate::transpiler::transpile_with_spatial(sql, Some(&capability())).unwrap();
        assert_eq!(generated.len(), 3, "{generated:?}");
        assert!(
            generated[0].contains("CREATE TEMPORARY TABLE __sda_temp_"),
            "{generated:?}"
        );
        assert!(
            generated[1].contains("INSERT INTO __sda_temp_"),
            "{generated:?}"
        );
        assert!(
            generated[2].contains("INNER JOIN __sda_temp_"),
            "{generated:?}"
        );
        assert!(generated[2].contains("mp.mukey = input.i"), "{generated:?}");
        assert!(generated[2].contains("MupolygonWktWgs84"), "{generated:?}");
        assert!(generated
            .iter()
            .all(|sql| !sql.contains("@mu") && !sql.contains("#mu")));
    }

    #[test]
    fn varchar255_macro_expands_without_rewriting_string_contents() {
        assert_eq!(
            parser_compatibility(
                "~DeclareVarchar255Table(@names)~ INSERT INTO @names VALUES ('@names')"
            ),
            "CREATE TABLE #names (v VARCHAR(255)); INSERT INTO #names VALUES ('@names')"
        );
    }

    #[test]
    fn macro_preprocessing_preserves_comments_and_delimited_identifiers() {
        let sql = "~DeclareIntTable(@mu)~ -- author's @mu and ~DeclareIntTable(@x)~\n\
                   SELECT '@mu' AS literal, [@mu] AS identifier, @mu AS input";
        assert_eq!(
            parser_compatibility(sql),
            "CREATE TABLE #mu (i INT); -- author's @mu and ~DeclareIntTable(@x)~\n\
             SELECT '@mu' AS literal, [@mu] AS identifier, #mu AS input"
        );
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
        assert!(
            generated.contains("GEOMFROMTEXT('POINT(0 0)', 4326)"),
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
