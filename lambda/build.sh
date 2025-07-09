#!/bin/bash

# Build script for Lambda deployment

set -e

echo "Building Go binary for Lambda..."

# Create build directory
mkdir -p build

# Build the binary for Linux (Lambda runtime)
cd ../src
GOOS=linux GOARCH=amd64 CGO_ENABLED=0 go build -o ../lambda/build/bootstrap .

echo "Binary built successfully at lambda/build/bootstrap"

# Create deployment package
cd ../lambda
mkdir -p build
cd build
zip -r ../bootstrap.zip bootstrap

echo "Deployment package created: bootstrap.zip"
