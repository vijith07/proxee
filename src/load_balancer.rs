use crate::config::BackendServer;
use rand::Rng;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::atomic::{AtomicUsize, Ordering},
};

pub enum LoadBalancingMethod {
    RoundRobin = 0,
    Random = 1,
    IPHash = 2,
    // LeastConnections,
}

pub struct LoadBalancer {
    method: LoadBalancingMethod,
    servers: Vec<BackendServer>,
    current_server_index: AtomicUsize,
}

impl LoadBalancer {
    pub fn new(
        method: LoadBalancingMethod,
        servers: Vec<BackendServer>,
        current: usize,
    ) -> LoadBalancer {
        LoadBalancer {
            method,
            servers,
            current_server_index: AtomicUsize::new(current),
        }
    }
  
    

    pub fn get_server(&mut self, client_ip: &str) -> Option<&BackendServer> {
        match self.method {
            LoadBalancingMethod::RoundRobin => self.get_server_round_robin(),
            LoadBalancingMethod::Random => self.get_server_random(),
            LoadBalancingMethod::IPHash => self.get_server_ip_hash(client_ip),
            // LoadBalancingMethod::LeastConnections => self.get_server_least_connections(),
        }
    }

    fn get_server_round_robin(&self) -> Option<&BackendServer> {
        if self.servers.is_empty() {
            return None;
        }

        let index = self.current_server_index.fetch_add(1, Ordering::SeqCst) % self.servers.len();
        self.servers.get(index)
    }

    fn get_server_random(&self) -> Option<&BackendServer> {
        if self.servers.is_empty() {
            return None;
        }

        let random_index = rand::thread_rng().gen_range(0..self.servers.len());
        self.servers.get(random_index)
    }

    fn get_server_ip_hash(&self, client_ip: &str) -> Option<&BackendServer> {
        let mut hasher = DefaultHasher::new();
        client_ip.hash(&mut hasher); // Directly hash the client IP
        let hash = hasher.finish();

        if self.servers.is_empty() {
            return None; // Handle empty server list
        }

        let random_index = hash as usize % self.servers.len();
        self.servers.get(random_index)
    }

    // fn get_server_least_connections(&self) -> Option<&BackendServer> {
    //     unimplemented!()
    // }
}

pub fn get_load_balancing_method(method: &str) -> LoadBalancingMethod {
    match method {
        "round_robin" => LoadBalancingMethod::RoundRobin,
        "random" => LoadBalancingMethod::Random,
        "ip_hash" => LoadBalancingMethod::IPHash,
        // "least_connections" => LoadBalancingMethod::LeastConnections,
        _ => LoadBalancingMethod::RoundRobin,
    }
}