# Lambda Build Makefile

.PHONY: build clean package deploy

BINARY_NAME=bootstrap
BUILD_DIR=build
ZIP_FILE=bootstrap.zip

build:
	@echo "Building Lambda binary..."
	GOOS=linux GOARCH=amd64 CGO_ENABLED=0 go build -o $(BUILD_DIR)/$(BINARY_NAME) ../src/main.go

clean:
	@echo "Cleaning build artifacts..."
	rm -rf $(BUILD_DIR)
	rm -f $(ZIP_FILE)

package: build
	@echo "Creating deployment package..."
	cd $(BUILD_DIR) && zip -r ../$(ZIP_FILE) $(BINARY_NAME)

deploy: package
	@echo "Deploying to AWS..."
	cd ../infrastructure && cdk deploy

all: clean build package deploy
