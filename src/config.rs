use clap::Parser;

/// Runtime configuration for the SDA service.
#[derive(Clone, Debug, Parser)]
#[command(name = "sda-rs", version)]
pub struct Config {
    /// SQLite database path or backend URL.
    #[arg(long, env = "SDA_BACKEND_URL", default_value = "sda.gpkg")]
    pub backend_url: String,

    /// HTTP port to listen on.
    #[arg(long, env = "SDA_PORT", default_value_t = 8080)]
    pub port: u16,

    /// Optional path to a SpatiaLite extension.
    #[arg(long, env = "SDA_SPATIAL_EXTENSION")]
    pub spatial_extension: Option<String>,

    /// Optional override for the GeoPackage geometry column.
    #[arg(long, env = "SDA_GEOM_COLUMN")]
    pub geometry_column: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use clap::Parser;

    #[test]
    fn defaults_and_cli_overrides() {
        let defaults = Config::try_parse_from(["sda-rs"]).unwrap();
        assert_eq!(defaults.backend_url, "sda.gpkg");
        assert_eq!(defaults.port, 8080);
        assert_eq!(defaults.spatial_extension, None);
        assert_eq!(defaults.geometry_column, None);

        let overridden = Config::try_parse_from([
            "sda-rs",
            "--backend-url",
            "data.sqlite",
            "--port",
            "9090",
            "--spatial-extension",
            "/usr/lib/mod_spatialite.so",
            "--geometry-column",
            "shape",
        ])
        .unwrap();
        assert_eq!(overridden.backend_url, "data.sqlite");
        assert_eq!(overridden.port, 9090);
        assert_eq!(
            overridden.spatial_extension.as_deref(),
            Some("/usr/lib/mod_spatialite.so")
        );
        assert_eq!(overridden.geometry_column.as_deref(), Some("shape"));
    }
}
