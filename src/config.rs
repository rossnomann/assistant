use carapax::types::UserId;
use serde::Deserialize;
use serde_yaml::Error as YamlError;
use std::{error::Error, fmt, fs::read_to_string, io::Error as IoError, net::SocketAddr, path::Path};

#[derive(Clone, Deserialize)]
pub struct Config {
    pub token: String,
    pub database_url: String,
    pub session_url: String,
    pub users: Vec<UserId>,
    pub webhook_address: Option<SocketAddr>,
    pub webhook_path: Option<String>,
}

impl Config {
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let data = read_to_string(path).map_err(ConfigError::Read)?;
        serde_yaml::from_str(&data).map_err(ConfigError::Parse)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Parse(YamlError),
    Read(IoError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::ConfigError::*;
        match self {
            Parse(err) => write!(out, "failed to parse config: {}", err),
            Read(err) => write!(out, "failed to read config: {}", err),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::ConfigError::*;
        Some(match self {
            Parse(err) => err,
            Read(err) => err,
        })
    }
}
