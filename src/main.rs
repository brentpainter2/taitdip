use serde::Deserialize;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Deserialize, Clone)]
struct LogConfig {
    directory: String,
    file_name: String,
    rotation: String,
    #[allow(dead_code)]
    retention_hours: u64,
    level: String,
}

#[derive(Deserialize, Clone)]
struct NodeSettings {
    version: u8,
    node_ip: String,
    port: u16,
    unit_address: String,
    priority: u8,
    codec: u8,
    keep_alive_interval: u64,
}

#[derive(Deserialize, Clone)]
struct Config {
    node_settings: NodeSettings,
    logging: LogConfig,
}

struct TaitClient {
    stream: TcpStream,
}

/// Helper to convert byte slices to a space-separated Hex string
fn to_hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ")
}

impl TaitClient {
    fn connect(settings: &NodeSettings) -> io::Result<Self> {
        let connection_string = format!("{}:{}", settings.node_ip, settings.port);
        info!("Connecting to {} (DIP v{})", connection_string, settings.version);

        let mut stream = TcpStream::connect(connection_string)?;
        
        // Handshake timeout
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;

        let login_cmd = format!(
            "li:{}:{}:{}:{}\n",
            settings.version, settings.unit_address, settings.priority, settings.codec
        );

        // Debug TX Handshake
        debug!("TX String: {:?}", login_cmd.trim());
        debug!("TX Hex:    [{}]", to_hex_string(login_cmd.as_bytes()));

        stream.write_all(login_cmd.as_bytes())?;

        let mut reader = BufReader::new(stream.try_clone()?);
        let mut response = String::new();

        reader.read_line(&mut response)?;
        
        // Debug RX Handshake
        debug!("RX String: {:?}", response.trim());
        debug!("RX Hex:    [{}]", to_hex_string(response.as_bytes()));

        if TaitClient::parse_login_response(response.trim()) {
            // Set session timeout (2x keep-alive) to catch zombie links
            let session_timeout = Duration::from_secs(settings.keep_alive_interval * 2);
            stream.set_read_timeout(Some(session_timeout))?;
            Ok(TaitClient { stream })
        } else {
            Err(io::Error::new(io::ErrorKind::ConnectionRefused, response.trim()))
        }
    }

    fn parse_login_response(resp: &str) -> bool {
        let parts: Vec<&str> = resp.split(':').collect();
        if parts.len() >= 3 && parts[0] == "li" {
            match parts[2] {
                "0" => {
                    info!(protocol_version = parts[1], "Login Successful");
                    true
                }
                "3" => {
                    warn!("Login Failed: Already connected (session likely ghosted)");
                    false
                }
                err => {
                    error!(code = err, "Login Failed");
                    false
                }
            }
        } else {
            error!(response = resp, "Malformed response from node");
            false
        }
    }
}

fn handle_session(client: TaitClient, settings: &NodeSettings) -> io::Result<()> {
    let mut ka_stream = client.stream.try_clone()?;
    let interval = settings.keep_alive_interval;

    // 1. Heartbeat Thread
    thread::spawn(move || {
        let ka_cmd = b"ka\n";
        let ka_hex = to_hex_string(ka_cmd);
        loop {
            thread::sleep(Duration::from_secs(interval));
            
            debug!("TX Heartbeat Hex: [{}]", ka_hex);
            info!("Sent ka heartbeat (ka)");

            if let Err(e) = ka_stream.write_all(ka_cmd) {
                error!("Keep Alive transmission failed: {}", e);
                break;
            }
        }
    });

    // 2. Main Listener
    let reader = BufReader::new(client.stream);
    for line in reader.lines() {
        match line {
            Ok(msg) => {
                let bytes = msg.as_bytes();
                let trimmed = msg.trim();

                debug!("RX Raw Hex: [{}]", to_hex_string(bytes));

                if trimmed == "ka" {
                    info!("Node acknowledged heartbeat (ka)");
                } else if !trimmed.is_empty() {
                    info!(message = trimmed, "Received DIP event");
                }
            }
            Err(e) => {
                return Err(e); // Break out to main loop for reconnect
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Config Loading
    let config_path = Path::new("config").join("default.toml");
    let config_contents = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_contents)?;

    // 2. Logging Setup
    let rotation = match config.logging.rotation.as_str() {
        "hourly" => tracing_appender::rolling::Rotation::HOURLY,
        "daily" => tracing_appender::rolling::Rotation::DAILY,
        _ => tracing_appender::rolling::Rotation::NEVER,
    };

    let file_appender = tracing_appender::rolling::RollingFileAppender::new(
        rotation,
        &config.logging.directory,
        &config.logging.file_name,
    );

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    let log_level = match config.logging.level.to_lowercase().as_str() {
        "error" => Level::ERROR,
        "warn" => Level::WARN,
        "debug" => Level::DEBUG,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(log_level.into()))
        .with(fmt::layer().with_writer(io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    info!("Starting Tait DIP client (Reliability Mode)");

    // 3. Resilience Loop
    let mut retry_count = 0;
    let max_retries = 3;

    loop {
        match TaitClient::connect(&config.node_settings) {
            Ok(client) => {
                retry_count = 0; 
                if let Err(e) = handle_session(client, &config.node_settings) {
                    error!("Session lost: {}. Reconnecting...", e);
                }
            }
            Err(e) => {
                retry_count += 1;
                error!("Connection failed ({}/{}): {}", retry_count, max_retries, e);

                if retry_count >= max_retries {
                    warn!("Critical failure. Cooling down for 60s before retry...");
                    thread::sleep(Duration::from_secs(60));
                    retry_count = 0;
                } else {
                    thread::sleep(Duration::from_secs(5));
                }
            }
        }
    }
}