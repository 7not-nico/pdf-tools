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
SECONDS=0
while true; do
    if read -t 1 -p "Enter choice (1 or 2): " tool_choice; then
        case $tool_choice in
            1)
                TOOL="pdf-opticompress"
                break
                ;;
            2)
                TOOL="pdf-renamer"
                break
                ;;
            "")
                # Empty input, ignore
                ;;
            *)
                echo "Invalid choice, try again."
                ;;
        esac
    else
        SECONDS=$((SECONDS + 1))
        if [ $SECONDS -eq 15 ]; then
            echo "Warning: 15 seconds have passed, please enter 1 or 2."
        elif [ $SECONDS -eq 30 ]; then
            echo "No input received in 30 seconds, exiting."
            exit 1
        fi
    fi
done

# Find asset
ASSET_NAME="$TOOL-$OS_NAME"
if [ "$OS_NAME" = "windows-latest" ]; then
    ASSET_NAME="$ASSET_NAME$EXT"
fi

# Parse JSON without jq
ASSET_URL=$(echo "$RELEASE_DATA" | grep -o '"browser_download_url":"[^"]*","name":"'"$ASSET_NAME"'"' | sed 's/.*"browser_download_url":"\([^"]*\)".*/\1/')
if [ -z "$ASSET_URL" ]; then
    echo "Asset not found: $ASSET_NAME"
    echo "Available assets:"
    echo "$RELEASE_DATA" | grep -o '"name":"[^"]*"' | sed 's/.*"name":"\([^"]*\)".*/\1/'
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