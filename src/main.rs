use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::load_balancer::{LoadBalancer, get_load_balancing_method};

mod config;
mod load_balancer;
mod metrics_collector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Load the configuration file
    let config = config::load_config("proxee.toml")?;

    println!("Config: {:?}", &config.load_balancing.method);
    println!("Backend servers: {:?}", &config.backend_servers.len());

    // metrics for my reverse proxy prometheus metrics
    let metrics_collector = metrics_collector::MetricsCollector::new(config.metrics.clone());

    let metrics_server = metrics_collector.clone();
    tokio::spawn(async move {
        metrics_server.start_metrics_server().await;
    });


    let config = Arc::new(config);

    // get load balancer method
    let load_balancing_method = get_load_balancing_method(&config.load_balancing.method);
    

    let load_balancer = Arc::new(Mutex::new(LoadBalancer::new(
        load_balancing_method,
        config.backend_servers.clone(),
        0,
    )));

    let listener = TcpListener::bind(format!(
        "{}:{}",
        config.proxy.listen_address, config.proxy.listen_port
    ))
    .await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let load_balancer = Arc::clone(&load_balancer);
        let metrics_collector = metrics_collector.clone();
        tokio::spawn(async move {
            handle_client(stream, load_balancer,metrics_collector).await;
        });
    }
}

async fn handle_client(
    mut client_stream: TcpStream,
    load_balancer: Arc<Mutex<load_balancer::LoadBalancer>>,
    metrics_collector: metrics_collector::MetricsCollector,
) {
    let start = std::time::Instant::now();
    let client_ip = client_stream.peer_addr().unwrap().ip().to_string();

    let mut load_balancer_guard = load_balancer.lock().await;
    let backend_server = load_balancer_guard
        .get_server(&client_ip)
        .expect("Failed to get backend server");
    let backend_server_address = backend_server.address.clone();
    drop(load_balancer_guard);

    let mut backend_stream = TcpStream::connect(backend_server_address)
        .await
        .expect("Failed to connect to backend server");

    let (mut reader, mut writer) = client_stream.split();
    let (mut backend_reader, mut backend_writer) = backend_stream.split();

    let client_to_server = tokio::io::copy(&mut reader, &mut backend_writer);
    let server_to_client = tokio::io::copy(&mut backend_reader, &mut writer);

    match tokio::try_join!(client_to_server, server_to_client) {
        Ok(_) => {
            metrics_collector.inc_total_requests();
            metrics_collector.inc_http_status_code(200);
        }
        Err(e) => {
            println!("Error: {:?}", e);
            metrics_collector.inc_http_status_code(500);
        }
    }

    let duration = start.elapsed().as_secs_f64();
    metrics_collector.observe_request_latency(duration);
}
