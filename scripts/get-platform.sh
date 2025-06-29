#!/bin/bash
set -e

# Get the current platform architecture
ARCH=$(uname -m)

# Map to Docker/Podman architecture names
case "$ARCH" in
    x86_64)
        echo "amd64"
        ;;
    aarch64|arm64)
        echo "arm64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH" >&2
        exit 1
        ;;
esac
