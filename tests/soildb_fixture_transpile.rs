#[test]
fn captured_soildb_queries_transpile_to_expected_sql() {
    for (name, source, expected) in [
        (
            "sda-spatial-mukey",
            include_str!("fixtures/soildb/sda-spatial-mukey.sql"),
            include_str!("fixtures/soildb/sda-spatial-mukey.sqlite.sql"),
        ),
        (
            "sda-spatial-areasymbol",
            include_str!("fixtures/soildb/sda-spatial-areasymbol.sql"),
            include_str!("fixtures/soildb/sda-spatial-areasymbol.sqlite.sql"),
        ),
        (
            "get-sda-muaggatt",
            include_str!("fixtures/soildb/get-sda-muaggatt.sql"),
            include_str!("fixtures/soildb/get-sda-muaggatt.sqlite.sql"),
        ),
    ] {
        let generated = sda_rs::transpiler::transpile(source)
            .unwrap_or_else(|error| panic!("{name} failed to transpile: {error}"));
        assert_eq!(generated, [expected.trim()], "{name}");
    }
}
