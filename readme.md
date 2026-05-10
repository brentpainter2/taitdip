# Tait DIP Client

## Introduction

The **Tait DIP Client** is a Rust-based TCP application designed to connect to a Tait Digital Interface Protocol (DIP) node, authenticate using a login command, maintain a persistent session using keep-alive heartbeats, and continuously receive and log DIP events.

The application is designed with a strong focus on:

- Reliability
- Automatic reconnection
- Structured logging
- Session monitoring
- Configuration-driven deployment

It is intended for long-running operational environments where resilient communications with a Tait radio/network infrastructure are required.

---

# Features

## Core Capabilities

- TCP client connection to a Tait DIP node
- DIP login/authentication support
- Automatic heartbeat ("ka") keep-alive messages
- Continuous event reception
- Structured logging using `tracing`
- Automatic reconnection and retry handling
- Configurable logging rotation
- Runtime configurable via `default.toml`
- Hex dump debugging for TX/RX protocol inspection

---

# Architecture Overview

```text
+------------------------------------------------------+
|                  Tait DIP Client                     |
+------------------------------------------------------+
                |
                v
+------------------------------------------------------+
|                Load Configuration                    |
|                 config/default.toml                  |
+------------------------------------------------------+
                |
                v
+------------------------------------------------------+
|                  Initialise Logging                  |
|     Console + Rotating File Logging via tracing      |
+------------------------------------------------------+
                |
                v
+------------------------------------------------------+
|                Main Resilience Loop                  |
|          Connection + Retry Management               |
+------------------------------------------------------+
                |
                v
+------------------------------------------------------+
|               Connect to DIP Node                    |
|             TCP Socket Establishment                 |
+------------------------------------------------------+
                |
                v
+------------------------------------------------------+
|                  Send Login Command                  |
|       li:<version>:<address>:<priority>:<codec>     |
+------------------------------------------------------+
                |
                v
+------------------------------------------------------+
|               Parse Login Response                   |
+------------------------------------------------------+
        | Success                    | Failure
        v                            v
+-------------------+      +--------------------------+
| Start Session     |      | Retry / Backoff Logic    |
| Keep Alive Thread |      | Reconnect Attempts       |
+-------------------+      +--------------------------+
        |
        v
+------------------------------------------------------+
|              Receive Incoming Messages               |
|          Log Events + Heartbeat Responses            |
+------------------------------------------------------+
```

---

# Configuration

The application reads configuration from:

```text
config/default.toml
```

---

## Example Configuration

```toml
[node_settings]
version = 1
node_ip = "192.168.1.100"
port = 5000
unit_address = "1234"
priority = 1
codec = 0
keep_alive_interval = 30

[logging]
directory = "logs"
file_name = "tait_dip.log"
rotation = "daily"
retention_hours = 168
level = "info"
```

---

# Configuration Parameters

## Node Settings

| Parameter | Description |
|---|---|
| `version` | DIP protocol version |
| `node_ip` | IP address of DIP node |
| `port` | TCP port number |
| `unit_address` | Unit/login identifier |
| `priority` | Connection priority |
| `codec` | Audio/data codec setting |
| `keep_alive_interval` | Heartbeat interval in seconds |

---

## Logging Settings

| Parameter | Description |
|---|---|
| `directory` | Log file directory |
| `file_name` | Base log file name |
| `rotation` | Log rotation mode (`hourly`, `daily`, `never`) |
| `retention_hours` | Reserved for future retention management |
| `level` | Logging level (`error`, `warn`, `info`, `debug`, `trace`) |

---

# Connection Workflow

## 1. TCP Connection

The client connects to the configured DIP node using a standard TCP socket.

```text
<node_ip>:<port>
```

Example:

```text
192.168.1.100:5000
```

---

## 2. Login Procedure

After connecting, the application transmits a DIP login command:

```text
li:<version>:<unit_address>:<priority>:<codec>
```

Example:

```text
li:1:1234:1:0
```

---

## 3. Login Response Handling

