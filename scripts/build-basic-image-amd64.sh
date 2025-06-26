#!/bin/bash

# build-basic-image-amd64.sh
# Build basic image for amd64 platform using Dockerfile.basic
# Usage: ./build-basic-image-amd64.sh [TAG]
#   TAG - Optional tag override (will use git tag if not provided)

set -e

# Get the script directory for relative path resolution
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to project root
cd "$PROJECT_ROOT"

# Source the basic image tag script
source "$SCRIPT_DIR/get-basic-image-tag.sh"

# Use the tag from the sourced script
IMAGE_TAG="$BASIC_IMAGE_TAG"
ARCH="amd64"
IMAGE_NAME="ghcr.io/daksha-rc/rc-basic"
FULL_TAG="${IMAGE_TAG}-${ARCH}"

echo "🏗️  Building basic image for ${ARCH}..."
echo "📦 Image: ${IMAGE_NAME}:${FULL_TAG}"
echo "📁 Project root: $PROJECT_ROOT"

# Verify Dockerfile exists
if [ ! -f "rc-web/basic-image/Dockerfile.basic" ]; then
    echo "❌ Error: Dockerfile.basic not found at rc-web/basic-image/Dockerfile.basic"
    exit 1
fi

# Build the image
echo "🔨 Building with Podman..."
podman build \
    --arch=${ARCH} \
    --tag="${IMAGE_NAME}:${FULL_TAG}" \
    --tag="${IMAGE_NAME}:latest-${ARCH}" \
    --file=rc-web/basic-image/Dockerfile.basic \
    --label="org.opencontainers.image.version=${IMAGE_TAG}" \
    --label="org.opencontainers.image.revision=$(git rev-parse HEAD)" \
    --label="org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --label="org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc-core" \
    --label="org.opencontainers.image.title=RC Basic Image" \
    --label="org.opencontainers.image.description=Basic Alpine image for Daksha RC" \
    --label="org.opencontainers.image.platform=linux/${ARCH}" \
    .

echo "✅ Successfully built basic image for ${ARCH}:"
echo "   📦 ${IMAGE_NAME}:${FULL_TAG}"
echo "   📦 ${IMAGE_NAME}:latest-${ARCH}"

# Verify the image was created
if podman image exists "${IMAGE_NAME}:${FULL_TAG}"; then
    echo "🔍 Image verification: SUCCESS"
    podman image inspect "${IMAGE_NAME}:${FULL_TAG}" --format "Size: {{.Size}}"
else
    echo "❌ Image verification: FAILED"
    exit 1
fi
