# Log Analyzer

A high-performance, self-contained log monitoring engine built with **Rust** and **WebAssembly**. It features real-time file tailing, a multi-threaded ETL pipeline, and a reactive dashboard powered by `uPlot`.
### Design Decisions

* **Linear-Time Parsing:** Log lines are processed in a single pass using a streaming `BufReader`, ensuring O(n) complexity.
* **WASM-Side Aggregation:** Instead of sending massive JSON blobs, the backend sends structured `LogUpdate` packets. The WASM client performs the final time-binning into the `BTreeMap`, offloading UI computation from the main JS thread.
* **Non-Blocking I/O:** The use of `mpsc` (Multi-Producer Single-Consumer) channels ensures the file-tailing thread never waits for the WebSocket or Web Server to catch up.


---
![screen-capture1-ezgif com-video-to-gif-converter](https://github.com/user-attachments/assets/7fbaed5a-5ed7-4ea0-8cf4-12e00fa69b35)

## 🏗️ Build Instructions

To create the single, self-contained binary, you must build the frontend assets **before** compiling the Rust application so they can be embedded via `rust_embed`.

### 1. Build WASM Library
```bash
cd wasm_client
wasm-pack build --target web
```

### 2. Build Web UI Assets
This generates the dist folder that the Rust binary will "bake" into its own executable.

```bash
cd site
npm install
npm run build
cd ../..
```

### 3. Build & Run the Application

```bash
cargo build
./target/debug/log_analyzer
```

## Usage
The analyzer supports two modes of operation:

### Mode A: Manual File Selection (Default)

Define your log files in `files.yaml` and run:
```bash
./target/debug/log_analyzer
```

### Mode B: Directory Monitoring

Monitor all files within a specific directory:
```bash
./target/debug/log_analyzer --dir_path /home/dhruvik/log_analyzer/example_logs
```

## Configuration
The analyzer relies on two configuration files located in the project root.

### 1. formats.json
Defines timestamp parsing and error classification.

```JSON
{
  "date_formats": [
    { "name": "Common Log", "format": "[%d/%b/%Y:%H:%M:%S %z]" },
    { "name": "JSON ISO",   "format": "%+" },
    { "name": "Syslog",     "format": "%b %d %H:%M:%S" }
  ],
  "error_indicators": ["error", "404", "500", "panic", "critical", "failed"]
}
```

### 2. files.yaml
Used for Mode A (Manual Selection).

```YAML
files:
  - "/home/dhruvik/log_analyzer/example_logs/access.log"
  - "/home/dhruvik/log_analyzer/example_logs/service.log"
```

## Service Endpoints

| Service    | Address | Description |
| -------- | ------- | ------------- |
| Web UI | http://127.0.0.1:7867/    | The dashboard served from memory |
| WebSocket | ws://127.0.0.1:9001/     | Real-time structured log stream |

## Internal Architecture
The system utilizes three primary parallel execution units:

- ETL Thread: Watches files using the notify crate. It parses strings into LogUpdate objects and flags errors based on your indicators.
- Web Server: A custom multi-threaded TcpListener that serves embedded WASM, HTML, and JS assets using RustEmbed.
- Socket Server: Manages active WebSocket connections and maintains a 500-log "Replay Cache" for new dashboard sessions.

## Features
- Zero-I/O UI: Assets are served from RAM for maximum speed.
- Monotonic Sorting: Client-side BTreeMap ensures the X-axis is always sorted correctly.
- Resource Efficient: 500ms throttled updates (via TimeoutFuture) prevent CPU spikes during log bursts.
- Logrotate Friendly: Automatically handles file renames and truncations without dropping the stream.
