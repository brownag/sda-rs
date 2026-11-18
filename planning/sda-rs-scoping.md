# **SPEC\_SDA\_RS: Architecture and Transpilation Specification for sda-rs**

This engineering specification defines the AST transpilation pipeline, spatial UDF rewrite catalog, HTTP wire protocol, and validation suite for sda-rs, a standalone Rust server emulating the USDA-NRCS Soil Data Access (SDA) POST /Tabular/post.rest endpoint over PostGIS or SQLite/SpatiaLite (OpenSSURGO GeoPackage) backends1. Three structural constraints govern the design: polyglot-sql defines its AST via polyglot\_sql::expressions::Expression and its SQL Server dialect via DialectType::TSQL rather than ast::Expr or Dialect::Tsql5; the JSON+COLUMNNAME+METADATA response places SQL Server column metadata in the second array element (0-indexed Row 1, documented by NRCS as Row 2\)2; and OpenSSURGO GeoPackage databases store geometry column names and spatial reference IDs dynamically in gpkg\_geometry\_columns rather than a fixed mupolygongeo column4.  
Several implementation details could not be verified from the publicly indexed documentation and require direct inspection of local polyglot-sql, soilDB, and py-soildb source checkouts before final implementation. Specifically, the exact Rust struct fields inside polyglot\_sql::expressions::Expression variants (such as Declare, Apply, and Function)10, the full Content-Type and response parsing code in soilDB (SDA\_query, .parseSDA) and py-soildb (SDAClient), and the complete SqlDataReader.GetSchemaTable() key set and integer SqlDbType mappings beyond the attributes listed in USDA-NRCS documentation2 were absent from inspected artifacts. All verified behaviors from official USDA-NRCS and polyglot-sql documentation are distinguished throughout this specification from provisional pseudocode and unverified candidate mappings.

## **1\. AST Representation and Transformation Passes in polyglot-sql**

### **Crate API Alignment and Native Dialect Coverage**

In the polyglot-sql crate (v0.14.0–0.14.1, MIT-licensed, derived from Python sqlglot)5, SQL parsing, AST manipulation, and SQL generation operate through polyglot\_sql::parse, polyglot\_sql::generate, polyglot\_sql::transpile, and polyglot\_sql::traversal::\*5. Unlike the separate sqlglot-rust crate—which defines Dialect::Tsql16—polyglot-sql exposes DialectType::TSQL for T-SQL and DialectType::Postgres for PostgreSQL5, alongside SQLite support among its more than 30 dialects12. Its central AST node is polyglot\_sql::expressions::Expression, a tagged enum with over 200 variants6, traversed via ast-tools (transform, walk, DFS/BFS iterators) with stack-growth protection enabled by the default stacker feature5.  
Publicly indexed polyglot-sql documentation confirms AST node coverage and partial normalization for several T-SQL constructs, while domain-specific passes remain necessary for SQL Server CLR spatial methods, SDA table-valued functions (TVFs), session variables, and temporary tables7. Exact field names inside Expression variants and full DialectType::TSQL-to-SQLite transpilation parity could not be confirmed from public docs alone and must be verified against crates/polyglot-sql/src/expressions.rs and src/dialects/tsql.rs in the crate source.

| T-SQL Construct | Verified Evidence in polyglot-sql | Native Transpilation Status | Required sda-rs AST Pass |
| :---- | :---- | :---- | :---- |
| TOP (N) | SELECT TOP 10 ... transpiles natively to LIMIT 10 targeting PostgreSQL17. | Native for unparenthesized TOP N to Postgres17; TOP (N) and SQLite unverified | Pass 6: Normalize TOP (N) if parenthesized form is not lowered natively for SQLite. |
| DECLARE @var type; SELECT @var \= ... | Declare and DeclareItem variants exist in expressions11; Action::SetToVariable exists in statement normalization7. | Partial / T-SQL specific | Pass 1: Fold variable declarations and @var \= expr assignments across the batch. |
| SELECT ... INTO \#temp FROM ... | Statement normalization includes INTO \-\> CREATE TABLE AS handling DialectType::TSQL7. | Partial (local \# prefix and SQLite target unverified) | Pass 2: Convert INTO \#temp to CREATE TEMP TABLE temp AS SELECT ... and strip \# in downstream references. |
| mupolygongeo.STIntersects(@aoi) \= 1, .STIntersection(@aoi).STArea() | Expression::Function exists10; exact AST variant for :: static calls and chained .ST\*() methods in TSQL is unverified. | Unsupported (SQL Server CLR spatial semantics) | Pass 3: Lower CLR spatial calls to PostGIS/SpatiaLite ST\_\* functions and unwrap \= 1 for Postgres. |
| FROM SDA\_Get\_\*(...) | Standard Expression::Function exists10; TVF wrapping in FROM/JOIN requires expressions.rs check. | Unsupported (NRCS-specific UDFs) | Pass 4: Expand SDA\_Get\_\* TVFs into derived subqueries over SSURGO spatial tables. |
| CROSS APPLY / OUTER APPLY | Apply variant exists in polyglot\_sql::expressions11. | Dialect-dependent (LATERAL in Postgres; unsupported in SQLite) | Pass 5: Emit CROSS/LEFT JOIN LATERAL for Postgres; rewrite scalar TOP 1 APPLY for SQLite. |
| STRING\_AGG(...), ISNULL(...), IIF(...) | IFNULL normalizes to COALESCE for MySQL-to-Postgres5; exact T-SQL mappings unverified. | Unverified across Postgres and SQLite | Pass 6: Map ISNULL to COALESCE, IIF to CASE WHEN, and STRING\_AGG to GROUP\_CONCAT (SQLite). |

### **AST Transformation Pipeline and Pattern-Matching Pseudocode**

