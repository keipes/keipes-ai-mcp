#!/usr/bin/env bash
# filepath: scripts/deploy.sh
set -e

# # Build binary locally
# export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
# export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
# cargo build --target aarch64-unknown-linux-gnu --release
# just build

# Define deployment targets
EC2_HOST="keipes"
OVH_HOST="your-ovh-host"
BINARY_NAME="main"
SERVICE_NAME="keipes-ai-mcp"

deploy_to_host() {
    local host=$1
    echo "Deploying to $host..."
    
    # Copy binary
    scp -C target/aarch64-unknown-linux-gnu/release/$BINARY_NAME $host:~/
    # rsync -avz --progress target/aarch64-unknown-linux-gnu/release/$BINARY_NAME $host:/tmp/ 

    # Deploy with systemd restart
    ssh $host << EOF
        sudo systemctl stop $SERVICE_NAME || true
        sudo mv ~/$BINARY_NAME /usr/local/bin/$SERVICE_NAME
        sudo chmod +x /usr/local/bin/$SERVICE_NAME
        sudo systemctl start $SERVICE_NAME
        sudo systemctl status $SERVICE_NAME
EOF
}

# Deploy to both hosts
deploy_to_host $EC2_HOST
# deploy_to_host $OVH_HOST