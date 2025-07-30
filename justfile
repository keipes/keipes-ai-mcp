# Justfile for Keipes AI MCP
# cargo install just

# Variables
project := "keipes-ai-mcp"
port := "80"

set windows-shell := ["powershell.exe", "-c"]

# # Build the Docker image
# build:
#     docker build -t {{project}} .

# # Run the Docker container
# run:
#     docker run -d -p {{port}}:{{port}} --name {{project}} {{project}}

# # Stop and remove the Docker container
# stop:
#     docker stop {{project}}; docker rm {{project}}

# # Rebuild and restart the container
# restart:
#     just stop; just build; just run


build-mac:
    brew install aarch64-unknown-linux-gnu
    rustup target add aarch64-unknown-linux-gnu
    export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
    cargo build --target aarch64-unknown-linux-gnu --release

build-linux:
    # Install cross if not already installed
    cargo install cross
    export PATH=~/.cargo/bin:$PATH
    # Install the target if not already installed
    rustup target add aarch64-unknown-linux-gnu
    # Set the linker for the target
    # export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
    # export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
    # Build the project for the target
    cross build --target aarch64-unknown-linux-gnu --release

# just --shell powershell.exe --shell-arg -c build-windows
build-windows:
    rustup target add aarch64-unknown-linux-gnu
    rustup toolchain add stable-x86_64-unknown-linux-gnu --force-non-host
    # $Env:CC_aarch64_unknown_linux_gnu="aarch64-linux-gnu-gcc"
    # $Env:CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-linux-gnu-gcc"
    # cargo build --target aarch64-unknown-linux-gnu --release
    cross build --target aarch64-unknown-linux-gnu --release

deploy:
    # just build-linux
    scripts/deploy.sh

setup-systemd host:
    # scp scripts/systemd/keipes-ai-mcp.service $host:/etc/systemd/system/keipes-ai-mcp.service
    # ssh $host "sudo systemctl daemon-reload && sudo systemctl enable keipes-ai-mcp"
    scp scripts/systemd/keipes-ai-mcp.service {{host}}:/tmp/
    ssh {{host}} "sudo mv /tmp/keipes-ai-mcp.service /etc/systemd/system/ && sudo systemctl daemon-reload && sudo systemctl enable keipes-ai-mcp"