Because polyglot-sql decouples parsing from dialect codegen, sda-rs should parse incoming T-SQL via polyglot\_sql::parse(sql, DialectType::TSQL) into Vec\<Expression\>5, apply six ordered AST transformation passes using traversal::transform, and emit target SQL via polyglot\_sql::generate(\&expr, target\_dialect)5. Prior to parsing, a text pre-pass must expand SDA macro tokens such as \~DeclareIntTable(@varname)\~ and \~DeclareVarchar255Table(@varname)\~18 into standard SQL CREATE TEMP TABLE declarations. The Rust pseudocode below captures the matching rules for each pass; struct field names inside Expression variants are provisional architectural pseudocode anchored to verified enum names (Declare, DeclareItem, Apply, Function)10 and must be aligned with expressions.rs.  
Pass 1 walks multi-statement batches, captures DECLARE @aoi geometry and SELECT @aoi \= geometry::STGeomFromText(wkt, srid), drops those procedural statements from the executable stream, and inlines the lowered spatial expression wherever @aoi appears in subsequent statements. Pass 2 intercepts SELECT ... INTO \#temp\_table statements—either before or in place of polyglot-sql's built-in INTO normalization7—to emit CREATE TEMP TABLE temp\_table AS SELECT ... and rewrites \#temp\_table identifiers in later statements of the batch.

Rust  
// Architectural pseudocode — align variant payload structs with polyglot\_sql::expressions  
use std::collections::{HashMap, HashSet};  
use polyglot\_sql::expressions::Expression;

/// Pass 1: Fold \`DECLARE @var\` and \`SELECT @var \= expr\` assignments across a statement batch.  
fn pass\_fold\_tsql\_variables(stmts: Vec\<Expression\>, backend: Backend) \-\> Vec\<Expression\> {  
    let mut env: HashMap\<String, Expression\> \= HashMap::new();  
    let mut out \= Vec::new();

    for stmt in stmts {  
        match \&stmt {  
            Expression::Declare { items, .. } \=\> {  
                for item in items {  
                    env.entry(item.name().to\_ascii\_lowercase()).or\_insert(Expression::Null);  
                }  
            }  
            Expression::Select { expressions, from: None, .. }  
                if expressions.len() \== 1 && is\_var\_assignment(\&expressions\[0\]) \=\>  
            {  
                let (var\_name, rhs) \= extract\_var\_assignment(\&expressions\[0\]);  
                env.insert(var\_name.to\_ascii\_lowercase(), pass\_lower\_clr\_spatial(rhs, backend));  
            }  
            \_ \=\> {  
                let rewritten \= transform\_expr(stmt, \|node\| match node {  
                    Expression::Parameter { name, .. } \| Expression::Column { name, .. }  
                        if name.starts\_with('@') \=\>  
                    {  
                        env.get(\&name.to\_ascii\_lowercase()).cloned().unwrap\_or(node)  
                    }  
                    other \=\> other,  
                });  
                out.push(rewritten);  
            }  
        }  
    }  
    out  
}

/// Pass 2: Rewrite \`SELECT ... INTO \#tmp FROM ...\` into \`CREATE TEMP TABLE tmp AS SELECT ...\`  
/// and strip \`\#\` prefixes on subsequent references to declared temp tables.  
fn pass\_rewrite\_temp\_tables(stmts: Vec\<Expression\>) \-\> Vec\<Expression\> {  
    let mut temp\_names: HashSet\<String\> \= HashSet::new();  
    let mut out \= Vec::new();

    for stmt in stmts {  
        let rewritten \= match stmt {  
            Expression::Select { into: Some(target), mut rest } if target.name().starts\_with('\#') \=\> {  
                let clean \= target.name().trim\_start\_matches('\#').to\_string();  
                temp\_names.insert(clean.to\_ascii\_lowercase());  
                let select\_body \= Expression::Select { into: None, ..rest };  
                make\_create\_temp\_table\_as(\&clean, select\_body)  
            }  
            other \=\> transform\_expr(other, \|node\| match node {  
                Expression::Table { name, .. } if name.starts\_with('\#') \=\> {  
                    let clean \= name.trim\_start\_matches('\#').to\_string();  
                    replace\_table\_name(node, clean)  
                }  
                other\_node \=\> other\_node,  
            }),  
        };  
        out.push(rewritten);  
    }  
    out  
}

Pass 3 lowers SQL Server static constructors (geometry::STGeomFromText) and chained CLR instance methods (.STIntersects(), .STIntersection(), .STArea(), .STAsText()) into PostGIS or SpatiaLite ST\_\* functions. Because SQL Server CLR spatial predicates return BIT (1/0), queries routinely compare mupolygongeo.STIntersects(@aoi) \= 1; Pass 3 unwraps EQ(ST\_Intersects(...), 1\) into a bare boolean predicate for PostgreSQL (avoiding boolean \= integer type errors) and wraps GeoPackage BLOB geometry columns in GeomFromGPB(...) when targeting OpenSSURGO GeoPackage files8. Pass 4 replaces SDA\_Get\_\* table-valued functions in FROM and JOIN clauses with derived subqueries mapped from the catalog in Section 2\.

Rust  
/// Pass 3: Lower CLR static/instance spatial calls and \`= 1\` predicate wrappers.  
fn pass\_lower\_clr\_spatial(expr: Expression, backend: Backend, gpkg\_geom\_col: \&str) \-\> Expression {  
    transform\_expr(expr, \|node\| match node {  
        Expression::StaticMethod { namespace, method, args }  
            if namespace.eq\_ignore\_ascii\_case("geometry")  
                && method.eq\_ignore\_ascii\_case("STGeomFromText") \=\>  
        {  
            make\_fn(if backend.is\_pg() { "ST\_GeomFromText" } else { "GeomFromText" }, args)  
        }  
        Expression::Method { this, name, args } \=\> {  
            let receiver \= normalize\_geom\_expr(\*this, backend, gpkg\_geom\_col);  
            match name.to\_ascii\_lowercase().as\_str() {  
                "stintersects" \=\> make\_fn("ST\_Intersects", prepend(receiver, args)),  
                "stintersection" \=\> make\_fn("ST\_Intersection", prepend(receiver, args)),  
                "starea" \=\> lower\_area\_expr(receiver, backend),  
                "stastext" \=\> make\_fn(if backend.is\_pg() { "ST\_AsText" } else { "AsText" }, vec\!\[receiver\]),  
                \_ \=\> Expression::Method { this: Box::new(receiver), name, args },  
            }  
        }  
        Expression::EQ { this, expression }  
            if backend.is\_pg() && is\_boolean\_spatial\_fn(\&this) && is\_literal\_one(\&expression) \=\>  
        {  
            \*this  
        }  
        other \=\> other,  
    })  
}

