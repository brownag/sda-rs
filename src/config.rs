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

pub const DEFAULT_BODY_LIMIT: usize = 2 * 1024 * 1024;
pub const MAX_BODY_LIMIT: usize = 64 * 1024 * 1024;

/// Extract the request body limit without adding a field to `Config`.
pub fn extract_body_limit(
    args: impl IntoIterator<Item = OsString>,
) -> Result<(Vec<OsString>, usize), Error> {
    extract_body_limit_with_env(args, env::var_os("SDA_BODY_LIMIT"))
}

fn extract_body_limit_with_env(
    args: impl IntoIterator<Item = OsString>,
    env_limit: Option<OsString>,
) -> Result<(Vec<OsString>, usize), Error> {
    let mut args = args.into_iter();
    let program = args.next().unwrap_or_else(|| OsString::from("sda-rs"));
    let mut filtered = vec![program];
    let mut body_limit = env_limit
        .map(|value| parse_body_limit(&value))
        .transpose()?
        .unwrap_or(DEFAULT_BODY_LIMIT);

    while let Some(argument) = args.next() {
        if argument == "--body-limit" {
            body_limit = parse_body_limit(&args.next().ok_or_else(|| {
                Error::new(ErrorKind::InvalidInput, "--body-limit requires a value")
            })?)?;
        } else if let Some(value) = argument
            .to_str()
            .and_then(|argument| argument.strip_prefix("--body-limit="))
        {
            body_limit = parse_body_limit(OsString::from(value).as_os_str())?;
        } else {
            filtered.push(argument);
        }
    }
    Ok((filtered, body_limit))
}

fn parse_body_limit(value: &std::ffi::OsStr) -> Result<usize, Error> {
    let value = value
        .to_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "body limit must be UTF-8"))?;
    let limit = value
        .parse::<usize>()
        .map_err(|_| Error::new(ErrorKind::InvalidInput, "body limit must be an integer"))?;
    if limit == 0 || limit > MAX_BODY_LIMIT {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("body limit must be between 1 and {MAX_BODY_LIMIT} bytes"),
        ));
    }
    Ok(limit)
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

    #[test]
    fn body_limit_defaults_parses_cli_and_environment_and_validates_range() {
        let (args, limit) =
            super::extract_body_limit_with_env(["sda-rs"].map(std::ffi::OsString::from), None)
                .unwrap();
        assert_eq!(args, ["sda-rs"].map(std::ffi::OsString::from));
        assert_eq!(limit, super::DEFAULT_BODY_LIMIT);

        let (_, cli_limit) = super::extract_body_limit_with_env(
            ["sda-rs", "--body-limit=2048", "--port", "9090"].map(std::ffi::OsString::from),
            None,
        )
        .unwrap();
        assert_eq!(cli_limit, 2048);
        let (_, env_limit) = super::extract_body_limit_with_env(
            ["sda-rs"].map(std::ffi::OsString::from),
            Some(std::ffi::OsString::from("4096")),
        )
        .unwrap();
        assert_eq!(env_limit, 4096);
        let (_, cli_precedence) = super::extract_body_limit_with_env(
            ["sda-rs", "--body-limit", "8192"].map(std::ffi::OsString::from),
            Some(std::ffi::OsString::from("4096")),
        )
        .unwrap();
        assert_eq!(cli_precedence, 8192);
        let (_, maximum) = super::extract_body_limit_with_env(
            ["sda-rs", "--body-limit", "67108864"].map(std::ffi::OsString::from),
            None,
        )
        .unwrap();
        assert_eq!(maximum, super::MAX_BODY_LIMIT);

        for invalid in ["0", "nope", "67108865"] {
            assert!(super::extract_body_limit_with_env(
                ["sda-rs", "--body-limit", invalid].map(std::ffi::OsString::from),
                None
            )
            .is_err());
        }
        assert!(super::extract_body_limit_with_env(
            ["sda-rs"].map(std::ffi::OsString::from),
            Some(std::ffi::OsString::from("0")),
        )
        .is_err());
    }
}
