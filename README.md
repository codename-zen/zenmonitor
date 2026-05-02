<p align="center">
  <img src="docs/logo.png" alt="ZenMonitor" width="200"/>
  <br/>
  <strong>ZenMonitor</strong>
  <br/>
  <em>Infrastructure monitoring that doesn't suck.</em>
  <br/><br/>
  <a href="https://zenlabs.ai"><img src="https://img.shields.io/badge/by-ZenLabsAI-blue?style=flat-square" alt="ZenLabsAI"/></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="MIT License"/></a>
  <a href="https://github.com/zenlabsai/zenmonitor/releases"><img src="https://img.shields.io/badge/version-0.1.0-orange?style=flat-square" alt="Version"/></a>
</p>

---

## What is ZenMonitor?

**ZenMonitor** is a lightweight, high-performance infrastructure monitoring service built by [ZenLabsAI](https://zenlabs.ai). It monitors your servers, websites, and services with minimal resource overhead — written in Rust for speed and reliability.

Unlike bloated monitoring stacks, ZenMonitor is a single binary server + lightweight agents that give you full visibility into your infrastructure in under 5 minutes.

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🌐 **HTTP/HTTPS Monitoring** | Monitor endpoints with response time tracking, status code validation, and SSL certificate expiry alerts |
| 🏓 **ICMP Ping Monitoring** | Latency and packet loss tracking for any host |
| 🔌 **TCP Port Monitoring** | Verify services are listening on expected ports |
| 🖥️ **VPS Metrics via Agent** | CPU, RAM, disk, network, and top processes — reported by a lightweight agent binary |
| 🧠 **Proxmox RAM Handling** | Correctly distinguishes actual used memory vs. cache/buffers (reads `/proc/meminfo` directly) |
| 🔔 **Alerts** | Telegram and Discord notifications when monitors go down or recover |
| 🌙 **Dark Theme Dashboard** | Beautiful real-time dashboard with SSE live updates |
| 🔒 **SSL Certificate Tracking** | Know exactly when your certs expire — days in advance |
| 📡 **Real-time SSE** | Server-Sent Events push updates to the dashboard instantly |
| 🐳 **Docker Ready** | Single `docker-compose up` to deploy the server |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          ZenMonitor Architecture                         │
└─────────────────────────────────────────────────────────────────────────┘

    ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
    │   VPS #1     │     │   VPS #2     │     │   VPS #3     │
    │              │     │              │     │              │
    │ ┌──────────┐ │     │ ┌──────────┐ │     │ ┌──────────┐ │
    │ │  Agent   │ │     │ │  Agent   │ │     │ │  Agent   │ │
    │ └────┬─────┘ │     │ └────┬─────┘ │     │ └────┬─────┘ │
    └──────┼───────┘     └──────┼───────┘     └──────┼───────┘
           │                    │                    │
           │  POST /api/agents/report (every 30s)   │
           └────────────────────┼────────────────────┘
                                │
                                ▼
                 ┌──────────────────────────────┐
                 │      ZenMonitor Server       │
                 │                              │
                 │  ┌────────────────────────┐  │
                 │  │   Monitoring Engine     │  │
                 │  │  • HTTP/HTTPS checks   │  │
                 │  │  • ICMP Ping           │  │
                 │  │  • TCP Port checks     │  │
                 │  │  • SSL cert expiry     │  │
                 │  └────────────────────────┘  │
                 │                              │
                 │  ┌────────────────────────┐  │
                 │  │   SQLite Database      │  │
                 │  └────────────────────────┘  │
                 │                              │
                 │  ┌────────────────────────┐  │
                 │  │   Alert Engine         │  │
                 │  │  • Telegram            │  │
                 │  │  • Discord             │  │
                 │  └────────────────────────┘  │
                 │                              │
                 │  ┌────────────────────────┐  │
                 │  │   Web Dashboard (SSE)  │──┼──── Real-time updates
                 │  └────────────────────────┘  │
                 └──────────────────────────────┘
                                │
                                ▼
                 ┌──────────────────────────────┐
                 │     Monitored Targets        │
                 │  • https://example.com       │
                 │  • 192.168.1.1 (ping)        │
                 │  • db-server:5432 (tcp)      │
                 └──────────────────────────────┘
```

---

## 🚀 Quick Start

### 1. Claim Your License

Visit the [ZenLabsAI Portal](https://portal.zenlabs.ai) to claim your license key.

### 2. Deploy the Server

**Option A: Docker (Recommended)**

```bash
# Clone the repository
git clone https://github.com/zenlabsai/zenmonitor.git
cd zenmonitor

# Configure
cp zenmonitor-server.toml.example zenmonitor-server.toml
# Edit zenmonitor-server.toml with your settings

# Start
docker-compose up -d
```

**Option B: Binary**

```bash
# Download the server binary for your platform
curl -fsSL https://releases.zenlabs.ai/zenmonitor/latest/zenmonitor-server-linux-amd64 -o zenmonitor-server
chmod +x zenmonitor-server

# Create config
cat > zenmonitor-server.toml <<EOF
listen_addr = "0.0.0.0:3000"
database_path = "/var/lib/zenmonitor/zenmonitor.db"
agent_api_key = "your-secure-api-key-here"
http_check_interval = 60
ping_check_interval = 30
tcp_check_interval = 60
ssl_check_interval = 3600
http_timeout = 10
EOF

# Run
./zenmonitor-server
```

### 3. Install Agents on Your Servers

```bash
# One-liner install (as root for system-wide):
curl -fsSL https://releases.zenlabs.ai/zenmonitor/install.sh | sudo bash

# Or with environment variables (non-interactive):
curl -fsSL https://releases.zenlabs.ai/zenmonitor/install.sh | \
  ZENMONITOR_SERVER_URL="https://monitor.example.com:3000" \
  ZENMONITOR_LICENSE_KEY="your-license-key" \
  sudo bash
```

The installer will:
- Detect your OS and architecture
- Download the correct agent binary
- Create configuration at `/etc/zenmonitor/agent.yaml`
- Set up a systemd service (Linux) or launchd (macOS)
- Start the agent and register with the server

---

## 📡 API Documentation

All API endpoints are prefixed with `/api`. Responses follow a consistent format:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

### Authentication

Agent endpoints require the `X-API-Key` header matching the server's `agent_api_key` configuration.

### Endpoints

#### Monitors

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/monitors` | List all monitors |
| `POST` | `/api/monitors` | Create a new monitor |
| `DELETE` | `/api/monitors/:id` | Delete a monitor |
| `GET` | `/api/monitors/:id/results` | Get check results for a monitor |

#### Agents

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/agents` | List all registered agents |
| `POST` | `/api/agents/report` | Submit agent metrics report |

#### Real-time Events

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/events` | SSE stream for real-time updates |

---

### Create Monitor

```bash
curl -X POST http://localhost:3000/api/monitors \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Website",
    "monitor_type": "https",
    "target": "https://example.com",
    "port": null,
    "interval_seconds": 60
  }'
```

**Monitor Types:** `http`, `https`, `ping`, `tcp`, `ssl`

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "My Website",
    "monitor_type": "Https",
    "target": "https://example.com",
    "port": null,
    "interval_seconds": 60,
    "enabled": true
  },
  "error": null
}
```

---

### Get Monitor Results

```bash
curl "http://localhost:3000/api/monitors/550e8400-e29b-41d4-a716-446655440000/results?limit=10"
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "monitor_id": "550e8400-e29b-41d4-a716-446655440000",
      "status": "Up",
      "response_time_ms": 142.5,
      "status_code": 200,
      "message": null,
      "checked_at": "2024-01-15T10:30:00Z"
    }
  ],
  "error": null
}
```

---

### Agent Report Payload

```bash
curl -X POST http://localhost:3000/api/agents/report \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "agent": {
      "id": "agent-uuid",
      "hostname": "web-server-01",
      "os": "Linux 6.1.0",
      "kernel": "6.1.0-debian",
      "ip_address": "192.168.1.10"
    },
    "metrics": {
      "agent_id": "agent-uuid",
      "cpu_usage": 23.5,
      "ram_total_mb": 16384.0,
      "ram_used_mb": 4096.0,
      "ram_cached_mb": 8192.0,
      "ram_available_mb": 12288.0,
      "uptime_seconds": 864000,
      "disks": [
        {
          "mount_point": "/",
          "total_gb": 100.0,
          "used_gb": 45.0,
          "available_gb": 55.0,
          "usage_percent": 45.0
        }
      ],
      "network": [
        {
          "interface": "eth0",
          "rx_bytes": 1073741824,
          "tx_bytes": 536870912,
          "rx_rate_bps": 1048576.0,
          "tx_rate_bps": 524288.0
        }
      ],
      "processes": [
        {
          "pid": 1234,
          "name": "nginx",
          "cpu_percent": 5.2,
          "memory_mb": 128.0
        }
      ]
    }
  }'