/// Pass 4: Expand \`FROM SDA\_Get\_\*(arg) \[AS alias\]\` into a derived subquery \`(SELECT ...) AS alias\`.  
fn pass\_expand\_sda\_udfs(expr: Expression, backend: Backend, schema: \&SpatialSchemaMeta) \-\> Expression {  
    transform\_expr(expr, \|node\| match node {  
        Expression::Table { name, args: Some(fn\_args), alias, .. }  
            if name.to\_ascii\_lowercase().starts\_with("sda\_get\_") \=\>  
        {  
            let subquery\_ast \= build\_sda\_udf\_subquery(\&name, \&fn\_args, backend, schema);  
            wrap\_as\_subquery(subquery\_ast, alias)  
        }  
        other \=\> other,  
    })  
}

Pass 5 rewrites CROSS APPLY and OUTER APPLY (Expression::Apply11) into CROSS JOIN LATERAL (...) and LEFT JOIN LATERAL (...) ON TRUE on PostgreSQL, whereas on SQLite—which lacks LATERAL—it transforms correlated single-column SELECT TOP 1 col FROM ... WHERE ... APPLY subqueries into scalar subqueries inside the SELECT list (or rejects multi-column row-multiplying APPLY expressions with a structured error). Pass 6 normalizes scalar/aggregate functions and parenthesized TOP (N) clauses so both backends execute without syntax errors.

Rust  
/// Pass 5: Rewrite \`CROSS APPLY\` / \`OUTER APPLY\` into PostgreSQL \`LATERAL\` or SQLite scalar subqueries.  
fn pass\_rewrite\_apply(expr: Expression, backend: Backend) \-\> Result\<Expression, TranspileError\> {  
    try\_transform\_expr(expr, \|node\| match node {  
        Expression::Apply { kind, this, expression, alias } \=\> match backend {  
            Backend::PostGIS \=\> Ok(Expression::Join {  
                side: if kind.is\_outer() { JoinSide::Left } else { JoinSide::Cross },  
                lateral: true,  
                this: Box::new(wrap\_subquery\_alias(\*expression, alias)),  
                on: if kind.is\_outer() { Some(Expression::Boolean(true)) } else { None },  
            }),  
            Backend::SpatiaLite \=\> lower\_top1\_apply\_to\_sqlite\_scalar(\*this, \*expression, alias),  
        },  
        other \=\> Ok(other),  
    })  
}

/// Pass 6: Normalize \`ISNULL\`, \`IIF\`, \`STRING\_AGG\`, and \`TOP (N)\` across Postgres and SQLite.  
fn pass\_normalize\_tsql\_functions(expr: Expression, backend: Backend) \-\> Expression {  
    transform\_expr(expr, \|node\| match node {  
        Expression::Function { name, args } if name.eq\_ignore\_ascii\_case("isnull") && args.len() \== 2 \=\> {  
            make\_fn("COALESCE", args)  
        }  
        Expression::Function { name, mut args } if name.eq\_ignore\_ascii\_case("iif") && args.len() \== 3 \=\> {  
            let (cond, t\_br, f\_br) \= (args.remove(0), args.remove(0), args.remove(0));  
            Expression::Case { ifs: vec\!\[(cond, t\_br)\], default: Some(Box::new(f\_br)) }  
        }  
        Expression::Function { name, args }  
            if \!backend.is\_pg() && name.eq\_ignore\_ascii\_case("string\_agg") && args.len() \== 2 \=\>  
        {  
            make\_fn("GROUP\_CONCAT", args)  
        }  
        Expression::Select { limit: Some(top\_expr), .. } if is\_tsql\_top(\&top\_expr) \=\> {  
            convert\_top\_to\_limit(node)  
        }  
        other \=\> other,  
    })  
}

## **2\. SDA Spatial UDF and CLR Method Catalog**

### **Complete SDA\_Get\_\* Table-Valued Function Surface Area**

Official USDA-NRCS Advanced Queries documentation specifies 34 SDA\_Get\_\* tabular functions partitioned symmetrically into WGS84 (WktWgs84, EPSG:4326 with longitude-then-latitude WKT) and Web Mercator (WktWm, EPSG:3857) variants1. Spatial queries invoked through SDA helper functions automatically exclude STATSGO records (areasymbol \!= 'US')19. No SDA\_Get\_Chorizonkey\_\* UDF exists in the official USDA-NRCS Advanced Queries catalog or in inspected soilDB and py-soildb documentation18; callers requiring horizon keys after spatial filtering join mapunit \-\> component \-\> chorizon on mukey and cokey1.  
For batch-input \*Table functions, SDA exposes pre-parse macros \~DeclareIntTable(@varname)\~ (expanding to dbo.IntTable with single column i int) and \~DeclareVarchar255Table(@varname)\~ / DeclareVarcharTable18. Additionally, because PostGIS schemas created via createSSURGO() or bulk dumps expose spatial columns such as geom or mupolygongeo while OpenSSURGO GeoPackage databases (created by NRCS SSURGO Portal, which defaults to GeoPackage over SpatiaLite4) register their geometry column names and SRS IDs in gpkg\_geometry\_columns9, sda-rs must introspect gpkg\_geometry\_columns at startup to bind {g} (wrapping BLOB values with GeomFromGPB({g}) in GeoPackage mode9). Because PostgreSQL folds unquoted aliases to lowercase, PostGIS output aliases matching mixed-case SDA column names (AreasymbolWkt84, MupolygonWktWgs84) must be double-quoted18. Similarly, because SSURGO mukey appears both as integer and quoted string literals in client queries20, \= \$1 comparisons against mukey should cast consistently (mukey::text \= \$1::text).