The application validates the response from the DIP node.

### Success Response

```text
li:<version>:0
```

Where:

```text
0 = Login successful
```

### Failure Response

```text
li:<version>:3
```

Where:

```text
3 = Already connected / ghosted session
```

Other error codes are logged as connection failures.

---

# Keep Alive Mechanism

Once authenticated, a dedicated background thread periodically sends heartbeat messages:

```text
ka
```

The node is expected to acknowledge with:

```text
ka
```

This mechanism ensures:

- Session persistence
- Connection monitoring
- Detection of broken TCP sessions

---

# Session Handling

The main session loop continuously reads incoming lines from the DIP node.

## Message Types

### Heartbeat Acknowledgements

```text
ka
```

Logged as:

```text
Node acknowledged heartbeat
```

---

### DIP Events

Any non-empty message is treated as a DIP event and logged.

Example:

```text
Received DIP event
```

---

# Reliability & Reconnection Logic

The application includes a resilience loop designed for unattended operation.

---

## Retry Behaviour

### Standard Retry

- Retry delay: 5 seconds
- Maximum retries before cooldown: 3

### Cooldown Mode

After repeated failures:

```text
Cooldown duration: 60 seconds
```

This prevents aggressive reconnect storms.

---

# Logging System

The application uses:

- `tracing`
- `tracing_subscriber`
- `tracing_appender`

for structured, high-performance logging.

---

## Logging Outputs

### Console Logging

Real-time logs written to stdout.

### File Logging

Rotating log files stored in the configured log directory.

---

## Supported Log Levels

| Level | Description |
|---|---|
| `ERROR` | Critical failures |
| `WARN` | Warning conditions |
| `INFO` | General operational events |
| `DEBUG` | Detailed diagnostics |
| `TRACE` | Very verbose protocol-level detail |

---

# Hex Debugging

For troubleshooting and protocol analysis, the application logs both:

- Human-readable strings
- Raw hexadecimal byte streams

Example:

```text
TX String: "li:1:1234:1:0"
TX Hex:    [6C 69 3A 31 3A 31 32 33 34]
```

This is useful for:

- DIP protocol debugging
- Wire-level troubleshooting
- Character encoding validation

---

# Threading Model

The application uses a lightweight multi-threaded design.

## Main Thread

Responsible for:

- Connection management
- Session handling
- Message reception

## Heartbeat Thread

Responsible for:

- Periodic keep-alive transmission

---

# Application Startup

On startup, the application logs:

- Application version
- Build profile (`debug` or `release`)

Example:

```text
=== Starting Tait DIP client (Reliability Mode) ===
```

---

# Error Handling

The application handles:

- TCP connection failures
- Login failures
- Session disconnects
- Broken keep-alive streams
- Malformed DIP responses

Errors are logged with structured metadata.

---

# Dependencies

## Core Crates

| Crate | Purpose |
|---|---|
| `serde` | TOML deserialization |
| `toml` | Configuration parsing |
| `tracing` | Structured logging |
| `tracing_subscriber` | Logging subscribers |
| `tracing_appender` | Rolling log files |

---

# Intended Use Cases

This application is suitable for:

- Radio network event monitoring
- Tait DIP integrations
- Infrastructure telemetry
- Alarm/event collection
- Long-running operational services
- Headless Linux deployments

---

# Build & Run

## Debug Build

```bash
cargo run
```

---

## Release Build

```bash
cargo build --release
```

Run:

```bash
./target/release/tait-dip-client
```

---

# Future Enhancement Ideas

Potential improvements include:

- Async Tokio-based networking
- Configurable exponential backoff
- Metrics/health endpoints
- Persistent event storage
- TLS support
- Systemd watchdog integration
- Graceful shutdown handling
- Heartbeat timeout detection
- Connection state dashboard

---

# Summary

The Tait DIP Client is a resilient Rust TCP application designed for reliable communication with Tait DIP nodes. It provides automatic recovery, structured logging, heartbeat monitoring, and continuous event processing suitable for production operational environments.