```

---

### SSE Events Stream

```bash
curl -N http://localhost:3000/api/events
```

Events are sent as JSON with the following types:
- `MonitorUpdate` — monitor created/deleted
- `AgentMetrics` — new agent metrics received
- `CheckResult` — new check result available

---

## ⚙️ Configuration Reference

### Server Configuration (`zenmonitor-server.toml`)

```toml
# Network binding address
listen_addr = "0.0.0.0:3000"

# SQLite database file path
database_path = "zenmonitor.db"

# Check intervals (seconds)
http_check_interval = 60      # HTTP/HTTPS endpoint checks
ping_check_interval = 30      # ICMP ping checks
tcp_check_interval = 60       # TCP port checks
ssl_check_interval = 3600     # SSL certificate expiry checks

# HTTP request timeout (seconds)
http_timeout = 10

# API key that agents must present
agent_api_key = "your-secure-key-here"
```

**Environment variable override:** Set `ZENMONITOR_CONFIG` to specify a custom config path.

---

### Agent Configuration (`zenmonitor-agent.toml`)

```toml
# Unique agent identifier (auto-generated on install)
agent_id = "550e8400-e29b-41d4-a716-446655440000"

# ZenMonitor server URL
server_url = "https://monitor.example.com:3000"

# API key (must match server's agent_api_key)
api_key = "your-secure-key-here"