| SDA Tabular Function | Input Parameter | Return Column(s) | PostGIS Target Equivalent ({g} \= schema geom col) | SpatiaLite / GeoPackage Equivalent ({g} \= GeomFromGPB(geom) or SpatiaLite col) |
| :---- | :---- | :---- | :---- | :---- |
| SDA\_Get\_Mukey\_from\_intersection\_with\_WktWgs8418 | WKT (WGS84, 4326\)1 | mukey18 | SELECT DISTINCT mp.mukey FROM mupolygon mp WHERE mp.areasymbol \<\> 'US' AND ST\_Intersects(ST\_Transform(mp.{g}, 4326), ST\_GeomFromText(\$1, 4326)) | SELECT DISTINCT mp.mukey FROM mupolygon mp WHERE mp.areasymbol \<\> 'US' AND ST\_Intersects(ST\_Transform({g}, 4326), GeomFromText(?1, 4326)) |
| SDA\_Get\_Mukey\_from\_intersection\_with\_WktWm18 | WKT (Web Mercator, 3857\)18 | mukey18 | Same as above with ST\_Transform(mp.{g}, 3857\) and ST\_GeomFromText(\$1, 3857\) | Same as above with ST\_Transform({g}, 3857\) and GeomFromText(?1, 3857\) |
| SDA\_Get\_Areasymbol\_from\_intersection\_with\_WktWgs84 / \_WktWm18 | WKT (4326 / 3857\)18 | areasymbol18 | SELECT DISTINCT areasymbol FROM sapolygon WHERE ST\_Intersects(ST\_Transform({g}, {srid}), ST\_GeomFromText(\$1, {srid})) | SELECT DISTINCT areasymbol FROM sapolygon WHERE ST\_Intersects(ST\_Transform({g}, {srid}), GeomFromText(?1, {srid})) |
| SDA\_Get\_AreasymbolWKTWgs84\_from\_Areasymbol / \_WKTWm\_18 | areasymbol18 | AreasymbolWkt84 / AreasymbolWktWm18 | SELECT ST\_AsText(ST\_Transform({g}, {srid})) AS "AreasymbolWkt..." FROM sapolygon WHERE areasymbol \= \$1 | SELECT AsText(ST\_Transform({g}, {srid})) AS AreasymbolWkt... FROM sapolygon WHERE areasymbol \= ?1 |
| SDA\_Get\_AreasymbolWktWgs84\_from\_AreasymbolTable / \_WktWm\_18 | DeclareVarcharTable18 | areasymbol, AreasymbolWkt84 / AreasymbolWktWm18 | SELECT sa.areasymbol, ST\_AsText(ST\_Transform(sa.{g}, {srid})) AS "AreasymbolWkt..." FROM sapolygon sa JOIN {tbl} t ON sa.areasymbol \= t.v | SELECT sa.areasymbol, AsText(ST\_Transform(sa.{g}, {srid})) AS AreasymbolWkt... FROM sapolygon sa JOIN {tbl} t ON sa.areasymbol \= t.v |
| SDA\_Get\_MupolygonWktWgs84\_from\_Mukey / \_WktWm\_18 | mukey18 | MupolygonWktWgs84 / MupolygonWktWm18 | SELECT ST\_AsText(ST\_Transform({g}, {srid})) AS "MupolygonWkt..." FROM mupolygon WHERE mukey::text \= \$1::text | SELECT AsText(ST\_Transform({g}, {srid})) AS MupolygonWkt... FROM mupolygon WHERE CAST(mukey AS TEXT) \= CAST(?1 AS TEXT) |
| SDA\_Get\_MupolygonWktWgs84\_from\_MukeyTable / \_WktWm\_18 | DefineIntTable (mukey table)18 | mukey, MupolygonWktWgs84 / MupolygonWktWm18 | SELECT mp.mukey, ST\_AsText(ST\_Transform(mp.{g}, {srid})) AS "MupolygonWkt..." FROM mupolygon mp JOIN {tbl} t ON mp.mukey::int \= t.i | SELECT mp.mukey, AsText(ST\_Transform(mp.{g}, {srid})) AS MupolygonWkt... FROM mupolygon mp JOIN {tbl} t ON CAST(mp.mukey AS INTEGER) \= t.i |
| SDA\_Get\_MupointWktWgs84\_from\_Mukey / \_WktWm\_ & ...Table18 | mukey / DefineIntTable18 | (mukey,) MupointWktWgs84 / MupointWktWm18 | mupoint table equivalent of MupolygonWkt\* queries | mupoint table equivalent of MupolygonWkt\* queries |
| SDA\_Get\_MulineWktWgs84\_from\_Mukey / \_WktWm\_ & ...Table18 | mukey / DefineIntTable18 | (mukey,) MulineWktWgs84 / MulineWktWm18 | muline table equivalent of MupolygonWkt\* queries | muline table equivalent of MupolygonWkt\* queries |
| SDA\_Get\_Sapolygonkey\_from\_intersection\_with\_WktWgs84 / \_WktWm18 | WKT (4326 / 3857\)18 | sapolygonkey18 | SELECT sapolygonkey FROM sapolygon WHERE ST\_Intersects(ST\_Transform({g}, {srid}), ST\_GeomFromText(\$1, {srid})) | SELECT sapolygonkey FROM sapolygon WHERE ST\_Intersects(ST\_Transform({g}, {srid}), GeomFromText(?1, {srid})) |
| SDA\_Get\_Sapolygonkey\_from\_intersection\_with\_sastatusmap\_WktWgs84 / \_WktWm18 | WKT (4326 / 3857\)18 | sapolygonkey18 | SELECT sp.sapolygonkey FROM sapolygon sp JOIN sastatusmap sm ON sp.areasymbol \= sm.areasymbol WHERE ST\_Intersects(...) | Same join against sapolygon and sastatusmap |
| SDA\_Get\_Mupolygonkey\_from\_intersection\_with\_WktWgs84 / \_WktWm18 | WKT (4326 / 3857\)18 | mupolygonkey18 | SELECT mupolygonkey FROM mupolygon WHERE ST\_Intersects(...) | SELECT mupolygonkey FROM mupolygon WHERE ST\_Intersects(...) |
| SDA\_Get\_Mupointkey\_from\_intersection\_with\_WktWgs84 / \_WktWm18 | WKT (4326 / 3857\)18 | mupointkey18 | SELECT mupointkey FROM mupoint WHERE ST\_Intersects(...) | SELECT mupointkey FROM mupoint WHERE ST\_Intersects(...) |
| SDA\_Get\_Mulinekey\_from\_intersection\_with\_WktWgs84 / \_WktWm18 | WKT (4326 / 3857\)18 | mulinekey18 | SELECT mulinekey FROM muline WHERE ST\_Intersects(...) | SELECT mulinekey FROM muline WHERE ST\_Intersects(...) |
| SDA\_Get\_Featlinekey\_from\_intersection\_with\_WktWgs84 / \_WktWm18 | WKT (4326 / 3857\)18 | featlinekey18 | SELECT featlinekey FROM featline WHERE ST\_Intersects(...) | SELECT featlinekey FROM featline WHERE ST\_Intersects(...) |
| SDA\_Get\_Featpointkey\_from\_intersection\_with\_WktWgs84 / \_WktWm18 | WKT (4326 / 3857\)18 | featpointkey18 | SELECT featpointkey FROM featpoint WHERE ST\_Intersects(...) | SELECT featpointkey FROM featpoint WHERE ST\_Intersects(...) |

