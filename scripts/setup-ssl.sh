#!/bin/bash
# NetChain SSL Setup Script
# Sets up Caddy as a reverse proxy with automatic HTTPS
#
# Prerequisites:
# - DNS A record pointing api.netchain.me to this server's IP
# - Ports 80 and 443 open in security group
# - NetChain node running on default ports (8545, 8546, 9090)
#
# Usage: sudo ./setup-ssl.sh [domain]
# Example: sudo ./setup-ssl.sh api.netchain.me

set -e

# Configuration
DOMAIN="${1:-api.netchain.me}"
RPC_PORT="${RPC_PORT:-8545}"
WS_PORT="${WS_PORT:-8546}"
METRICS_PORT="${METRICS_PORT:-9090}"

echo "=============================================="
echo "NetChain SSL Setup with Caddy"
echo "=============================================="
echo "Domain: $DOMAIN"
echo "RPC Port: $RPC_PORT"
echo "WebSocket Port: $WS_PORT"
echo "Metrics Port: $METRICS_PORT"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Error: Please run as root (sudo ./setup-ssl.sh)"
    exit 1
fi

# Detect OS
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
else
    echo "Error: Cannot detect OS"
    exit 1
fi

echo "[1/5] Installing Caddy..."

case $OS in
    ubuntu|debian)
        apt-get update -qq
        apt-get install -y -qq debian-keyring debian-archive-keyring apt-transport-https curl
        curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg 2>/dev/null || true
        curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | tee /etc/apt/sources.list.d/caddy-stable.list > /dev/null
        apt-get update -qq
        apt-get install -y -qq caddy
        ;;
    amzn|rhel|centos|fedora)
        yum install -y -q yum-plugin-copr 2>/dev/null || dnf install -y -q dnf-plugins-core
        yum copr enable -y @caddy/caddy 2>/dev/null || dnf copr enable -y @caddy/caddy
        yum install -y -q caddy 2>/dev/null || dnf install -y -q caddy
        ;;
    *)
        echo "Error: Unsupported OS: $OS"
        echo "Please install Caddy manually: https://caddyserver.com/docs/install"
        exit 1
        ;;
esac

echo "[2/5] Creating Caddy configuration..."

cat > /etc/caddy/Caddyfile << EOF
# NetChain API Reverse Proxy Configuration
# Domain: $DOMAIN
# Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")

$DOMAIN {
    # Enable automatic HTTPS with Let's Encrypt
    # Caddy handles certificate provisioning and renewal automatically

    # CORS headers for browser access
    header {
        Access-Control-Allow-Origin *
        Access-Control-Allow-Methods "GET, POST, OPTIONS"
        Access-Control-Allow-Headers "Content-Type, Authorization"
        Access-Control-Max-Age 86400
    }

    # Handle preflight OPTIONS requests
    @options method OPTIONS
    respond @options 204

    # RPC endpoint - JSON-RPC API
    handle /rpc* {
        reverse_proxy localhost:$RPC_PORT
    }

    # WebSocket endpoint for real-time updates
    handle /ws* {
        reverse_proxy localhost:$WS_PORT
    }

    # Metrics endpoint for Prometheus scraping
    handle /metrics* {
        reverse_proxy localhost:$METRICS_PORT
    }

    # Health check endpoint
    handle /health {
        respond "OK" 200
    }

    # Root path - API info
    handle / {
        respond "NetChain API - https://$DOMAIN" 200
    }

    # Catch-all - return 404 for unknown paths
    handle {
        respond "Not Found" 404
    }

    # Logging
    log {
        output file /var/log/caddy/access.log {
            roll_size 100mb
            roll_keep 5
        }
    }
}
EOF

echo "[3/5] Setting up log directory..."
mkdir -p /var/log/caddy
chown caddy:caddy /var/log/caddy

echo "[4/5] Validating Caddy configuration..."
caddy validate --config /etc/caddy/Caddyfile

echo "[5/5] Starting Caddy service..."
systemctl enable caddy
systemctl restart caddy

# Wait for Caddy to start
sleep 3

# Check status
if systemctl is-active --quiet caddy; then
    echo ""
    echo "=============================================="
    echo "SSL Setup Complete!"
    echo "=============================================="
    echo ""
    echo "Endpoints:"
    echo "  RPC:       https://$DOMAIN/rpc"
    echo "  WebSocket: wss://$DOMAIN/ws"
    echo "  Metrics:   https://$DOMAIN/metrics"
    echo "  Health:    https://$DOMAIN/health"
    echo ""
    echo "Caddy will automatically obtain and renew"
    echo "SSL certificates from Let's Encrypt."
    echo ""
    echo "View logs: journalctl -u caddy -f"
    echo "Config:    /etc/caddy/Caddyfile"
    echo ""
else
    echo ""
    echo "Error: Caddy failed to start"
    echo "Check logs: journalctl -u caddy -n 50"
    exit 1
fi
