#!/bin/bash
# User data script for EC2 instance initialization
# This runs automatically when the instance first boots

set -e

# Update system
apt-get update
apt-get upgrade -y

# Install dependencies
apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    jq \
    curl \
    protobuf-compiler \
    htop \
    screen \
    awscli

# Install Rust as ubuntu user
sudo -u ubuntu bash -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'

# Create NetChain directory
sudo -u ubuntu mkdir -p /home/ubuntu/NetChain

# Note: You'll need to deploy your code separately
# Either via git clone or rsync after instance creation

echo "EC2 instance initialized. Ready for NetChain deployment."
echo "Run the setup script: bash /home/ubuntu/NetChain/scripts/setup-ec2.sh"