Note on mupolygon.areasymbol filtering: USDA-NRCS feature layers carry AREASYMBOL as a feature attribute21, enabling a direct mp.areasymbol \<\> 'US' STATSGO exclusion19. For local PostGIS or GeoPackage tables loaded without areasymbol on mupolygon, sda-rs should fall back to joining mapunit and legend on mukey and lkey to enforce legend.areasymbol \<\> 'US'18.

### **SQL Server CLR Spatial Methods**

Because full soilDB and py-soildb source trees were not available for exhaustive scanning, only .STArea() (documented in soilDB's STArea() \* 0.000247105 AS area\_ac example19) is directly evidenced in inspected artifacts; STGeomFromText, STIntersects, STIntersection, and STAsText represent standard SQL Server CLR methods targeted by the sda-rs lowering pass. Care is required with coordinate units: when queries multiply STArea() by 0.000247105 to derive acres from square meters19, ST\_Area must evaluate in meters-squared (via PostGIS ::geography cast or SpatiaLite transformation to a metric equal-area/UTM SRS) rather than raw EPSG:4326 square degrees.

| SQL Server CLR Expression | PostGIS Equivalent | SpatiaLite / GeoPackage Equivalent | SRID & Unit Notes |
| :---- | :---- | :---- | :---- |
| geometry::STGeomFromText(wkt, srid) | ST\_GeomFromText(wkt, srid) | GeomFromText(wkt, srid) | WGS84 queries use SRID 43261; Web Mercator uses 3857\. |
| g.STIntersects(h) \= 1 | ST\_Intersects(g, h) | ST\_Intersects(g, h) \= 1 | Align mismatched SRIDs via ST\_Transform before intersection. |
| g.STIntersection(h) | ST\_Intersection(g, h) | ST\_Intersection(g, h) | Returns intersection geometry for area or WKT chaining. |
| g.STArea() | ST\_Area(g::geography) (for 4326 meters-squared) or ST\_Area(g) | ST\_Area(ST\_Transform(g, {metric\_srid})) (if 4326\) or ST\_Area(g) | Metric area required for 0.000247105 acre conversions19. |
| g.STAsText() | ST\_AsText(g) | AsText(g) | Emits OGC Well-Known Text. |

## **3\. Wire Protocol, Request Dispatch, and Metadata Mapping**

### **POST /Tabular/post.rest Request Contract**

Official USDA-NRCS Web Service Help documentation defines https\://SDMDataAccess.sc.egov.usda.gov/Tabular/post.rest as a REST/POST-only endpoint1. For tabular SQL execution (SERVICE=query, REQUEST=query, both optional), callers submit QUERY (required: one or more SELECT statements with optional macros, or a single Shape command) and optional FORMAT (XML, JSON, JSON+COLUMNNAME, JSON+COLUMNNAME+METADATA, or \~, defaulting to XML)2. The same endpoint multiplexes SERVICE=aoi (REQUEST=create, returning {"id": 12345}), SERVICE=interpretation, and SERVICE=report2.  
Because the HTTP client implementations in soilDB::SDA\_query and py-soildb SDAClient could not be inspected in the retrieved sources, sda-rs should accept both application/x-www-form-urlencoded and application/json request bodies on /Tabular/post.rest, matching field names case-insensitively (QUERY/query, FORMAT/format). When FORMAT=\~ is supplied, the server must expand any macros and return the resulting SQL string as text/plain without executing it2.

### **Response Schemas and Multi-Result-Set Layout**

The official webservicehelp.aspx specification defines the row semantics of the five FORMAT modes summarized below2. Exact outer JSON container keys (Table, Table1, Table2), empty result-set payloads, and exact XML \<NewDataSet\> element structures are not specified in webservicehelp.aspx2 and were absent from retrieved soilDB .parseSDA / py-soildb artifacts; they must be verified against live post.rest captures or client source before production release.

| FORMAT Parameter | Content-Type | Verified Structural Contract (webservicehelp.aspx) | Unverified Wrapper / Edge Behavior (Confirm via Live Capture) |
| :---- | :---- | :---- | :---- |
| XML (default) | application/xml | Default query output and the only execution format supporting Shape commands2. | Exact .NET DataSet XML element nesting (\<NewDataSet\>\<Table\>...\</Table\>\</NewDataSet\>) and empty-set representation. |
| JSON | application/json | Data rows serialized in JSON syntax without column-name or metadata header rows2. | Top-level object wrapper keys ({"Table": \[...\], "Table1": \[...\]}) and empty-set payload ({} vs {"Table": \[\]}). |
| JSON+COLUMNNAME | application/json | Row 1 (0-indexed index 0\) contains column names; Row 2 onward contain data rows2. | Multi-result-set key naming (Table, Table1, Table2) and empty-set handling. |
| JSON+COLUMNNAME+METADATA | application/json | Row 1 (index 0\) contains column names; Row 2 (index 1\) contains per-column metadata strings; Row 3 onward contain data rows2. | Complete GetSchemaTable() key set beyond ProviderType, width, NumericPrecision, NumericScale2. |
| \~ | text/plain | Echoes the submitted SQL query and full macro expansions without executing it2. | Line-ending conventions (\\r\\n vs \\n). |

### **SQL Server Type Metadata Mapping**

In JSON+COLUMNNAME+METADATA, Row 2 (the second array row, 0-indexed Row 1\) contains one string per column composed of comma-separated name=value pairs derived from a subset of SQL Server's SqlDataReader.GetSchemaTable()2. Official USDA-NRCS documentation explicitly identifies five metadata components: ProviderType (an integer from SQL Server's SqlDbType enumeration), column width (for char and varchar), numeric type, NumericPrecision (e.g., NumericPrecision=255), and NumericScale (e.g., NumericScale=255)2. Whether DataTypeName is emitted verbatim as a key or inferred by clients from ProviderType could not be verified without .parseSDA / py-soildb code or a live JSON+COLUMNNAME+METADATA sample. Accordingly, the type mapping table below pairs verified webservicehelp.aspx attributes2 with provisional SQL Server type mappings flagged for live verification.

