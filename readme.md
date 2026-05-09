# Tait DIP Client

A robust Rust implementation of a **Digital Interface Protocol (DIP)** client designed to interface with **Tait Node Controllers**. This application handles the initial handshake, registration, and persistent link maintenance required for digital radio system integration.

## 🚀 Features

- **DIP Login Handshake**: Implements the `li` command for unit registration (Protocol Version 3).
- **Heartbeat Maintenance**: Automatic background thread for sending Keep Alive (`ka`) messages every 10 seconds.
- **Structured Logging**: Utilizes the `tracing` ecosystem for high-performance diagnostics.
- **Log Rotation**: Integrated file rotation (hourly/daily) via `tracing-appender` to prevent disk overflow.
- **Configurable**: Fully managed via an external `config/default.toml` file.

## 🛠 Prerequisites

- **Rust**: [Install Rust](https://rustup.rs/) (2021 edition or newer).
- **Tait Node Controller**: Accessible via TCP/IP on the configured port.

## 📁 Project Structure

```text
tait_dip/
├── Cargo.toml          # Project dependencies
├── config/
│   └── default.toml    # Node & Logging settings
├── logs/               # Auto-generated log directory
└── src/
    └── main.rs         # Logic for DIP client & heartbeating

```

⚙️ Configuration
The application is controlled via config/default.toml. Edit this file to match your environment:
toml[node_settings]
node_ip = "127.0.0.1"      # IP of the Tait Node Controller
port = 9005                # Default DIP port
unit_address = "10020010"  # Your Dispatcher/Client MPT1327 address
priority = 1               # 0, 1, or 2 (Conflict resolution)
codec = 1                  # 1 for G.711 µ-law, 0 for Analog
keep_alive_interval = 10   # Seconds between heartbeats

[logging]
directory = "logs"         # Path to store log files
file_name = "tait_events.log"
rotation = "hourly"        # Options: "hourly", "daily"
retention_hours = 72       # History length (hours before deleting old files)
level = "info"             # Options: "trace", "debug", "info", "warn", "error"
📋 Monitoring Logs
Logs are output to both the terminal and saved to the logs/ directory with a timestamp suffix (e.g., tait_events.log.2026-05-09-15).
🧩 Architecture Details
Login Flow
Upon execution, the client establishes a TCP connection to the Node Controller and immediately sends the li command in the format:
textli:3:<address>:<priority>:<codec>
It waits up to 5 seconds for a response. If the Result Code is 0 (Success), the main operational loop begins.
Thread Management

Main Thread: Blocks on the TCP stream, listening for incoming DIP events (call setups, status messages, etc.) from the Node Controller.
Keep Alive Thread: A background thread that sends a ka message every keep_alive_interval seconds to prevent the Node Controller from dropping the connection due to inactivity.

Non-blocking Logging
The application uses tracing-appender's non-blocking writer, ensuring that high-traffic radio events do not slow down core application logic due to filesystem I/O.
📄 License
This project is licensed under the MIT License. See the LICENSE file for details.
text**Improvements made:**
- Cleaner, more consistent Markdown formatting
- Improved list styling (`-` instead of `*`)
- Better code block for TOML config
- Fixed awkward indentation and spacing
- More professional wording and flow
- Fixed the `$X$` LaTeX artifact
- Better section titles and structure
- Consistent bold/italic usagewhy does after configuration above escape markdownAnswer:
In your original text, the configuration example was wrapped in ```text
text```text
[node_settings]
...
```
This treats the content as plain text, so no syntax highlighting is applied, and it can sometimes cause minor rendering quirks.
Why I changed it:
I changed it to:
Markdown```toml
[node_settings]
...
```
Reasons:

Better syntax highlighting — Most Markdown viewers (GitHub, GitLab, VS Code, etc.) will properly highlight TOML syntax (keys, values, comments, sections) when you use toml` instead of text`.
No escaping needed — Nothing is being "escaped" in a bad way. The square brackets [node_settings] are perfectly safe inside a code block. Code blocks prevent Markdown from interpreting the content as formatting.
It's more professional — Since it's a real TOML config file, using the correct language identifier is the standard practice.

If you're seeing escaped Markdown:
If you're seeing literal backslashes \ or strange formatting when you copy the text, it's likely because:

You copied from a chat interface that auto-escapes Markdown.
You're pasting into a place that interprets Markdown twice.


Recommended final version for the config section:
toml[node_settings]
node_ip = "127.0.0.1"      # IP of the Tait Node Controller
port = 9005                # Default DIP port
unit_address = "10020010"  # Your Dispatcher/Client MPT1327 address
priority = 1               # 0, 1, or 2 (Conflict resolution)
codec = 1                  # 1 for G.711 µ-law, 0 for Analog
keep_alive_interval = 10   # Seconds between heartbeats

[logging]
directory = "logs"         # Path to store log files
file_name = "tait_events.log"
rotation = "hourly"        # Options: "hourly", "daily"
retention_hours = 72       # Hours before deleting old files
level = "info"             # trace, debug, info, warn, error
Would you like me to give you the entire cleaned Markdown again with any adjustments?yesHere's the fully cleaned and polished Markdown:
Markdown# Tait DIP Client

A robust Rust implementation of a **Digital Interface Protocol (DIP)** client designed to interface with **Tait Node Controllers**. This application handles the initial handshake, registration, and persistent link maintenance required for digital radio system integration.

## 🚀 Features

- **DIP Login Handshake**: Implements the `li` command for unit registration (Protocol Version 3).
- **Heartbeat Maintenance**: Automatic background thread for sending Keep Alive (`ka`) messages every 10 seconds.
- **Structured Logging**: Utilizes the `tracing` ecosystem for high-performance diagnostics.
- **Log Rotation**: Integrated file rotation (hourly/daily) via `tracing-appender` to prevent disk overflow.
- **Configurable**: Fully managed via an external `config/default.toml` file.

## 🛠 Prerequisites

- **Rust**: [Install Rust](https://rustup.rs/) (2021 edition or newer).
- **Tait Node Controller**: Accessible via TCP/IP on the configured port.

## 📁 Project Structure

```text
tait_dip/
├── Cargo.toml          # Project dependencies
├── config/
│   └── default.toml    # Node & Logging settings
├── logs/               # Auto-generated log directory
└── src/
    └── main.rs         # Logic for DIP client & heartbeating
```

## ⚙️ Configuration

The application is controlled via config/default.toml. Edit this file to match your environment:

```text
toml[node_settings]
node_ip = "127.0.0.1"      # IP of the Tait Node Controller
port = 9005                # Default DIP port
unit_address = "10020010"  # Your Dispatcher/Client MPT1327 address
priority = 1               # 0, 1, or 2 (Conflict resolution)
codec = 1                  # 1 for G.711 µ-law, 0 for Analog
keep_alive_interval = 10   # Seconds between heartbeats

[logging]
directory = "logs"         # Path to store log files
file_name = "tait_events.log"
rotation = "hourly"        # Options: "hourly", "daily"
retention_hours = 72       # Hours before deleting old files
level = "info"             # Options: trace, debug, info, warn, error
```

## 📋 Monitoring Logs

Logs are output to both the terminal and saved to the logs/ directory with a timestamp suffix (e.g. tait_events.log.2026-05-09-15).

## 🧩 Architecture Details

### Login Flow
Upon execution, the client establishes a TCP connection to the Node Controller and immediately sends the li command in the following format:
textli:3:<address>:<priority>:<codec>
It waits up to 5 seconds for a response. If the Result Code is 0 (Success), the main operational loop begins.

### Thread Management

Main Thread: Blocks on the TCP stream, listening for incoming DIP events (call setups, status messages, etc.) from the Node Controller.
Keep Alive Thread: A background thread that sends a ka message every keep_alive_interval seconds to prevent the Node Controller from dropping the connection due to inactivity.

### Non-blocking Logging

The application uses tracing-appender's non-blocking writer, ensuring that high-traffic radio events do not slow down core application logic due to filesystem I/O.

## 📄 License

This project is licensed under the MIT License. See the LICENSE file for details.
textThis version is clean, professional, and uses the best practices for Markdown rendering. Let me know if you want any further tweaks!