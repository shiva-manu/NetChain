# NetChain AWS EC2 Deployment Guide

This guide will walk you through deploying the NetChain node on an AWS EC2 instance.

## Prerequisites

- AWS Account with EC2 access
- SSH key pair for EC2 access
- Basic familiarity with AWS Console and SSH

## Step 1: Launch EC2 Instance

### Recommended Instance Type
- **Minimum**: `t3.medium` (2 vCPU, 4 GB RAM)
- **Recommended**: `t3.large` (2 vCPU, 8 GB RAM) or higher
- **Storage**: 30 GB+ SSD (gp3)

### AMI Selection
- **Ubuntu Server 22.04 LTS** (recommended)
- Or **Ubuntu Server 24.04 LTS**

### Launch Steps

1. **Go to EC2 Dashboard**
   - Navigate to: https://console.aws.amazon.com/ec2/

2. **Launch Instance**
   - Click "Launch Instance"
   - Name: `netchain-node`

3. **Choose AMI**
   - Select: Ubuntu Server 22.04 LTS (64-bit x86)

4. **Instance Type**
   - Select: `t3.medium` or `t3.large`

5. **Key Pair**
   - Select existing key pair or create new one
   - Download `.pem` file if creating new

6. **Network Settings**
   - Create or select a security group with the following inbound rules:
     - **SSH**: Port 22 (from your IP)
     - **P2P**: Port 30333 (from anywhere 0.0.0.0/0)
     - **RPC**: Port 8545 (from anywhere 0.0.0.0/0)
     - **WebSocket**: Port 8546 (from anywhere 0.0.0.0/0)
     - **Monitoring**: Port 9090 (from your IP or trusted sources)

7. **Storage**
   - Configure: 30 GB gp3 SSD (minimum)

8. **Launch Instance**
   - Click "Launch Instance"
   - Wait for instance to reach "Running" state

## Step 2: Connect to EC2 Instance

```bash
# Make key file read-only
chmod 400 your-key.pem

# Connect via SSH (replace with your instance public IP)
ssh -i your-key.pem ubuntu@<EC2_PUBLIC_IP>
```

## Step 3: Transfer NetChain Code to EC2

### Option A: Using Git (Recommended if you have a remote repo)

```bash
# On EC2 instance
git clone https://github.com/your-username/netchain.git
cd netchain
```

### Option B: Using SCP (From your local machine)

```bash
# From your local machine (in NetChain directory)
cd /home/mani/Desktop/NetChain
tar -czf netchain.tar.gz --exclude='target' --exclude='node_modules' --exclude='.git' .

# Copy to EC2
scp -i your-key.pem netchain.tar.gz ubuntu@<EC2_PUBLIC_IP>:~

# On EC2 instance
ssh -i your-key.pem ubuntu@<EC2_PUBLIC_IP>
mkdir netchain
tar -xzf netchain.tar.gz -C netchain
cd netchain
```

## Step 4: Run Deployment Script

```bash
# On EC2 instance
sudo ./deploy/deploy.sh
```

The script will:
- Install Rust and system dependencies
- Build NetChain in release mode (~10-15 minutes)
- Create systemd service
- Configure firewall rules
- Set up production configuration

## Step 5: Start the Node

```bash
# Start the service
sudo systemctl start netchain

# Check status
sudo systemctl status netchain

# View logs
sudo journalctl -u netchain -f
```

## Step 6: Verify Node is Running

```bash
# Check RPC endpoint
curl http://localhost:8545

# Check from external (replace with your EC2 public IP)
curl http://<EC2_PUBLIC_IP>:8545
```

## Managing the Node

### Common Commands

```bash
# Start node
sudo systemctl start netchain

# Stop node
sudo systemctl stop netchain

# Restart node
sudo systemctl restart netchain

# Check status
sudo systemctl status netchain

# View live logs
sudo journalctl -u netchain -f

# View last 100 lines of logs
sudo journalctl -u netchain -n 100
```

### Configuration

- Config file: `/opt/netchain/config/production.toml`
- Data directory: `/opt/netchain/data`
- Binary: `/opt/netchain/target/release/netchain`

### Updating the Node

```bash
# Stop the service
sudo systemctl stop netchain

# Update code (if using git)
cd /opt/netchain
sudo git pull

# Or upload new files via SCP

# Rebuild
sudo cargo build --release

# Restart service
sudo systemctl start netchain
```

## Security Considerations

### Security Group Rules (AWS Console)

Update EC2 Security Group to restrict access:

1. **SSH (Port 22)**: Only from your IP address
2. **Monitoring (Port 9090)**: Only from trusted IPs
3. **RPC/WebSocket**: Consider using AWS VPC or restricting to known clients
4. **P2P (Port 30333)**: Can remain open for blockchain network

### Additional Hardening

```bash
# Update system packages
sudo apt-get update && sudo apt-get upgrade -y

# Enable automatic security updates
sudo apt-get install unattended-upgrades -y
sudo dpkg-reconfigure -plow unattended-upgrades

# Set up fail2ban for SSH protection
sudo apt-get install fail2ban -y
sudo systemctl enable fail2ban
sudo systemctl start fail2ban
```

## Monitoring

### Check Node Health

```bash
# View monitoring endpoint
curl http://localhost:9090

# Check disk usage
df -h

# Check memory usage
free -h

# Check CPU usage
top
```

### CloudWatch Monitoring (Optional)

Consider setting up AWS CloudWatch for:
- CPU utilization alerts
- Disk space alerts
- Network traffic monitoring
- Custom metrics from the node

## Troubleshooting

### Node won't start

```bash
# Check logs for errors
sudo journalctl -u netchain -n 200

# Check if ports are already in use
sudo netstat -tulpn | grep -E '30333|8545|8546|9090'

# Verify permissions
ls -la /opt/netchain/data
```

### Build fails

```bash
# Ensure Rust is installed
rustc --version

# Update Rust
rustup update

# Clean and rebuild
cd /opt/netchain
cargo clean
cargo build --release
```

### Cannot connect to RPC

```bash
# Check if service is running
sudo systemctl status netchain

# Check firewall
sudo ufw status

# Verify AWS Security Group allows inbound traffic on port 8545
```

## Estimated Costs (AWS us-east-1)

- **t3.medium**: ~$30/month (2 vCPU, 4 GB RAM)
- **t3.large**: ~$60/month (2 vCPU, 8 GB RAM)
- **Storage (30GB gp3)**: ~$2.40/month
- **Data transfer**: Variable based on usage

## Node Endpoints

Once deployed, your node will be available at:

- **RPC**: `http://<EC2_PUBLIC_IP>:8545`
- **WebSocket**: `ws://<EC2_PUBLIC_IP>:8546`
- **Monitoring**: `http://<EC2_PUBLIC_IP>:9090`
- **P2P**: Port `30333`

## Next Steps

1. **Create a wallet**: Use the wallet CLI to create an account
   ```bash
   /opt/netchain/target/release/netchain-wallet create
   ```

2. **Monitor node performance**: Check the monitoring endpoint regularly

3. **Set up automated backups**: Back up `/opt/netchain/data` regularly

4. **Configure alerts**: Set up CloudWatch alarms for critical metrics

5. **Join the network**: Connect to other NetChain nodes via P2P

## Support

For issues or questions:
- Check logs: `sudo journalctl -u netchain -f`
- Review configuration: `/opt/netchain/config/production.toml`
- Check system resources: `top`, `df -h`, `free -h`
