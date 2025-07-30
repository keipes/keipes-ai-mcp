#!/usr/bin/env bash

set -e

EC2_HOST="keipes"
OVH_HOST="your-ovh-host"
BINARY_NAME="main"
SERVICE_NAME="keipes-ai-mcp"

SOURCE_BIN_PATH="target/aarch64-unknown-linux-gnu/release/$BINARY_NAME"
DEST_BIN_STAGING="~/$BINARY_NAME"
DEST_BIN_PATH="/usr/local/bin/$SERVICE_NAME"

SOURCE_SERVICE_PATH="scripts/systemd/keipes-ai-mcp.service"
DEST_SERVICE_STAGING="~/keipes-ai-mcp.service"
DEST_SERVICE_PATH="/etc/systemd/system/keipes-ai-mcp.service"


check_files_differ() {
    local host="$1"
    local source_service_path="$2"
    local remote_service_path="$3"
    local source_bin_path="$4"
    local remote_bin_path="$5"
    
    local source_service_hash=$(sha256sum "$source_service_path" | awk '{print $1}')
    # echo "local:$source_service_path $source_service_hash"
    local source_bin_hash=$(sha256sum "$source_bin_path" | awk '{print $1}')
    # echo "Check remote hashes"

    # Check if remote files exist and get hashes
    local remote_hashes=$(ssh -q "$host" "
        if [ -f $remote_service_path ]; then
            sha256sum $remote_service_path | awk '{print \$1}'
        else
            echo 'missing'
        fi
        if [ -f $remote_bin_path ]; then
            sha256sum $remote_bin_path | awk '{print \$1}'
        else
            echo 'missing'
        fi
    ")
    
    local remote_service_hash=$(echo "$remote_hashes" | head -n1)
    local remote_bin_hash=$(echo "$remote_hashes" | tail -n1)
    echo "$source_service_hash local:$source_service_path"
    echo "$remote_service_hash $host:$remote_service_path"
    echo "$source_bin_hash local:$source_bin_path"
    echo "$remote_bin_hash $host:$remote_bin_path"
    service_differs=1
    binary_differs=1
    
    if [ "$source_service_hash" != "$remote_service_hash" ]; then
        # echo "Service differs: $source_service_path vs $remote_service_path on $host"
        service_differs=0
    # else
    #     echo "Service unchanged: $source_service_path vs $remote_service_path on $host"
    fi
    
    if [ "$source_bin_hash" != "$remote_bin_hash" ]; then
        # echo "Binary differs: $source_bin_path vs $remote_bin_path on $host"
        binary_differs=0
    # else
    #     echo "Binary unchanged: $source_bin_path vs $remote_bin_path on $host"
    fi
}

deploy_to_host() {
    local host=$1

    # echo "Deploying to $host"
    
    # Check if files differ in single SSH call
    check_files_differ "$host" "$SOURCE_SERVICE_PATH" "$DEST_SERVICE_PATH" "$SOURCE_BIN_PATH" "$DEST_BIN_PATH"
    
    # Update service definition
    if [ "$service_differs" -eq 0 ]; then
        echo "Service staging"
        scp -C "$SOURCE_SERVICE_PATH" "$host:$DEST_SERVICE_STAGING"
    fi
    if [ "$binary_differs" -eq 0 ]; then
        echo "Binary staging"
        scp -C "$SOURCE_BIN_PATH" "$host:$DEST_BIN_STAGING"
    fi
    # Deploy with systemd restart
    if [ "$service_differs" -eq 0 ] || [ "$binary_differs" -eq 0 ]; then
        # echo "Deploying to $host"
            ssh -q "$host" << EOF
set -e
echo "Stopping service"
sudo systemctl stop $SERVICE_NAME || true
if [ $service_differs -eq 0 ]; then
    echo "Copying service file"
    sudo mv $DEST_SERVICE_STAGING $DEST_SERVICE_PATH
    echo "Reloading systemd daemon"
    sudo systemctl daemon-reload
    echo "Enabling service"
    sudo systemctl enable $SERVICE_NAME
fi
if [ $binary_differs -eq 0 ]; then
    echo "Copying binary"
    sudo cp $DEST_BIN_STAGING $DEST_BIN_PATH
    echo "Setting permissions"
    sudo chmod +x $DEST_BIN_PATH
fi
echo "Starting service"
sudo systemctl start $SERVICE_NAME
sudo systemctl status $SERVICE_NAME
EOF
    else
        echo "No changes detected, skipping deployment to $host"
        return 0
    fi
}

# Deploy to both hosts
deploy_to_host $EC2_HOST
# deploy_to_host $OVH_HOST