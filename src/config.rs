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
    pub fn load_env() -> anyhow::Result<Self> {
        let port = std::env::var("WAFT_PORT")?.parse::<u16>()?;
        let room_key = std::env::var("WAFT_ROOM_KEY")?;
        let rtmp_port = std::env::var("WAFT_RTMP_PORT")?.parse::<u16>()?;
        let stream_key = std::env::var("WAFT_STREAM_KEY")?;
        let title = std::env::var("WAFT_TITLE")?;

        Ok(Self {
            port: Some(port),
            key: Some(room_key),
            rtmp_port: Some(rtmp_port),
            title: Some(title),
            stream_key,
        })
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
