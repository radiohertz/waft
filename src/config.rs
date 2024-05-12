//! Config for the server

use serde::Deserialize;

/// port to default to
const DEFAULT_SERVER_PORT: u16 = 3000;

/// port to default to
const DEFAULT_RTMP_SERVER_PORT: u16 = 1935;

/// Configure the runtime of the server
#[derive(Default, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Config {
    /// Port on which the server will run on
    port: Option<u16>,
    /// Password for auth
    key: Option<String>,
    /// rtmp port
    rtmp_port: Option<u16>,
    /// stream key
    stream_key: String,
    /// title of the stream
    title: Option<String>,
}

impl Config {
    /// Tries to load config from toml file specified by user
    pub fn load_from_file(path: &std::path::PathBuf) -> anyhow::Result<Self> {
        tracing::info!("Loading config from {path:?}");
        let bytes = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&bytes)?;
        assert!(
            config.rtmp_port != config.port,
            "HTTP server port and RTMP port cant be the same."
        );

        Ok(config)
    }

    /// Get the port if specified in config or default
    pub fn port(&self) -> u16 {
        self.port.unwrap_or(DEFAULT_SERVER_PORT)
    }

    pub fn rtmp_port(&self) -> u16 {
        self.rtmp_port.unwrap_or(DEFAULT_RTMP_SERVER_PORT)
    }

    pub fn stream_key(&self) -> &str {
        &self.stream_key
    }

    pub fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    pub fn _set_key(&mut self, key: Option<String>) {
        tracing::info!("Config: setting a new key: {key:?}");
        self.key = key;
    }

    pub fn title(&self) -> Option<String> {
        self.title.clone()
    }
}
