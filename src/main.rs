use tokio::net::{TcpListener, TcpStream};
use std::sync::Arc;
use tokio::sync::Mutex;

mod config;
mod load_balancer;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the configuration file
    let config = config::load_config("proxee.toml")?;

    println!("Config: {:?}", &config.load_balancing.method);
    println!("Backend servers: {:?}", &config.backend_servers.len());

    let config = Arc::new(config);

    // get load balancer method
    let load_balancing_method = match config.load_balancing.method.as_str() {
        "round_robin" => load_balancer::LoadBalancingMethod::RoundRobin,
        "random" => load_balancer::LoadBalancingMethod::Random,
        "ip_hash" => load_balancer::LoadBalancingMethod::IPHash,
        _ => load_balancer::LoadBalancingMethod::RoundRobin,
    };

    let load_balancer = Arc::new(Mutex::new(load_balancer::LoadBalancer::new(
        load_balancing_method,
        config.backend_servers.clone(),
        0,
    )));

    let listener = TcpListener::bind(format!("{}:{}", config.proxy.listen_address, config.proxy.listen_port)).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let load_balancer = Arc::clone(&load_balancer);
        tokio::spawn(async move {
            handle_client(stream, load_balancer).await;
        });
    }
}

async fn handle_client(
    mut client_stream: TcpStream,
    load_balancer: Arc<Mutex<load_balancer::LoadBalancer>>,
) {
    let client_ip = client_stream.peer_addr().unwrap().ip().to_string();

    let mut load_balancer_guard = load_balancer.lock().await;
    let backend_server = load_balancer_guard.get_server(&client_ip)
        .expect("Failed to get backend server");
    let backend_server_address = backend_server.address.clone();

    let mut backend_stream = TcpStream::connect(backend_server_address).await
        .expect("Failed to connect to backend server");

    let (mut reader, mut writer) = client_stream.split();
    let (mut backend_reader, mut backend_writer) = backend_stream.split();

    let client_to_server = tokio::io::copy(&mut reader, &mut backend_writer);
    let server_to_client = tokio::io::copy(&mut backend_reader, &mut writer);

    tokio::try_join!(client_to_server, server_to_client).unwrap();
}