| PostgreSQL Type | SQLite Affinity | Provisional SQL Server Type (DataTypeName) | Provisional ProviderType (SqlDbType Enum) | Verified webservicehelp.aspx Metadata Rules |
| :---- | :---- | :---- | :---- | :---- |
| INT4 (integer) | INTEGER | int *(verify)* | Integer code for SqlDbType.Int *(verify enum int)* | ProviderType, NumericPrecision, NumericScale |
| INT8 (bigint) | INTEGER | bigint *(verify)* | Integer code for SqlDbType.BigInt *(verify enum int)* | ProviderType, NumericPrecision, NumericScale |
| FLOAT8 (double precision) | REAL | float *(verify)* | Integer code for SqlDbType.Float *(verify enum int)* | ProviderType, NumericPrecision, NumericScale |
| NUMERIC(p, s) | REAL / NUMERIC | decimal / numeric *(verify)* | Integer code for SqlDbType.Decimal *(verify enum int)* | ProviderType, NumericPrecision=p, NumericScale=s2 |
| VARCHAR(n) / TEXT | TEXT | varchar / nvarchar *(verify)* | Integer code for VarChar / NVarChar *(verify enum int)* | ProviderType, column width (for char/varchar)2 |
| BOOL | INTEGER | bit *(verify)* | Integer code for SqlDbType.Bit *(verify enum int)* | ProviderType |
| TIMESTAMPTZ / TIMESTAMP | TEXT | datetime2 / datetime *(verify)* | Integer code for DateTime / DateTime2 *(verify enum int)* | ProviderType |
| BYTEA | BLOB | varbinary *(verify)* | Integer code for SqlDbType.VarBinary *(verify enum int)* | ProviderType, column width |

## **4\. T-SQL Benchmark and Compatibility Suite**

Because complete raw .R and .py source files from soilDB and py-soildb were not available for exhaustive extraction, Queries 1–11 below are verbatim queries or documented fragments retrieved from soilDB vignettes/discussions, USDA-NRCS documentation, and public SDA examples1, while Queries 12–20 are explicitly labeled **Spec Template**—synthetic validation queries grounded in documented soilDB/py-soildb schemas and patterns2 designed to exercise @aoi variables, \#temp tables, CROSS APPLY, OUTER APPLY, STRING\_AGG, ISNULL, IIF, and multi-result-set execution.

