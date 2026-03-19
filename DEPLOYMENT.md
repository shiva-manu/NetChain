# NetChain AWS EC2 Deployment Guide

This guide walks you through deploying a NetChain blockchain node on AWS EC2.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [EC2 Instance Setup](#ec2-instance-setup)
3. [Installation](#installation)
4. [Configuration](#configuration)
5. [Running the Node](#running-the-node)
6. [Monitoring & Maintenance](#monitoring--maintenance)
7. [Security Best Practices](#security-best-practices)
8. [Troubleshooting](#troubleshooting)

---

## Prerequisites

- AWS account with EC2 access
- SSH key pair for EC2 access
- Basic knowledge of Linux command line
- (Optional) Domain name for your node

---

## EC2 Instance Setup

### 1. Launch EC2 Instance

**Recommended Specifications:**
- **Instance Type**: `t3.medium` or larger (2 vCPU, 4GB RAM minimum)
- **OS**: Ubuntu 22.04 LTS (or Ubuntu 24.04 LTS)
- **Storage**: 30GB+ EBS volume (gp3 recommended)
- **Region**: Choose based on your location/needs

**Step-by-step:**

1. Go to AWS EC2 Console → Launch Instance
2. Configure:
   - **Name**: `netchain-node-1`
   - **AMI**: Ubuntu Server 22.04 LTS (HVM), SSD Volume Type
   - **Instance type**: `t3.medium`
   - **Key pair**: Select existing or create new
   - **Network settings**: Create/select security group (see below)
   - **Storage**: 30 GB gp3

### 2. Security Group Configuration

Configure inbound rules:

| Type        | Protocol | Port Range | Source          | Description                    |
|-------------|----------|------------|-----------------|--------------------------------|
| SSH         | TCP      | 22         | Your IP/0.0.0.0 | SSH access                     |
| Custom TCP  | TCP      | 8545       | 0.0.0.0/0       | RPC endpoint (or restrict)     |
| Custom TCP  | TCP      | 8546       | 0.0.0.0/0       | WebSocket endpoint             |
| Custom TCP  | TCP      | 9090       | Your IP         | Metrics/monitoring (restrict!) |
| Custom TCP  | TCP      | 30333      | 0.0.0.0/0       | P2P networking                 |

**Security Notes:**
- **Port 9090**: Only expose to trusted IPs (monitoring data)
- **Port 8545**: Consider restricting to known IPs if not public RPC
- **Port 22**: Restrict to your IP for better security

### 3. Elastic IP (Optional but Recommended)

1. Allocate an Elastic IP
2. Associate it with your EC2 instance
3. This ensures your node's IP doesn't change on restart

---

## Installation

### 1. Connect to Your Instance

```bash
# Replace with your key path and EC2 public IP/DNS
ssh -i ~/.ssh/your-key.pem ubuntu@ec2-XX-XX-XX-XX.compute-1.amazonaws.com
```

### 2. Update System

```bash
sudo apt update && sudo apt upgrade -y
```

### 3. Install Rust

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Load Rust environment
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### 4. Install Dependencies

```bash
# Build essentials and required libraries
sudo apt install -y build-essential pkg-config libssl-dev git jq curl

# Install protobuf compiler (required by libp2p)
sudo apt install -y protobuf-compiler
```

### 5. Clone NetChain Repository

**Option A: From your Git repository (if pushed)**
```bash
cd ~
git clone https://github.com/YOUR_USERNAME/NetChain.git
cd NetChain
```

**Option B: Upload from local machine**
```bash
# On your local machine:
rsync -avz -e "ssh -i ~/.ssh/your-key.pem" \
  --exclude 'target' --exclude '.git' --exclude 'data' \
  /home/mani/Desktop/NetChain/ \
  ubuntu@ec2-XX-XX-XX-XX.compute-1.amazonaws.com:~/NetChain/
```

### 6. Build NetChain

```bash
cd ~/NetChain

# Build release binary (optimized, takes 5-10 minutes)
cargo build --release --bin netchain

# Build wallet CLI
cargo build --release --bin netchain-wallet

# Verify binaries
ls -lh target/release/netchain
ls -lh target/release/netchain-wallet
```

---

## Configuration

### 1. Create Configuration Directory

```bash
mkdir -p ~/NetChain/config
```

### 2. Create Production Config

```bash
nano ~/NetChain/config/production.toml
```

**Paste this configuration:**

```toml
# Production configuration for NetChain node

[node]
log_level = "info"  # Options: trace, debug, info, warn, error

[data]
data_dir = "./data"  # Blockchain data storage

[node.p2p]
p2p_port = 30333
# Add bootstrap nodes here when available
# bootstrap_nodes = ["12D3KooW...@bootstrap.netchain.io:30333"]

[rpc]
enabled = true
bind_addr = "0.0.0.0"  # Bind to all interfaces
port = 8545

[websocket]
enabled = true
bind_addr = "0.0.0.0"
port = 8546

[monitoring]
enabled = true
bind_addr = "0.0.0.0"  # Change to "127.0.0.1" to restrict access
port = 9090

[producer]
max_txs_per_block = 1000
block_interval_secs = 12
block_reward = 1000
mempool_ttl_secs = 3600
metric_measurement_interval_secs = 300  # 5 minutes
metric_announcement_interval_secs = 600  # 10 minutes
epoch_blocks = 100
stake_weight = 0.3  # 30% stake, 70% PoI metrics

[measurement]
enabled = true
sample_count = 5
interval_secs = 60
timeout_secs = 10
test_urls = [
    "https://speed.cloudflare.com/__down?bytes=10000000",
    "https://proof.ovh.net/files/10Mb.dat",
]

[aggregator]
min_attestations = 3
attestation_window_blocks = 50
decay_rate = 0.95

[anti_gaming]
max_speed_mbps = 10000.0
min_speed_mbps = 0.01
max_latency_ms = 1000.0
max_stddev_multiplier = 3.0
rate_limit_window_secs = 300
max_challenges_per_window = 10
min_attestations_trusted = 5
```

Save and exit (`Ctrl+X`, `Y`, `Enter`).

### 3. Set Environment Variables

```bash
# Add to ~/.bashrc for persistence
echo 'export NETCHAIN_CONFIG=~/NetChain/config/production.toml' >> ~/.bashrc
echo 'export RUST_LOG=info' >> ~/.bashrc
source ~/.bashrc
```

---

## Running the Node

### Option 1: Run in Foreground (Testing)

```bash
cd ~/NetChain
./target/release/netchain
```

Press `Ctrl+C` to stop.

### Option 2: Run as Systemd Service (Recommended for Production)

#### Create Service File

```bash
sudo nano /etc/systemd/system/netchain.service
```

**Paste this configuration:**

```ini
[Unit]
Description=NetChain Blockchain Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/NetChain
Environment="RUST_LOG=info"
ExecStart=/home/ubuntu/NetChain/target/release/netchain
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
ReadWritePaths=/home/ubuntu/NetChain/data

[Install]
WantedBy=multi-user.target
```

Save and exit.

#### Enable and Start Service

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service to start on boot
sudo systemctl enable netchain

# Start the service
sudo systemctl start netchain

# Check status
sudo systemctl status netchain
```

#### Service Management Commands

```bash
# View logs
sudo journalctl -u netchain -f

# View last 100 lines
sudo journalctl -u netchain -n 100

# Stop service
sudo systemctl stop netchain

# Restart service
sudo systemctl restart netchain

# Disable service
sudo systemctl disable netchain
```

### Option 3: Run with Screen/Tmux (Alternative)

```bash
# Install screen
sudo apt install -y screen

# Start screen session
screen -S netchain

# Run node
cd ~/NetChain
./target/release/netchain

# Detach: Press Ctrl+A, then D
# Reattach: screen -r netchain
# Kill session: screen -X -S netchain quit
```

---

## Monitoring & Maintenance

### 1. Health Checks

```bash
# Check if node is running
curl http://localhost:9090/health | jq .

# Check metrics
curl http://localhost:9090/metrics

# Get chain info via RPC
curl -X POST http://localhost:8545/rpc \
  -H "Content-Type: application/json" \
  -d '{"method":"get_chain_info"}' | jq .
```

### 2. Monitor Logs

```bash
# Real-time logs (systemd)
sudo journalctl -u netchain -f

# Filter by priority
sudo journalctl -u netchain -p err  # Errors only
sudo journalctl -u netchain -p warning  # Warnings and above
```

### 3. Check Resource Usage

```bash
# CPU and memory
htop

# Disk usage
df -h
du -sh ~/NetChain/data

# Network connections
sudo netstat -tulpn | grep netchain
```

### 4. Backup Data

```bash
# Stop node first
sudo systemctl stop netchain

# Backup blockchain data
tar -czf netchain-backup-$(date +%Y%m%d).tar.gz ~/NetChain/data/

# Copy to S3 (optional)
aws s3 cp netchain-backup-*.tar.gz s3://your-bucket/backups/

# Restart node
sudo systemctl start netchain
```

### 5. Update Node

```bash
# Stop service
sudo systemctl stop netchain

# Pull latest code (if using git)
cd ~/NetChain
git pull

# Rebuild
cargo build --release --bin netchain

# Restart service
sudo systemctl start netchain
```

---

## Security Best Practices

### 1. Firewall Configuration (UFW)

```bash
# Enable UFW
sudo ufw enable

# Allow SSH (important!)
sudo ufw allow 22/tcp

# Allow NetChain ports
sudo ufw allow 30333/tcp  # P2P
sudo ufw allow 8545/tcp   # RPC
sudo ufw allow 8546/tcp   # WebSocket

# Restrict metrics to your IP only
sudo ufw allow from YOUR_IP_ADDRESS to any port 9090

# Enable firewall
sudo ufw enable

# Check status
sudo ufw status
```

### 2. Secure RPC Endpoint

If you want to restrict RPC access, edit config:

```toml
[rpc]
bind_addr = "127.0.0.1"  # Only localhost
```

Then use SSH tunnel:
```bash
# On your local machine
ssh -i ~/.ssh/your-key.pem -L 8545:localhost:8545 ubuntu@ec2-XX-XX-XX-XX.compute-1.amazonaws.com
```

### 3. Regular Updates

```bash
# Update system packages weekly
sudo apt update && sudo apt upgrade -y
sudo reboot  # If kernel updated
```

### 4. SSH Hardening

```bash
# Disable password authentication (use keys only)
sudo nano /etc/ssh/sshd_config
# Set: PasswordAuthentication no

sudo systemctl restart ssh
```

---

## Troubleshooting

### Node Won't Start

```bash
# Check logs
sudo journalctl -u netchain -n 100

# Common issues:
# 1. Port already in use
sudo netstat -tulpn | grep -E '8545|8546|9090|30333'

# 2. Permission issues
sudo chown -R ubuntu:ubuntu ~/NetChain/data

# 3. Config file errors
~/NetChain/target/release/netchain  # Run manually to see errors
```

### High Memory Usage

```bash
# Check memory
free -h

# Add swap if needed (2GB example)
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

### Disk Space Issues

```bash
# Check disk usage
df -h
du -sh ~/NetChain/data/*

# Clean old logs (if using journald)
sudo journalctl --vacuum-time=7d

# Clean Rust build cache
cd ~/NetChain
cargo clean
```

### Connection Issues

```bash
# Test ports are open
nc -zv localhost 8545
nc -zv localhost 9090

# Check firewall
sudo ufw status

# Check security group in AWS console
```

### Performance Issues

```bash
# 1. Upgrade instance type (t3.medium → t3.large)
# 2. Increase EBS IOPS (gp3 with higher IOPS)
# 3. Monitor with:
top
iostat -x 1
```

---

## CloudWatch Integration (Optional)

Install CloudWatch agent for advanced monitoring:

```bash
# Download and install agent
wget https://s3.amazonaws.com/amazoncloudwatch-agent/ubuntu/amd64/latest/amazon-cloudwatch-agent.deb
sudo dpkg -i -E ./amazon-cloudwatch-agent.deb

# Configure (follow AWS docs)
sudo /opt/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent-config-wizard

# Start agent
sudo /opt/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent-ctl \
  -a fetch-config \
  -m ec2 \
  -s -c file:/opt/aws/amazon-cloudwatch-agent/bin/config.json
```

---

## Quick Reference Commands

```bash
# Start node
sudo systemctl start netchain

# Stop node
sudo systemctl stop netchain

# View logs
sudo journalctl -u netchain -f

# Check status
curl http://localhost:9090/health | jq .

# Backup data
tar -czf backup.tar.gz ~/NetChain/data/

# Update node
cd ~/NetChain && git pull && cargo build --release && sudo systemctl restart netchain
```

---

## Additional Resources

- **Node Configuration**: See `config/default.toml` for all options
- **Wallet CLI Guide**: See `README.md` for wallet usage
- **API Documentation**: See `docs/` directory (if available)

## Support

For issues or questions:
- GitHub Issues: https://github.com/YOUR_USERNAME/NetChain/issues
- Documentation: Check project README.md

---

**Note**: Replace `YOUR_USERNAME`, `ec2-XX-XX-XX-XX`, and other placeholders with your actual values.
