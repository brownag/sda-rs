use clap::Parser;
use std::{
    env,
    ffi::OsString,
    io::{Error, ErrorKind},
};

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

/// Extract the server bind address without adding a field to `Config`.
/// The existing all-interface default is preserved for compatibility.
pub fn extract_bind_address(
    args: impl IntoIterator<Item = OsString>,
) -> Result<(Vec<OsString>, String), Error> {
    let mut args = args.into_iter();
    let program = args.next().unwrap_or_else(|| OsString::from("sda-rs"));
    let mut filtered = vec![program];
    let mut bind_address = env::var("SDA_BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".into());

    while let Some(argument) = args.next() {
        if argument == "--bind-address" {
            bind_address = args
                .next()
                .ok_or_else(|| {
                    Error::new(ErrorKind::InvalidInput, "--bind-address requires a value")
                })?
                .into_string()
                .map_err(|_| Error::new(ErrorKind::InvalidInput, "bind address must be UTF-8"))?;
        } else if let Some(value) = argument
            .to_str()
            .and_then(|argument| argument.strip_prefix("--bind-address="))
        {
            bind_address = value.to_owned();
        } else {
            filtered.push(argument);
        }
    }

    if bind_address.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "bind address cannot be empty",
        ));
    }
    Ok((filtered, bind_address))
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

    #[test]
    fn extracts_bind_address_without_changing_existing_config_arguments() {
        let (args, bind_address) = super::extract_bind_address(
            ["sda-rs", "--port", "9090", "--bind-address", "127.0.0.1"]
                .into_iter()
                .map(std::ffi::OsString::from),
        )
        .unwrap();
        assert_eq!(
            args,
            ["sda-rs", "--port", "9090"]
                .into_iter()
                .map(std::ffi::OsString::from)
                .collect::<Vec<_>>()
        );
        assert_eq!(bind_address, "127.0.0.1");
    }
}
