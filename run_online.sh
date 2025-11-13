#!/bin/bash

# Temporary PDF Tools Runner
# Downloads and runs the latest binary from GitHub releases

set -e

REPO="7not-nico/pdf-tools"
API_URL="https://api.github.com/repos/$REPO/releases/latest"

echo "PDF Tools Temporary Runner"
echo "=========================="

# Detect OS
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
case $OS in
    linux)
        OS_NAME="ubuntu-latest"
        ;;
    darwin)
        OS_NAME="macos-latest"
        ;;
    mingw*|msys*|cygwin*)
        OS_NAME="windows-latest"
        EXT=".exe"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Detected OS: $OS_NAME"

# Get latest release
echo "Fetching latest release..."
RELEASE_DATA=$(curl -s $API_URL)
if [ $? -ne 0 ]; then
    echo "Failed to fetch release data"
    exit 1
fi

# Choose tool
echo ""
echo "Choose tool:"
echo "1. pdf-opticompress"
echo "2. pdf-renamer"
read -p "Enter choice (1 or 2): " tool_choice

case $tool_choice in
    1)
        TOOL="pdf-opticompress"
        ;;
    2)
        TOOL="pdf-renamer"
        ;;
    *)
        echo "Invalid choice"
        exit 1
        ;;
esac

# Find asset
ASSET_NAME="$TOOL-$OS_NAME"
if [ "$OS_NAME" = "windows-latest" ]; then
    ASSET_NAME="$ASSET_NAME$EXT"
fi

ASSET_URL=$(echo $RELEASE_DATA | jq -r ".assets[] | select(.name == \"$ASSET_NAME\") | .browser_download_url")
if [ -z "$ASSET_URL" ] || [ "$ASSET_URL" = "null" ]; then
    echo "Asset not found: $ASSET_NAME"
    echo "Available assets:"
    echo $RELEASE_DATA | jq -r ".assets[].name"
    exit 1
fi

echo "Downloading $TOOL..."
TEMP_DIR=$(mktemp -d)
BINARY_PATH="$TEMP_DIR/$TOOL$EXT"

curl -L -o "$BINARY_PATH" "$ASSET_URL"
if [ $? -ne 0 ]; then
    echo "Failed to download binary"
    exit 1
fi

chmod +x "$BINARY_PATH"

echo "Running $TOOL..."
echo "Note: For URLs, paste the full URL when prompted."
echo ""

"$BINARY_PATH"

echo ""
echo "Cleaning up..."
rm -rf "$TEMP_DIR"

echo "Done."