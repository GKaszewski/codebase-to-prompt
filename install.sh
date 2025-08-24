#!/bin/sh
# install.sh
#
# This script downloads and installs the latest release of codebase-to-prompt.
set -e

LATEST_RELEASE=$(curl -s "https://api.github.com/repos/GKaszewski/codebase-to-prompt/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

TARGET=""
if [ "$OS" = "linux" ] && [ "$ARCH" = "x86_64" ]; then
    TARGET="x86_64-unknown-linux-gnu"
elif [ "$OS" = "darwin" ] && [ "$ARCH" = "x86_64" ]; then
    TARGET="x86_64-apple-darwin"
else
    echo "Unsupported OS or architecture: $OS $ARCH"
    exit 1
fi

DOWNLOAD_URL="https://github.com/GKaszewski/codebase-to-prompt/releases/download/$LATEST_RELEASE/codebase-to-prompt-$TARGET"
curl -L "$DOWNLOAD_URL" -o codebase-to-prompt

chmod +x codebase-to-prompt
mv codebase-to-prompt /usr/local/bin/

echo "codebase-to-prompt installed successfully!"