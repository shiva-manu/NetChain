#!/bin/bash
# NetChain EC2 Setup Script
# Run this script on a fresh Ubuntu 22.04/24.04 EC2 instance

set -e  # Exit on error

echo "======================================"
echo "NetChain Node - EC2 Setup Script"
echo "======================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
    print_error "Please do not run this script as root"
    exit 1
fi

# Step 1: Update system
print_info "Step 1/7: Updating system packages..."
sudo apt update -qq
sudo apt upgrade -y -qq
print_success "System updated"

# Step 2: Install Rust
print_info "Step 2/7: Installing Rust toolchain..."
if command -v rustc &> /dev/null; then
    print_info "Rust already installed: $(rustc --version)"
else
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
    print_success "Rust installed: $(rustc --version)"
fi

# Step 3: Install dependencies
print_info "Step 3/7: Installing build dependencies..."
sudo apt install -y -qq \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    jq \
    curl \
    protobuf-compiler \
    htop \
    screen
print_success "Dependencies installed"

# Step 4: Clone/copy NetChain
print_info "Step 4/7: Setting up NetChain..."
NETCHAIN_DIR="$HOME/NetChain"

if [ -d "$NETCHAIN_DIR" ]; then
    print_info "NetChain directory exists, pulling latest changes..."
    cd "$NETCHAIN_DIR"
    if [ -d ".git" ]; then
        git pull
    else
        print_info "Not a git repository, skipping pull"
    fi
else
    print_error "NetChain directory not found at $NETCHAIN_DIR"
    echo "Please upload NetChain code to $NETCHAIN_DIR first, then run this script again."
    echo ""
    echo "From your local machine, run:"
    echo "rsync -avz -e 'ssh -i ~/.ssh/your-key.pem' \\"
    echo "  --exclude 'target' --exclude '.git' --exclude 'data' \\"
    echo "  /path/to/NetChain/ \\"
    echo "  ubuntu@YOUR_EC2_IP:~/NetChain/"
    exit 1
fi

# Step 5: Build NetChain
print_info "Step 5/7: Building NetChain (this may take 5-10 minutes)..."
cd "$NETCHAIN_DIR"
source $HOME/.cargo/env
cargo build --release --bin netchain
cargo build --release --bin netchain-wallet
print_success "Build complete"

# Step 6: Create configuration
print_info "Step 6/7: Creating production configuration..."
mkdir -p "$NETCHAIN_DIR/config"

if [ ! -f "$NETCHAIN_DIR/config/production.toml" ]; then
    # Copy default config if production doesn't exist
    if [ -f "$NETCHAIN_DIR/config/default.toml" ]; then
        cp "$NETCHAIN_DIR/config/default.toml" "$NETCHAIN_DIR/config/production.toml"
        
        # Update bind addresses for production
        sed -i 's/bind_addr = "127.0.0.1"/bind_addr = "0.0.0.0"/' "$NETCHAIN_DIR/config/production.toml"
        
        print_success "Production config created from default.toml"
    else
        print_error "No default config found"
    fi
else
    print_info "Production config already exists"
fi

# Add environment variables to .bashrc
if ! grep -q "NETCHAIN_CONFIG" ~/.bashrc; then
    echo "" >> ~/.bashrc
    echo "# NetChain environment" >> ~/.bashrc
    echo "export NETCHAIN_CONFIG=$NETCHAIN_DIR/config/production.toml" >> ~/.bashrc
    echo "export RUST_LOG=info" >> ~/.bashrc
    print_success "Environment variables added to .bashrc"
fi

# Step 7: Create systemd service
print_info "Step 7/7: Creating systemd service..."
sudo tee /etc/systemd/system/netchain.service > /dev/null <<EOF
[Unit]
Description=NetChain Blockchain Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$NETCHAIN_DIR
Environment="RUST_LOG=info"
ExecStart=$NETCHAIN_DIR/target/release/netchain
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=netchain

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=$NETCHAIN_DIR/data

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable netchain
print_success "Systemd service created and enabled"

# Setup complete
echo ""
echo "======================================"
print_success "Setup Complete!"
echo "======================================"
echo ""
echo "Next steps:"
echo ""
echo "1. Review configuration:"
echo "   nano $NETCHAIN_DIR/config/production.toml"
echo ""
echo "2. Start the node:"
echo "   sudo systemctl start netchain"
echo ""
echo "3. Check status:"
echo "   sudo systemctl status netchain"
echo ""
echo "4. View logs:"
echo "   sudo journalctl -u netchain -f"
echo ""
echo "5. Test endpoints:"
echo "   curl http://localhost:9090/health | jq ."
echo "   curl http://localhost:9090/metrics"
echo ""
echo "6. Configure firewall (UFW):"
echo "   sudo ufw allow 22/tcp      # SSH"
echo "   sudo ufw allow 30333/tcp   # P2P"
echo "   sudo ufw allow 8545/tcp    # RPC"
echo "   sudo ufw allow 8546/tcp    # WebSocket"
echo "   sudo ufw enable"
echo ""
echo "Binary locations:"
echo "  Node:   $NETCHAIN_DIR/target/release/netchain"
echo "  Wallet: $NETCHAIN_DIR/target/release/netchain-wallet"
echo ""
print_info "For detailed documentation, see DEPLOYMENT.md"
