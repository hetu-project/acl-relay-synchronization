use crate::common::error;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub db_url: String,
    pub max_connect_pool: u32,
    pub min_connect_pool: u32,
    pub connect_timeout: u64,
    pub acquire_timeout: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct IndexdbBackendConfig {
    pub invite_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WakuConfig {
    pub node_url: String,
    pub send_api: String,
    pub pubsub_topic: String,
    pub content_topic: String,
    pub node_addr: String,
    pub cluster_id: String,
    pub shared: String,
    pub waku_bin: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NostrConfig {
    pub priv_key: String,
    pub ws_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub indexdb_backend: IndexdbBackendConfig,
    pub waku: WakuConfig,
    pub nostr: NostrConfig,
}

impl Config {
    pub fn load_config(path: PathBuf) -> error::Result<Config> {
        let p: &Path = path.as_ref();
        let config_yaml = std::fs::read_to_string(p).map_err(|err| match err {
            e @ std::io::Error { .. } if e.kind() == std::io::ErrorKind::NotFound => {
                error::Error::ConfigMissing(path)
            }
            _ => err.into(),
        })?;

        let config: Config =
            serde_yaml::from_str(&config_yaml).map_err(error::Error::SerializationError)?;
        Ok(config)
    }
}
