use serde::Deserialize;
use std::{
    io::{Error, ErrorKind, Result},
    path::Path,
};

#[derive(Deserialize)]
pub struct Config {
    pub listen_address: String,
    pub listen_port: u16,
    pub auth_return_header_name: Option<String>,
    pub debug: bool,
    pub reverse_proxy: ReverseProxyConfig,
    pub https: HttpsConfig,
    pub session: SessionConfig,
}

#[derive(Deserialize)]
pub struct ReverseProxyConfig {
    pub enabled: bool,
    pub proxy_address: Option<String>,
}

#[derive(Deserialize)]
pub struct SessionConfig {
    pub cookie_name: String,
    pub session_expire: u64,
    pub storage: String,
    pub redis_conn: Option<String>
}

#[derive(Deserialize)]
pub struct HttpsConfig {
    pub enabled: bool,
    pub cert: Option<String>,
    pub key: Option<String>,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let string = std::fs::read_to_string(path)?;
        match toml::from_str(&string) {
            Ok(config) => Ok(config),
            Err(err) => Err(Error::new(ErrorKind::InvalidData, err)),
        }
    }
}
