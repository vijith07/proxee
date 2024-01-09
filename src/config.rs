// src/config.rs

use serde_derive::Deserialize;
use std::fs;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub proxy: ProxyConfig,
    pub load_balancing: LoadBalancingConfig,
    pub backend_servers: Vec<BackendServer>,
    pub metrics: Metrics,
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

#[derive(Deserialize,Clone)]
pub struct Metrics{
    pub listen_port: u16,
    pub route: String,
    pub allowed_ips: Vec<String>,
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(file_path)?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

// get metrics config
// if not set, use default values
impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            listen_port: 8080,
            route: "/metrics".to_string(),
            allowed_ips: vec!["127.0.0.1".to_string()],
        }
    }
}

// get load balancing config
// if not set, use default values
impl Default for LoadBalancingConfig {
    fn default() -> Self {
        LoadBalancingConfig {
            method: "round_robin".to_string(),
        }
    }
}
