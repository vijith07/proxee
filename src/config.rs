// src/config.rs

use serde_derive::Deserialize;
use std::fs;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub proxy: ProxyConfig,
    pub load_balancing: LoadBalancingConfig,
    pub backend_servers: Vec<BackendServer>,
}

#[derive(Deserialize)]
pub struct ProxyConfig {
    pub listen_address: String,
    pub listen_port: u16,
}

#[derive(Deserialize)]
pub struct LoadBalancingConfig {
    pub method: String,
}

#[derive(Deserialize,Clone)]
pub struct BackendServer {
    pub address: String,
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(file_path)?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}
