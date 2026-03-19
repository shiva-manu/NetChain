#!/bin/bash
set -e

echo "=================================="
echo "NetChain Node Deployment Script"
echo "=================================="
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="/opt/netchain"
SERVICE_NAME="netchain"
USER="ubuntu"

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}Please run as root or with sudo${NC}"
    exit 1
fi

echo -e "${GREEN}Step 1: Installing system dependencies...${NC}"
apt-get update
apt-get install -y build-essential pkg-config libssl-dev git curl

echo -e "${GREEN}Step 2: Installing Rust...${NC}"
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> /home/$USER/.bashrc
else
    echo "Rust is already installed"
fi

# Make cargo available for ubuntu user
if [ -f "/root/.cargo/env" ]; then
    source /root/.cargo/env
fi

echo -e "${GREEN}Step 3: Creating installation directory...${NC}"
mkdir -p $INSTALL_DIR
cd $INSTALL_DIR

echo -e "${GREEN}Step 4: Cloning/Copying NetChain repository...${NC}"
# If you're running this from the repo, copy files
if [ -f "../Cargo.toml" ]; then
    cp -r ../* .
else
    echo "Please ensure NetChain source code is in $INSTALL_DIR"
    exit 1
fi

echo -e "${GREEN}Step 5: Building NetChain (Release mode)...${NC}"
echo "This may take 10-15 minutes..."
cargo build --release

echo -e "${GREEN}Step 6: Creating data directory...${NC}"
mkdir -p $INSTALL_DIR/data
chown -R $USER:$USER $INSTALL_DIR

echo -e "${GREEN}Step 7: Creating production config...${NC}"
cat > $INSTALL_DIR/config/production.toml << 'EOF'
[node]
data_dir = "/opt/netchain/data"
p2p_port = 30333
log_level = "info"

[rpc]
bind_addr = "0.0.0.0"
port = 8545

[monitoring]
enabled = true
bind_addr = "0.0.0.0"
port = 9090

[websocket]
enabled = true
bind_addr = "0.0.0.0"
port = 8546

[producer]
max_txs_per_block = 100
block_interval_secs = 15
block_reward = 50
metric_measurement_interval_secs = 120
stake_weight = 0.3
mempool_ttl_secs = 900

[measurement]
download_endpoints = ["https://speed.cloudflare.com/__down?bytes=10000000"]
upload_endpoints = ["https://speed.cloudflare.com/__up"]
timeout_secs = 30
download_bytes = 10000000
upload_bytes = 5000000
history_size = 10
min_interval_secs = 60

[aggregator]
min_attestations = 3
attestation_max_age_secs = 3600
reputation_history_epochs = 10
self_report_weight = 0.2
blocks_per_epoch = 100
attestation_decay = 0.95

[anti_gaming]
outlier_threshold_sigma = 3.0
min_trusted_attestations = 3
max_challenges_per_hour = 60
max_received_challenges_per_hour = 120
history_size = 100
suspicious_penalty = 0.5

[anti_gaming.bounds]
max_download_mbps = 10000.0
max_upload_mbps = 10000.0
min_latency_ms = 0.1
max_latency_ms = 10000.0

[slashing]
invalid_block_penalty_bps = 1000
metric_fraud_penalty_bps = 500
missed_block_penalty_bps = 100
EOF

echo -e "${GREEN}Step 8: Setting up systemd service...${NC}"
cp $INSTALL_DIR/deploy/netchain.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable $SERVICE_NAME

echo -e "${GREEN}Step 9: Configuring firewall...${NC}"
if command -v ufw &> /dev/null; then
    ufw allow 30333/tcp comment 'NetChain P2P'
    ufw allow 8545/tcp comment 'NetChain RPC'
    ufw allow 8546/tcp comment 'NetChain WebSocket'
    ufw allow 9090/tcp comment 'NetChain Monitoring'
    echo "Firewall rules added"
else
    echo -e "${YELLOW}UFW not found. Please configure firewall manually:${NC}"
    echo "  - P2P Port: 30333"
    echo "  - RPC Port: 8545"
    echo "  - WebSocket Port: 8546"
    echo "  - Monitoring Port: 9090"
fi

echo ""
echo -e "${GREEN}=================================="
echo "Installation Complete!"
echo -e "==================================${NC}"
echo ""
echo "Next steps:"
echo "1. Start the node:    sudo systemctl start netchain"
echo "2. Check status:      sudo systemctl status netchain"
echo "3. View logs:         sudo journalctl -u netchain -f"
echo "4. Stop the node:     sudo systemctl stop netchain"
echo ""
echo "Node endpoints:"
echo "  - RPC:        http://$(curl -s ifconfig.me):8545"
echo "  - WebSocket:  ws://$(curl -s ifconfig.me):8546"
echo "  - Monitoring: http://$(curl -s ifconfig.me):9090"
echo "  - P2P:        port 30333"
echo ""
echo -e "${YELLOW}Important: Ensure AWS Security Group allows inbound traffic on these ports${NC}"
