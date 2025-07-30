#!/usr/bin/env bash

set -e

EC2_HOST="$1"
CERTS_ARN="$2"

if [ -z "$EC2_HOST" ] || [ -z "$CERTS_ARN" ]; then
    echo "Usage: $0 <ec2-host> <s3://bucket/path/to/certs>"
    exit 1
fi

ssh -q "$EC2_HOST" << EOF
    echo "Installing certificates on $EC2_HOST..."
    
    # Create directories for certificates
    sudo mkdir -p /var/keipes-ai-mcp/tls
    sudo mkdir -p /var/keipes-ai-mcp/mcp.diceplz.com

    # Download the certificates from AWS S3
    sudo aws s3 cp "$CERTS_ARN/fullchain.pem" /var/keipes-ai-mcp/tls/fullchain.pem
    sudo aws s3 cp "$CERTS_ARN/privkey.pem" /var/keipes-ai-mcp/tls/privkey.pem

    # Set permissions
    sudo chown root:root /var/keipes-ai-mcp/tls/fullchain.pem
    sudo chown root:root /var/keipes-ai-mcp/tls/privkey.pem
    sudo chmod 644 /var/keipes-ai-mcp/tls/fullchain.pem
    sudo chmod 600 /var/keipes-ai-mcp/tls/privkey.pem

    echo "Certificates installed successfully."
EOF
