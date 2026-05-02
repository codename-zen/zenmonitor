#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# ZenMonitor Agent Installer
# Copyright (c) 2024 ZenLabsAI - https://zenlabs.ai
#
# This script downloads and installs the ZenMonitor agent on your system.
# Supports: Linux (amd64/arm64), macOS (amd64/arm64), Windows (via WSL)
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

# ─── Configuration ────────────────────────────────────────────────────────────

ZENMONITOR_VERSION="${ZENMONITOR_VERSION:-latest}"
DOWNLOAD_BASE_URL="${ZENMONITOR_DOWNLOAD_URL:-https://releases.zenlabs.ai/zenmonitor}"
AGENT_BINARY_NAME="zenmonitor-agent"
CONFIG_DIR=""
INSTALL_DIR=""
IS_ROOT=false

# ─── Colors ───────────────────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# ─── Helper Functions ─────────────────────────────────────────────────────────

info()    { echo -e "${BLUE}[INFO]${NC} $*"; }
success() { echo -e "${GREEN}[OK]${NC} $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $*"; }
error()   { echo -e "${RED}[ERROR]${NC} $*"; exit 1; }

banner() {
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                                                              ║"
    echo "║     ███████╗███████╗███╗   ██╗                              ║"
    echo "║     ╚══███╔╝██╔════╝████╗  ██║                              ║"
    echo "║       ███╔╝ █████╗  ██╔██╗ ██║  MONITOR                     ║"
    echo "║      ███╔╝  ██╔══╝  ██║╚██╗██║  Agent Installer             ║"
    echo "║     ███████╗███████╗██║ ╚████║  v${ZENMONITOR_VERSION}       ║"
    echo "║     ╚══════╝╚══════╝╚═╝  ╚═══╝                              ║"
    echo "║                                                              ║"
    echo "║     by ZenLabsAI                                             ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# ─── Detect OS and Architecture ──────────────────────────────────────────────

detect_platform() {
    local os=""
    local arch=""

    # Detect OS
    case "$(uname -s)" in
        Linux*)   os="linux" ;;
        Darwin*)  os="darwin" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
        *)        error "Unsupported operating system: $(uname -s)" ;;
    esac

    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)   arch="amd64" ;;
        aarch64|arm64)  arch="arm64" ;;
        armv7l)         arch="armv7" ;;
        *)              error "Unsupported architecture: $(uname -m)" ;;
    esac

    PLATFORM="${os}"
    ARCH="${arch}"
    PLATFORM_STRING="${os}/${arch}"

    info "Detected platform: ${BOLD}${PLATFORM_STRING}${NC}"
}

# ─── Check Prerequisites ─────────────────────────────────────────────────────

check_prerequisites() {
    # Check for curl or wget
    if command -v curl &>/dev/null; then
        DOWNLOADER="curl"
    elif command -v wget &>/dev/null; then
        DOWNLOADER="wget"
    else
        error "Neither 'curl' nor 'wget' found. Please install one of them."
    fi

    # Check if running as root
    if [[ $EUID -eq 0 ]]; then
        IS_ROOT=true
        INSTALL_DIR="/usr/local/bin"
        CONFIG_DIR="/etc/zenmonitor"
    else
        IS_ROOT=false
        INSTALL_DIR="${HOME}/.local/bin"
        CONFIG_DIR="${HOME}/.zenmonitor"
        warn "Not running as root. Installing to user directories."
        warn "  Binary: ${INSTALL_DIR}/"
        warn "  Config: ${CONFIG_DIR}/"
    fi
}

# ─── Download Binary ─────────────────────────────────────────────────────────

