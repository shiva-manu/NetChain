# NetChain AWS EC2 Deployment - Quick Start

Three methods to deploy NetChain on AWS EC2:

## Method 1: Automated Setup Script (Recommended)

### Prerequisites
- AWS EC2 instance running Ubuntu 22.04/24.04
- SSH access to the instance

### Steps

**1. Launch EC2 Instance (AWS Console)**
- AMI: Ubuntu 22.04 LTS
- Instance type: t3.medium or larger
- Storage: 30GB+ gp3
- Security group: Open ports 22, 8545, 8546, 9090, 30333

**2. Upload NetChain Code**
```bash
# From your local machine
rsync -avz -e "ssh -i ~/.ssh/your-key.pem" \
  --exclude 'target' --exclude '.git' --exclude 'data' \
  /home/mani/Desktop/NetChain/ \
  ubuntu@YOUR_EC2_IP:~/NetChain/
```

**3. Run Setup Script**
```bash
# SSH into instance
ssh -i ~/.ssh/your-key.pem ubuntu@YOUR_EC2_IP

# Run setup
cd ~/NetChain
bash scripts/setup-ec2.sh
```

**4. Start Node**
```bash
sudo systemctl start netchain
sudo systemctl status netchain
```

**5. Verify**
```bash
curl http://localhost:9090/health | jq .
```

---

## Method 2: Terraform Infrastructure-as-Code

### Prerequisites
- Terraform installed locally
- AWS CLI configured with credentials
- SSH key pair created in AWS EC2

### Steps

**1. Configure Variables**
```bash
cd terraform/
cp terraform.tfvars.example terraform.tfvars
nano terraform.tfvars  # Edit with your values
```

**2. Deploy Infrastructure**
```bash
terraform init
terraform plan
terraform apply
```

**3. Note Outputs**
```bash
terraform output ssh_command
terraform output rpc_endpoint
```

**4. Upload Code & Setup**
```bash
# Use the SSH command from output
ssh -i ~/.ssh/your-key.pem ubuntu@ELASTIC_IP

# Upload code (from local machine)
rsync -avz -e "ssh -i ~/.ssh/your-key.pem" \
  --exclude 'target' --exclude '.git' \
  /home/mani/Desktop/NetChain/ \
  ubuntu@ELASTIC_IP:~/NetChain/

# Run setup (on EC2)
cd ~/NetChain
bash scripts/setup-ec2.sh
sudo systemctl start netchain
```

**5. Cleanup (when done)**
```bash
terraform destroy
```

---

## Method 3: Manual Step-by-Step

See **DEPLOYMENT.md** for complete manual instructions.

---

## Post-Deployment

### Enable HTTPS (Recommended)

Set up SSL/HTTPS with automatic Let's Encrypt certificates:

**1. Add DNS Record**

Add an A record pointing your subdomain to your EC2 IP:
| Type | Name | Value |
|------|------|-------|
| A | api | YOUR_EC2_IP |

**2. Open Port 443 in AWS Security Group**
```bash
# AWS Console: EC2 > Security Groups > Edit inbound rules
# Add: HTTPS (443) from 0.0.0.0/0
```

**3. Run SSL Setup Script**
```bash
ssh -i ~/.ssh/your-key.pem ubuntu@YOUR_EC2_IP
cd ~/NetChain
sudo bash scripts/setup-ssl.sh api.yourdomain.com
```

**4. Verify HTTPS**
```bash
curl https://api.yourdomain.com/health
curl https://api.yourdomain.com/metrics
```

Your endpoints will be:
- RPC: `https://api.yourdomain.com/rpc`
- WebSocket: `wss://api.yourdomain.com/ws`
- Metrics: `https://api.yourdomain.com/metrics`

---

### Check Node Status
```bash
sudo systemctl status netchain
sudo journalctl -u netchain -f
```

### Test Endpoints
```bash
# Health check
curl http://YOUR_IP:9090/health | jq .

# Metrics
curl http://YOUR_IP:9090/metrics

# Chain info (RPC)
curl -X POST http://YOUR_IP:8545/rpc \
  -H "Content-Type: application/json" \
  -d '{"method":"get_chain_info"}' | jq .
```

### Monitor Resources
```bash
htop                    # CPU/Memory
df -h                   # Disk usage
sudo ufw status         # Firewall
```

---

## Firewall Setup (Important!)

```bash
sudo ufw allow 22/tcp       # SSH
sudo ufw allow 80/tcp       # HTTP (for Let's Encrypt)
sudo ufw allow 443/tcp      # HTTPS
sudo ufw allow 30333/tcp    # P2P

# Only if NOT using HTTPS reverse proxy:
# sudo ufw allow 8545/tcp   # RPC (direct)
# sudo ufw allow 8546/tcp   # WebSocket (direct)
# sudo ufw allow from YOUR_IP to any port 9090  # Metrics

sudo ufw enable
```

---

## Useful Commands

```bash
# Service management
sudo systemctl start netchain
sudo systemctl stop netchain
sudo systemctl restart netchain
sudo systemctl status netchain

# Logs
sudo journalctl -u netchain -f          # Follow logs
sudo journalctl -u netchain -n 100      # Last 100 lines
sudo journalctl -u netchain --since today

# Backup
tar -czf backup.tar.gz ~/NetChain/data/

# Update node
cd ~/NetChain
git pull  # or upload new code
cargo build --release
sudo systemctl restart netchain
```

---

## Troubleshooting

**Node won't start:**
```bash
sudo journalctl -u netchain -n 100
~/NetChain/target/release/netchain  # Run manually
```

**Can't connect to endpoints:**
```bash
sudo ufw status                      # Check firewall
sudo netstat -tulpn | grep netchain  # Check ports
```

**Out of disk space:**
```bash
df -h
du -sh ~/NetChain/data/*
```

---

## Security Checklist

- [ ] Enable HTTPS with SSL certificates (scripts/setup-ssl.sh)
- [ ] Restrict SSH to your IP only
- [ ] Close direct ports (8545, 8546, 9090) - use HTTPS proxy instead
- [ ] Enable UFW firewall
- [ ] Use SSH keys (disable password auth)
- [ ] Enable automatic security updates
- [ ] Set up CloudWatch monitoring
- [ ] Regular backups of data directory

---

## Cost Estimate (AWS)

**t3.medium (2 vCPU, 4GB RAM):**
- Instance: ~$30/month
- Storage (50GB gp3): ~$5/month
- Data transfer: ~$10/month
- **Total: ~$45/month**

**t3.large (2 vCPU, 8GB RAM) - Recommended for production:**
- Instance: ~$60/month
- Storage (100GB gp3): ~$10/month
- Data transfer: ~$10/month
- **Total: ~$80/month**

---

## Support

- Full documentation: `DEPLOYMENT.md`
- Issues: GitHub Issues
- Configuration: `config/production.toml`
