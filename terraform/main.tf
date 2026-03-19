# NetChain EC2 Terraform Configuration
# This creates a production-ready EC2 instance for running NetChain

terraform {
  required_version = ">= 1.0"
  
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

# Configure AWS Provider
provider "aws" {
  region = var.aws_region
  
  default_tags {
    tags = {
      Project     = "NetChain"
      Environment = var.environment
      ManagedBy   = "Terraform"
    }
  }
}

# Variables
variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "production"
}

variable "instance_type" {
  description = "EC2 instance type"
  type        = string
  default     = "t3.medium"
}

variable "key_name" {
  description = "SSH key pair name"
  type        = string
}

variable "allowed_ssh_cidr" {
  description = "CIDR blocks allowed to SSH"
  type        = list(string)
  default     = ["0.0.0.0/0"]  # Change to your IP for better security
}

variable "allowed_monitoring_cidr" {
  description = "CIDR blocks allowed to access monitoring"
  type        = list(string)
  default     = ["0.0.0.0/0"]  # Change to your IP for better security
}

# Data source for latest Ubuntu 22.04 AMI
data "aws_ami" "ubuntu" {
  most_recent = true
  owners      = ["099720109477"] # Canonical

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-jammy-22.04-amd64-server-*"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
}

# Security Group
resource "aws_security_group" "netchain" {
  name        = "netchain-node-${var.environment}"
  description = "Security group for NetChain blockchain node"

  # SSH access
  ingress {
    description = "SSH"
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = var.allowed_ssh_cidr
  }

  # P2P networking
  ingress {
    description = "P2P"
    from_port   = 30333
    to_port     = 30333
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # RPC endpoint
  ingress {
    description = "RPC"
    from_port   = 8545
    to_port     = 8545
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # WebSocket endpoint
  ingress {
    description = "WebSocket"
    from_port   = 8546
    to_port     = 8546
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Monitoring endpoint (restricted)
  ingress {
    description = "Monitoring"
    from_port   = 9090
    to_port     = 9090
    protocol    = "tcp"
    cidr_blocks = var.allowed_monitoring_cidr
  }

  # Outbound - allow all
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "netchain-sg-${var.environment}"
  }
}

# Elastic IP
resource "aws_eip" "netchain" {
  domain   = "vpc"
  instance = aws_instance.netchain.id

  tags = {
    Name = "netchain-eip-${var.environment}"
  }
}

# EC2 Instance
resource "aws_instance" "netchain" {
  ami           = data.aws_ami.ubuntu.id
  instance_type = var.instance_type
  key_name      = var.key_name

  vpc_security_group_ids = [aws_security_group.netchain.id]

  root_block_device {
    volume_type = "gp3"
    volume_size = 50
    iops        = 3000
    throughput  = 125
    encrypted   = true

    tags = {
      Name = "netchain-root-${var.environment}"
    }
  }

  user_data = templatefile("${path.module}/user-data.sh", {
    environment = var.environment
  })

  metadata_options {
    http_endpoint               = "enabled"
    http_tokens                 = "required"
    http_put_response_hop_limit = 1
  }

  tags = {
    Name = "netchain-node-${var.environment}"
  }
}

# CloudWatch Log Group (optional)
resource "aws_cloudwatch_log_group" "netchain" {
  name              = "/aws/ec2/netchain-${var.environment}"
  retention_in_days = 7

  tags = {
    Name = "netchain-logs-${var.environment}"
  }
}

# Outputs
output "instance_id" {
  description = "EC2 instance ID"
  value       = aws_instance.netchain.id
}

output "public_ip" {
  description = "Elastic IP address"
  value       = aws_eip.netchain.public_ip
}

output "public_dns" {
  description = "Public DNS name"
  value       = aws_instance.netchain.public_dns
}

output "ssh_command" {
  description = "SSH command to connect"
  value       = "ssh -i ~/.ssh/${var.key_name}.pem ubuntu@${aws_eip.netchain.public_ip}"
}

output "rpc_endpoint" {
  description = "RPC endpoint URL"
  value       = "http://${aws_eip.netchain.public_ip}:8545"
}

output "websocket_endpoint" {
  description = "WebSocket endpoint URL"
  value       = "ws://${aws_eip.netchain.public_ip}:8546"
}

output "metrics_endpoint" {
  description = "Metrics endpoint URL"
  value       = "http://${aws_eip.netchain.public_ip}:9090/metrics"
}

output "health_endpoint" {
  description = "Health check endpoint URL"
  value       = "http://${aws_eip.netchain.public_ip}:9090/health"
}