download_binary() {
    local url=""
    local extension=""

    if [[ "${PLATFORM}" == "windows" ]]; then
        extension=".exe"
    fi

    url="${DOWNLOAD_BASE_URL}/${ZENMONITOR_VERSION}/${AGENT_BINARY_NAME}-${PLATFORM}-${ARCH}${extension}"

    info "Downloading ZenMonitor agent from:"
    info "  ${url}"

    # Create install directory
    mkdir -p "${INSTALL_DIR}"

    local tmp_file
    tmp_file=$(mktemp)

    if [[ "${DOWNLOADER}" == "curl" ]]; then
        if ! curl -fsSL "${url}" -o "${tmp_file}"; then
            rm -f "${tmp_file}"
            error "Failed to download agent binary. Check your network and the download URL."
        fi
    else
        if ! wget -q "${url}" -O "${tmp_file}"; then
            rm -f "${tmp_file}"
            error "Failed to download agent binary. Check your network and the download URL."
        fi
    fi

    # Move to install directory
    mv "${tmp_file}" "${INSTALL_DIR}/${AGENT_BINARY_NAME}${extension}"
    chmod +x "${INSTALL_DIR}/${AGENT_BINARY_NAME}${extension}"

    success "Agent binary installed to: ${INSTALL_DIR}/${AGENT_BINARY_NAME}${extension}"
}

# ─── Collect Configuration ────────────────────────────────────────────────────

collect_config() {
    local server_url=""
    local license_key=""

    echo ""
    echo -e "${BOLD}── Configuration ──${NC}"
    echo ""

    # Server URL
    if [[ -n "${ZENMONITOR_SERVER_URL:-}" ]]; then
        server_url="${ZENMONITOR_SERVER_URL}"
        info "Using server URL from environment: ${server_url}"
    else
        read -rp "$(echo -e "${CYAN}?${NC}") ZenMonitor Server URL (e.g., https://monitor.example.com:3000): " server_url
        if [[ -z "${server_url}" ]]; then
            error "Server URL is required."
        fi
    fi

    # License / API Key
    if [[ -n "${ZENMONITOR_LICENSE_KEY:-}" ]]; then
        license_key="${ZENMONITOR_LICENSE_KEY}"
        info "Using license key from environment."
    else
        read -rp "$(echo -e "${CYAN}?${NC}") License Key (API Key): " license_key
        if [[ -z "${license_key}" ]]; then
            error "License key is required."
        fi
    fi

    SERVER_URL="${server_url}"
    LICENSE_KEY="${license_key}"
}

# ─── Create Configuration File ───────────────────────────────────────────────

create_config() {
    mkdir -p "${CONFIG_DIR}"

    local agent_id
    agent_id=$(cat /proc/sys/kernel/random/uuid 2>/dev/null || uuidgen 2>/dev/null || python3 -c "import uuid; print(uuid.uuid4())" 2>/dev/null || echo "agent-$(date +%s)-$$")

    local config_file="${CONFIG_DIR}/agent.yaml"

    cat > "${config_file}" <<EOF
# ZenMonitor Agent Configuration
# Generated by install.sh on $(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Unique agent identifier (auto-generated)
agent_id: "${agent_id}"

# ZenMonitor server URL
server_url: "${SERVER_URL}"

# License / API key for authentication
api_key: "${LICENSE_KEY}"

# Reporting interval in seconds
report_interval: 30

# Number of top processes to report (by CPU usage)
top_processes: 10

# Log level: trace, debug, info, warn, error
log_level: "info"
EOF

    chmod 600 "${config_file}"
    success "Configuration written to: ${config_file}"

    # Also create a TOML config for the agent binary (it reads TOML)
    local toml_config="${CONFIG_DIR}/zenmonitor-agent.toml"
    cat > "${toml_config}" <<EOF
# ZenMonitor Agent Configuration (TOML format)
# Generated by install.sh on $(date -u +"%Y-%m-%dT%H:%M:%SZ")

agent_id = "${agent_id}"
server_url = "${SERVER_URL}"
api_key = "${LICENSE_KEY}"
report_interval = 30
top_processes = 10
EOF

    chmod 600 "${toml_config}"

    AGENT_ID="${agent_id}"
    CONFIG_FILE="${config_file}"
}

# ─── Setup systemd Service (Linux) ───────────────────────────────────────────

