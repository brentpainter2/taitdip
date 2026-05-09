//src/main.rs

use serde::Deserialize;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use std::time::Duration;
use tracing::{info, warn, error, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[derive(Deserialize, Clone)]
struct LogConfig {
    directory: String,
    file_name: String,
    rotation: String,
    #[allow(dead_code)] // Suppress warning until we implement manual cleanup
    retention_hours: u64,
    level: String,
}

#[derive(Deserialize, Clone)]
struct NodeSettings {
    node_ip: String,
    port: u16,
    unit_address: String,
    priority: u8,
    codec: u8,
    keep_alive_interval: u64,
}

#[derive(Deserialize, Clone)]
struct Config {
    node_settings: NodeSettings, // Matches [node_settings] in TOML
    logging: LogConfig,           // Matches [logging] in TOML
}

struct TaitClient {
    stream: TcpStream,
}

impl TaitClient {
    fn connect(settings: &NodeSettings) -> io::Result<Self> {
        let connection_string = format!("{}:{}", settings.node_ip, settings.port);
        let mut stream = TcpStream::connect(connection_string)?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;

        let login_cmd = format!(
            "li:3:{}:{}:{}\n",
            settings.unit_address, settings.priority, settings.codec
        );

        stream.write_all(login_cmd.as_bytes())?;

        let mut reader = BufReader::new(stream.try_clone()?);
        let mut response = String::new();
        reader.read_line(&mut response)?;

        let trimmed_resp = response.trim();
        if TaitClient::parse_login_response(trimmed_resp) {
            stream.set_read_timeout(None)?;
            Ok(TaitClient { stream })
        } else {
            Err(io::Error::new(io::ErrorKind::ConnectionRefused, trimmed_resp))
        }
    }

    fn parse_login_response(resp: &str) -> bool {
        let parts: Vec<&str> = resp.split(':').collect();
        if parts.len() >= 3 && parts[0] == "li" {
            match parts[2] {
                "0" => {
                    info!(protocol = parts[1], "Login Successful");
                    true
                }
                "3" => {
                    warn!("Login Failed: Already connected");
                    false
                }
                err => {
                    error!(code = err, "Login Failed");
                    false
                }
            }
        } else {
            error!(response = resp, "Malformed response");
            false
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load Config
    let config_path = Path::new("config").join("default.toml");
    let config_contents = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&config_contents)?;

    // 2. Setup Tracing with Rotation
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

    info!("Starting Tait DIP client");

    // 3. Connect & Login
    let client = TaitClient::connect(&config.node_settings)?;

    // 4. Keep Alive Thread
    let mut ka_stream = client.stream.try_clone()?;
    let interval = config.node_settings.keep_alive_interval;
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(interval));
            if let Err(e) = ka_stream.write_all(b"ka\n") {
                error!("Keep Alive transmission failed: {}", e);
                break;
            }
            info!("Sent ka heartbeat");
        }
    });

    // 5. Main Listener Loop
    let reader = BufReader::new(client.stream.try_clone()?);
    for line in reader.lines() {
        match line {
            Ok(msg) => {
                let trimmed = msg.trim();
                if trimmed == "ka" {
                    info!("Node acknowledged heartbeat (ka)");
                } else if !trimmed.is_empty() {
                    info!(message = trimmed, "Received DIP event");
                }
            }
            Err(e) => {
                error!("TCP Connection lost: {}", e);
                break;
            }
        }
    }

    Ok(())
}