# How often to report metrics (seconds)
report_interval = 30

# Number of top processes to include in reports
top_processes = 10
```

**Environment variable override:** Set `ZENMONITOR_AGENT_CONFIG` to specify a custom config path.

**Default config locations:**
- Root install: `/etc/zenmonitor/zenmonitor-agent.toml`
- User install: `~/.zenmonitor/zenmonitor-agent.toml`

---

## 🧠 Proxmox / Linux Memory Handling

ZenMonitor's agent reads `/proc/meminfo` directly to correctly report memory usage on Linux systems (especially Proxmox VMs and containers):

| Metric | Meaning |
|--------|---------|
| `ram_total_mb` | Total physical RAM |
| `ram_used_mb` | **Actual** used memory (`Total - Available`) |
| `ram_cached_mb` | Page cache + SReclaimable (can be freed) |
| `ram_available_mb` | Memory available for new allocations (kernel estimate) |

This avoids the common mistake of reporting `Total - Free` as "used" — which incorrectly counts cache/buffers as consumed memory.

---

## 🐳 Docker Deployment

```bash
cd zenmonitor
docker-compose up -d
```

See [`docker-compose.yml`](docker-compose.yml) for the full configuration.

The server will be available at `http://localhost:3000`.

---

## 📁 Project Structure

```
zenmonitor/
├── Cargo.toml              # Workspace root
├── server/                 # ZenMonitor Server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Entry point
│       ├── config.rs       # Server configuration
│       ├── db.rs           # SQLite database layer
│       ├── models.rs       # Data models
│       ├── sse.rs          # Server-Sent Events
│       ├── handlers/       # HTTP API handlers
│       │   ├── mod.rs      # Router setup
│       │   ├── api.rs      # REST API endpoints
│       │   └── dashboard.rs
│       └── monitors/       # Background monitoring tasks
│           ├── mod.rs
│           ├── http_monitor.rs
│           ├── ping_monitor.rs
│           ├── tcp_monitor.rs
│           └── ssl_monitor.rs
├── agent/                  # ZenMonitor Agent
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Entry point
│       ├── config.rs       # Agent configuration
│       └── collectors/     # Metric collectors
│           ├── mod.rs
│           ├── cpu.rs
│           ├── memory.rs
│           ├── disk.rs
│           ├── network.rs
│           └── processes.rs
├── install.sh              # Agent installer script
├── docker-compose.yml      # Docker deployment
├── zenmonitor-server.toml  # Server config example
└── README.md               # This file
```

---

## 🔧 Building from Source

```bash
# Prerequisites: Rust 1.75+
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/zenlabsai/zenmonitor.git
cd zenmonitor

# Build both server and agent (release mode)
cargo build --release

# Binaries will be at:
#   target/release/zenmonitor-server
#   target/release/zenmonitor-agent
```

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

<p align="center">
  Built with ❤️ and 🦀 by <a href="https://zenlabs.ai">ZenLabsAI</a>
</p>