| \# | Provenance | T-SQL Benchmark Query | Target AST Passes Exercised |
| :---- | :---- | :---- | :---- |
| 1 | Verbatim (soilDB discussion23) | SELECT areasymbol, sapubstatuscode, iscomplete FROM sastatusmap | Baseline tabular pass-through |
| 2 | Verbatim (soilDB SDA tutorial19) | SELECT mukey, muname FROM mapunit WHERE mukey IN ( SELECT \* from SDA\_Get\_Mukey\_from\_intersection\_with\_WktWgs84('point(-121.77100 37.368402)') ) | Pass 4 (SDA\_Get\_Mukey\_\* IN-subquery) |
| 3 | Verbatim (USDA SDA / Esri1) | SELECT mukey FROM SDA\_Get\_Mukey\_from\_intersection\_with\_WktWgs84('point(-93.65 42.03)') | Pass 4 (SDA\_Get\_Mukey\_\* point WKT) |
| 4 | Verbatim (NRCS NASIS report24) | SELECT mukey AS MUKEY, muname AS Map\_unit\_name FROM mapunit WHERE mukey IN (SELECT \* from SDA\_Get\_Mukey\_from\_intersection\_with\_WktWgs84('point (-121.77100 37.368402)')) | Pass 4 \+ alias case preservation |
| 5 | Verbatim (farmOS / NRCS REST25) | SELECT muname, musym, nationalmusym FROM SDA\_Get\_Mukey\_from\_intersection\_with\_WktWgs84('point(-93.65 42.03)') K LEFT JOIN mapunit MU ON K.mukey \= MU.mukey | Pass 4 TVF LEFT JOIN |
| 6 | Verbatim (NRCS AdvancedQueries18) | select \* from SDA\_Get\_Areasymbol\_from\_intersection\_with\_WktWgs84('polygon(( \-121.77100 37.368402, \-121.77100 37.373473, \-121.76000 37.373473, \-121.76000 37.368402, \-121.77100 37.368402))') | Pass 4 (Areasymbol polygon WGS84) |
| 7 | Verbatim (NRCS AdvancedQueries18) | select \* from SDA\_Get\_Mukey\_from\_intersection\_with\_WktWm('polygon(( \-13555610.9782664 4490483.16765171, \-13555610.9782664 4491391.31746713, \-13554135.9341103 4491391.31746713, \-13554135.9341103 4490483.16765171, \-13555610.9782664 4490483.16765171))') | Pass 4 (EPSG:3857 Web Mercator transform) |
| 8 | Verbatim (NRCS AdvancedQueries18) | select \* from SDA\_Get\_Mupolygonkey\_from\_intersection\_with\_WktWgs84('polygon(( \-121.77100 37.368402, \-121.77100 37.373473, \-121.76000 37.373473, \-121.76000 37.368402, \-121.77100 37.368402))') | Pass 4 (mupolygonkey TVF) |
| 9 | Verbatim (NRCS AdvancedQueries18) | select S.mukey, M.musym from SDA\_Get\_Mukey\_from\_intersection\_with\_WktWm('polygon(( \-13555610.9782664 4490483.16765171, \-13555610.9782664 4491391.31746713, \-13554135.9341103 4491391.31746713, \-13554135.9341103 4490483.16765171, \-13555610.9782664 4490483.16765171))') as S, legend as L, mapunit M where M.mukey \= S.mukey and M.lkey \= L.lkey and L.areasymbol \= 'CA646' | Pass 4 \+ implicit comma joins |
| 10 | Verbatim (public SDA example20) | SELECT Top 3 G.MupolygonWktWgs84 as geom, '462594' as mukey from SDA\_Get\_MupolygonWktWgs84\_from\_Mukey('462594') as G | TOP-\>LIMIT \+ Pass 4 (MupolygonWktWgs84) |
| 11 | Reconstructed from fragment (soilDB tutorial19) | SELECT mukey, mupolygongeo.STArea() \* 0.000247105 AS area\_ac FROM mupolygon WHERE mukey \= '418254' | Pass 3 (.STArea() metric scaling) |
| 12 | Spec Template (py-soildb / soilDB join pattern19) | SELECT m.mukey, m.muname, m.musym, c.compname, c.comppct\_r FROM mapunit m INNER JOIN legend l ON m.lkey \= l.lkey INNER JOIN component c ON m.mukey \= c.mukey WHERE l.areasymbol \!= 'US' AND c.compname \= 'Miami' AND c.majcompflag \= 'Yes' ORDER BY m.mukey, c.comppct\_r DESC | Multi-table join \+ STATSGO filter |
| 13 | Spec Template (soilDB fetchLDM filter26) | SELECT TOP (10) pedon\_key, CASE WHEN corr\_name IS NOT NULL THEN LOWER(corr\_name) ELSE LOWER(samp\_name) END AS taxon FROM lab\_pedon WHERE area\_code \= 'CA630' | Pass 6 (TOP (N)) \+ CASE |
| 14 | Spec Template (@aoi variable & .STIntersects) | DECLARE @aoi geometry; SELECT @aoi \= geometry::STGeomFromText('POLYGON((-93.65 42.03, \-93.64 42.03, \-93.64 42.04, \-93.65 42.04, \-93.65 42.03))', 4326); SELECT mukey, mupolygongeo.STIntersection(@aoi).STArea() AS area\_m2 FROM mupolygon WHERE mupolygongeo.STIntersects(@aoi) \= 1 | Pass 1 (@aoi) \+ Pass 3 (chained CLR \+ \= 1\) |
| 15 | Spec Template (SELECT ... INTO \#temp) | SELECT mukey, musym INTO \#target\_mu FROM mapunit WHERE lkey IN (SELECT lkey FROM legend WHERE areasymbol \= 'IN005'); SELECT t.mukey, c.cokey, c.compname FROM \#target\_mu t INNER JOIN component c ON t.mukey \= c.mukey | Pass 2 (\#temp table batch rewrite) |
| 16 | Spec Template (CROSS APPLY TOP 1\) | SELECT m.mukey, m.muname, dom.compname, dom.comppct\_r FROM mapunit m CROSS APPLY (SELECT TOP 1 c.compname, c.comppct\_r FROM component c WHERE c.mukey \= m.mukey ORDER BY c.comppct\_r DESC) dom WHERE m.mukey IN ('462594', '418254') | Pass 5 (CROSS APPLY \-\> LATERAL / scalar rewrite) |
| 17 | Spec Template (OUTER APPLY) | SELECT c.cokey, c.compname, surf.hzdept\_r, surf.claytotal\_r FROM component c OUTER APPLY (SELECT TOP 1 ch.hzdept\_r, ch.claytotal\_r FROM chorizon ch WHERE ch.cokey \= c.cokey ORDER BY ch.hzdept\_r ASC) surf WHERE c.mukey \= '462594' | Pass 5 (OUTER APPLY \-\> LEFT JOIN LATERAL) |
| 18 | Spec Template (STRING\_AGG, ISNULL, IIF) | SELECT m.mukey, ISNULL(m.muname, 'Unknown') AS muname, IIF(MAX(c.comppct\_r) \>= 85, 'Consociation', 'Complex') AS mu\_kind, STRING\_AGG(c.compname, '; ') AS comps FROM mapunit m JOIN component c ON m.mukey \= c.mukey WHERE m.mukey \= '462594' GROUP BY m.mukey, m.muname | Pass 6 (ISNULL, IIF, STRING\_AGG) |
| 19 | Spec Template (SDA \~DeclareIntTable\~ macro18) | \~DeclareIntTable(@mu)\~ INSERT INTO @mu VALUES (462594), (418254); SELECT mukey, MupolygonWktWgs84 FROM SDA\_Get\_MupolygonWktWgs84\_from\_MukeyTable(@mu) | Macro pre-pass \+ Pass 4 (\*Table UDF) |
| 20 | Spec Template (Multi-result-set batch2) | SELECT areasymbol, areaname FROM legend WHERE areasymbol \= 'IA109'; SELECT TOP 5 mukey, muname FROM mapunit WHERE lkey IN (SELECT lkey FROM legend WHERE areasymbol \= 'IA109') | Multi-statement Table/Table1 response muxing |

#### **Works cited**

> 1. Find SSURGO data for a location \- Esri Community, [https\://community.esri.com/en/discussion/1496968/find-ssurgo-data-for-a-location](https://community.esri.com/en/discussion/1496968/find-ssurgo-data-for-a-location)  
> 2. [https\://sdmdataaccess.nrcs.usda.gov/webservicehelp.aspx](https://sdmdataaccess.nrcs.usda.gov/webservicehelp.aspx)  
> 3. Create a database from SSURGO Exports — createSSURGO • soilDB, [https\://ncss-tech.github.io/soilDB/reference/createSSURGO.html](https://ncss-tech.github.io/soilDB/reference/createSSURGO.html)  
> 4. SSURGO Portal \| Natural Resources Conservation Service, [https\://www\.nrcs.usda.gov/resources/data-and-reports/ssurgo-portal](https://www.nrcs.usda.gov/resources/data-and-reports/ssurgo-portal)  
> 5. polyglot/crates/polyglot-sql/README.md at main \- GitHub, [https\://github.com/tobilg/polyglot/blob/main/crates/polyglot-sql/README.md](https://github.com/tobilg/polyglot/blob/main/crates/polyglot-sql/README.md)  
> 6. "" Search \- Rust \- Docs.rs, [https\://docs.rs/polyglot-sql/latest/polyglot\_sql/expressions/index.html?search=](https://docs.rs/polyglot-sql/latest/polyglot_sql/expressions/index.html?search)  
> 7. statements.rs \- source \- Docs.rs, [https\://docs.rs/polyglot-sql/latest/src/polyglot\_sql/dialects/normalization/statements.rs.html](https://docs.rs/polyglot-sql/latest/src/polyglot_sql/dialects/normalization/statements.rs.html)  
> 8. Soil Survey Geographic Database (SSURGO) Portal User Guide, [https\://www\.nrcs.usda.gov/sites/default/files/2024-11/SSURGO-Portal-User-Guide.pdf](https://www.nrcs.usda.gov/sites/default/files/2024-11/SSURGO-Portal-User-Guide.pdf)  
> 9. Getting Started With GeoPackage, [https\://www\.geopackage.org/guidance/getting-started.html](https://www.geopackage.org/guidance/getting-started.html)  
> 10. temporal.rs \- source \- Docs.rs, [https\://docs.rs/polyglot-sql/latest/src/polyglot\_sql/dialects/normalization/temporal.rs.html](https://docs.rs/polyglot-sql/latest/src/polyglot_sql/dialects/normalization/temporal.rs.html)  
> 11. "std::vec" Search \- Rust \- Docs.rs, [https\://docs.rs/polyglot-sql/latest/polyglot\_sql/expressions/index.html?search=std::vec](https://docs.rs/polyglot-sql/latest/polyglot_sql/expressions/index.html?search=std::vec)  
> 12. tobilg/polyglot: Rust/Wasm-powered SQL transpiler for more than 30, [https\://github.com/tobilg/polyglot](https://github.com/tobilg/polyglot)  
> 13. polyglot-sql-r/inst/COPYRIGHTS at main \- GitHub, [https\://github.com/StrategicProjects/polyglot-sql-r/blob/main/inst/COPYRIGHTS](https://github.com/StrategicProjects/polyglot-sql-r/blob/main/inst/COPYRIGHTS)  
> 14. TransformKind in polyglot\_sql::query\_analysis \- Rust \- Docs.rs, [https\://docs.rs/polyglot-sql/latest/polyglot\_sql/query\_analysis/enum.TransformKind.html](https://docs.rs/polyglot-sql/latest/polyglot_sql/query_analysis/enum.TransformKind.html)  
> 15. polyglot-sql \- crates.io: Rust Package Registry, [https\://crates.io/crates/polyglot-sql/0.3.5](https://crates.io/crates/polyglot-sql/0.3.5)  
> 16. sqlglot-rust \- Lib.rs, [https\://lib.rs/crates/sqlglot-rust](https://lib.rs/crates/sqlglot-rust)  
> 17. polyglot: Rust/Wasm SQL Transpiler for 33 Dialects \- tobilg.com, [https\://tobilg.com/posts/introducing-polyglot-a-rust-wasm-sql-transpilation-library/](https://tobilg.com/posts/introducing-polyglot-a-rust-wasm-sql-transpilation-library/)  
> 18. Advanced Queries \- Soil Data Access \- USDA, [https\://sdmdataaccess.nrcs.usda.gov/documents/AdvancedQueries.html](https://sdmdataaccess.nrcs.usda.gov/documents/AdvancedQueries.html)  
> 19. Soil Data Access (SDA) Tutorial \- ncss-tech.github.io, [https\://ncss-tech.github.io/AQP/soilDB/SDA-tutorial.html](https://ncss-tech.github.io/AQP/soilDB/SDA-tutorial.html)  
> 20. loop through sql in r \- Stack Overflow, [https\://stackoverflow.com/questions/52877564/loop-through-sql-in-r](https://stackoverflow.com/questions/52877564/loop-through-sql-in-r)  
> 21. Layer: MUPolygon (ID: 0\) \- USDA, [https\://apps.geo.fpac.usda.gov/nrcs-geodata/rest/services/soils/cg\_soils/FeatureServer/0](https://apps.geo.fpac.usda.gov/nrcs-geodata/rest/services/soils/cg_soils/FeatureServer/0)  
> 22. GitHub \- brownag/py-soildb: Python client for USDA-NRCS Soil Data, [https\://github.com/brownag/py-soildb](https://github.com/brownag/py-soildb)  
> 23. List of non-SSURGO (mapping incomplete/NOTCOM) counties? \#319, [https\://github.com/ncss-tech/soilDB/discussions/319](https://github.com/ncss-tech/soilDB/discussions/319)  
> 24. MUKEY by Point Intersect \- USDA, [https\://nasis.sc.egov.usda.gov/NasisReportsWebSite/limsreport.aspx?report\_name=SDA-PointIntersect](https://nasis.sc.egov.usda.gov/NasisReportsWebSite/limsreport.aspx?report_name=SDA-PointIntersect)  
> 25. Using SSURGO and improving usability \- GOATech, [https\://forum.goatech.org/t/using-ssurgo-and-improving-usability/1466](https://forum.goatech.org/t/using-ssurgo-and-improving-usability/1466)  
> 26. soilDB/R/fetchLDM.R at master \- GitHub, [https\://github.com/ncss-tech/soilDB/blob/master/R/fetchLDM.R](https://github.com/ncss-tech/soilDB/blob/master/R/fetchLDM.R)