setup_systemd() {
    if [[ "${IS_ROOT}" != true ]]; then
        # User-level systemd service
        local service_dir="${HOME}/.config/systemd/user"
        mkdir -p "${service_dir}"

        cat > "${service_dir}/zenmonitor-agent.service" <<EOF
[Unit]
Description=ZenMonitor Agent
Documentation=https://docs.zenlabs.ai/zenmonitor
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=${INSTALL_DIR}/${AGENT_BINARY_NAME}
Environment=ZENMONITOR_AGENT_CONFIG=${CONFIG_DIR}/zenmonitor-agent.toml
Environment=RUST_LOG=zenmonitor_agent=info
Restart=always
RestartSec=10

[Install]
WantedBy=default.target
EOF

        systemctl --user daemon-reload
        systemctl --user enable zenmonitor-agent.service
        systemctl --user start zenmonitor-agent.service

        success "User systemd service created and started."
        info "  Status: systemctl --user status zenmonitor-agent"
        info "  Logs:   journalctl --user -u zenmonitor-agent -f"
    else
        # System-level systemd service
        cat > /etc/systemd/system/zenmonitor-agent.service <<EOF
[Unit]
Description=ZenMonitor Agent
Documentation=https://docs.zenlabs.ai/zenmonitor
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=${INSTALL_DIR}/${AGENT_BINARY_NAME}
Environment=ZENMONITOR_AGENT_CONFIG=${CONFIG_DIR}/zenmonitor-agent.toml
Environment=RUST_LOG=zenmonitor_agent=info
Restart=always
RestartSec=10
User=zenmonitor
Group=zenmonitor

# Security hardening
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadOnlyPaths=/
ReadWritePaths=/var/log
PrivateTmp=yes

[Install]
WantedBy=multi-user.target
EOF

        # Create service user if it doesn't exist
        if ! id -u zenmonitor &>/dev/null; then
            useradd --system --no-create-home --shell /usr/sbin/nologin zenmonitor
            info "Created system user: zenmonitor"
        fi

        # Ensure config is readable by service user
        chown -R zenmonitor:zenmonitor "${CONFIG_DIR}"

        systemctl daemon-reload
        systemctl enable zenmonitor-agent.service
        systemctl start zenmonitor-agent.service

        success "Systemd service created and started."
        info "  Status: systemctl status zenmonitor-agent"
        info "  Logs:   journalctl -u zenmonitor-agent -f"
    fi
}

# ─── Setup launchd Service (macOS) ────────────────────────────────────────────

setup_launchd() {
    local plist_dir=""
    local plist_label="ai.zenlabs.zenmonitor-agent"

    if [[ "${IS_ROOT}" == true ]]; then
        plist_dir="/Library/LaunchDaemons"
    else
        plist_dir="${HOME}/Library/LaunchAgents"
    fi

    mkdir -p "${plist_dir}"

    local plist_file="${plist_dir}/${plist_label}.plist"

    cat > "${plist_file}" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${plist_label}</string>

    <key>ProgramArguments</key>
    <array>
        <string>${INSTALL_DIR}/${AGENT_BINARY_NAME}</string>
    </array>

    <key>EnvironmentVariables</key>
    <dict>
        <key>ZENMONITOR_AGENT_CONFIG</key>
        <string>${CONFIG_DIR}/zenmonitor-agent.toml</string>
        <key>RUST_LOG</key>
        <string>zenmonitor_agent=info</string>
    </dict>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>${CONFIG_DIR}/agent.log</string>

    <key>StandardErrorPath</key>
    <string>${CONFIG_DIR}/agent.err.log</string>

    <key>ThrottleInterval</key>
    <integer>10</integer>
</dict>
</plist>
EOF

    if [[ "${IS_ROOT}" == true ]]; then
        launchctl load -w "${plist_file}"
    else
        launchctl load -w "${plist_file}"
    fi

    success "launchd service created and loaded."
    info "  Plist: ${plist_file}"
    info "  Logs:  ${CONFIG_DIR}/agent.log"
    info "  Stop:  launchctl unload ${plist_file}"
}

# ─── Setup Service ────────────────────────────────────────────────────────────

setup_service() {
    echo ""
    echo -e "${BOLD}── Service Setup ──${NC}"
    echo ""

    case "${PLATFORM}" in
        linux)
            if command -v systemctl &>/dev/null; then
                setup_systemd
            else
                warn "systemd not found. You'll need to start the agent manually:"
                warn "  ZENMONITOR_AGENT_CONFIG=${CONFIG_DIR}/zenmonitor-agent.toml ${INSTALL_DIR}/${AGENT_BINARY_NAME}"
            fi
            ;;
        darwin)
            setup_launchd
            ;;
        windows)
            warn "Windows detected. Please run the agent manually or set up a Windows Service:"
            warn "  set ZENMONITOR_AGENT_CONFIG=${CONFIG_DIR}\\zenmonitor-agent.toml"
            warn "  ${INSTALL_DIR}\\${AGENT_BINARY_NAME}.exe"
            ;;
    esac
}

# ─── Report Platform to Server ────────────────────────────────────────────────

report_platform() {
    echo ""
    info "Reporting agent registration to server..."

    local hostname
    hostname=$(hostname 2>/dev/null || echo "unknown")

    local os_info
    os_info=$(uname -sr 2>/dev/null || echo "unknown")

    local payload
    payload=$(cat <<EOF
{
    "agent": {
        "id": "${AGENT_ID}",
        "hostname": "${hostname}",
        "os": "${os_info}",
        "kernel": "$(uname -r 2>/dev/null || echo 'unknown')",
        "ip_address": null
    },
    "event": "agent_installed",
    "platform": "${PLATFORM_STRING}",
    "version": "${ZENMONITOR_VERSION}",
    "installed_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF
)

    local register_url="${SERVER_URL}/api/agents/report"

    if [[ "${DOWNLOADER}" == "curl" ]]; then
        local http_code
        http_code=$(curl -s -o /dev/null -w "%{http_code}" \
            -X POST \
            -H "Content-Type: application/json" \
            -H "X-API-Key: ${LICENSE_KEY}" \
            -d "${payload}" \
            "${register_url}" 2>/dev/null || echo "000")

        if [[ "${http_code}" == "200" || "${http_code}" == "201" ]]; then
            success "Agent registered with server successfully."
        else
            warn "Could not reach server (HTTP ${http_code}). The agent will retry on next report cycle."
        fi
    else
        wget -q --post-data="${payload}" \
            --header="Content-Type: application/json" \
            --header="X-API-Key: ${LICENSE_KEY}" \
            -O /dev/null \
            "${register_url}" 2>/dev/null && \
            success "Agent registered with server successfully." || \
            warn "Could not reach server. The agent will retry on next report cycle."
    fi
}

# ─── Summary ──────────────────────────────────────────────────────────────────

print_summary() {
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║          ✅ ZenMonitor Agent Installation Complete           ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "  ${BOLD}Agent ID:${NC}    ${AGENT_ID}"
    echo -e "  ${BOLD}Server:${NC}      ${SERVER_URL}"
    echo -e "  ${BOLD}Binary:${NC}      ${INSTALL_DIR}/${AGENT_BINARY_NAME}"
    echo -e "  ${BOLD}Config:${NC}      ${CONFIG_DIR}/agent.yaml"
    echo -e "  ${BOLD}Platform:${NC}    ${PLATFORM_STRING}"
    echo ""
    echo -e "  ${BOLD}Useful commands:${NC}"
    if [[ "${PLATFORM}" == "linux" ]]; then
        if [[ "${IS_ROOT}" == true ]]; then
            echo "    systemctl status zenmonitor-agent"
            echo "    journalctl -u zenmonitor-agent -f"
            echo "    systemctl restart zenmonitor-agent"
        else
            echo "    systemctl --user status zenmonitor-agent"
            echo "    journalctl --user -u zenmonitor-agent -f"
            echo "    systemctl --user restart zenmonitor-agent"
        fi
    elif [[ "${PLATFORM}" == "darwin" ]]; then
        echo "    tail -f ${CONFIG_DIR}/agent.log"
        echo "    launchctl list | grep zenmonitor"
    fi
    echo ""
    echo -e "  ${BOLD}Uninstall:${NC}"
    echo "    curl -fsSL ${DOWNLOAD_BASE_URL}/uninstall.sh | bash"
    echo ""
}

# ─── Main ─────────────────────────────────────────────────────────────────────

main() {
    banner
    detect_platform
    check_prerequisites
    download_binary
    collect_config
    create_config
    setup_service
    report_platform
    print_summary
}

# Run main unless sourced
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
