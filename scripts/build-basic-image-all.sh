#!/bin/bash

# build-basic-image-all.sh
# Build basic image for all platforms (amd64/arm64) with multi-platform manifests
# Usage: ./build-basic-image-all.sh [TAG]
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
IMAGE_NAME="ghcr.io/daksha-rc/rc-basic"
COMMIT_SHA=$(git rev-parse --short HEAD)

echo "🏗️  Building multi-platform basic image..."
echo "📦 Base image: ${IMAGE_NAME}"
echo "🏷️  Tag: ${IMAGE_TAG}"
echo "📋 Commit: ${COMMIT_SHA}"
echo "📁 Project root: $PROJECT_ROOT"

# Verify Dockerfile exists
if [ ! -f "rc-web/basic-image/Dockerfile.basic" ]; then
    echo "❌ Error: Dockerfile.basic not found at rc-web/basic-image/Dockerfile.basic"
    exit 1
fi

# Clean up any existing manifests to avoid conflicts
echo "🧹 Cleaning up existing manifests..."
podman manifest rm "${IMAGE_NAME}:${IMAGE_TAG}" 2>/dev/null || true
podman manifest rm "${IMAGE_NAME}:${COMMIT_SHA}" 2>/dev/null || true
podman manifest rm "${IMAGE_NAME}:latest" 2>/dev/null || true

# Build for amd64 architecture
echo "🔨 Building for amd64..."
podman build \
    --arch=amd64 \
    --tag="rc-basic:${IMAGE_TAG}-amd64" \
    --tag="rc-basic:latest-amd64" \
    --tag="${IMAGE_NAME}:${IMAGE_TAG}-amd64" \
    --tag="${IMAGE_NAME}:latest-amd64" \
    --file=rc-web/basic-image/Dockerfile.basic \
    --label="org.opencontainers.image.version=${IMAGE_TAG}" \
    --label="org.opencontainers.image.revision=$(git rev-parse HEAD)" \
    --label="org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --label="org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc-core" \
    --label="org.opencontainers.image.title=RC Basic Image" \
    --label="org.opencontainers.image.description=Basic Alpine image for Daksha RC" \
    --label="org.opencontainers.image.platform=linux/amd64" \
    .

# Verify amd64 build
if ! podman image exists "rc-basic:${IMAGE_TAG}-amd64"; then
    echo "❌ Error: amd64 build failed"
    exit 1
fi
echo "✅ amd64 build completed"

# Build for arm64 architecture
echo "🔨 Building for arm64..."
podman build \
    --arch=arm64 \
    --tag="rc-basic:${IMAGE_TAG}-arm64" \
    --tag="rc-basic:latest-arm64" \
    --tag="${IMAGE_NAME}:${IMAGE_TAG}-arm64" \
    --tag="${IMAGE_NAME}:latest-arm64" \
    --file=rc-web/basic-image/Dockerfile.basic \
    --label="org.opencontainers.image.version=${IMAGE_TAG}" \
    --label="org.opencontainers.image.revision=$(git rev-parse HEAD)" \
    --label="org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --label="org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc-core" \
    --label="org.opencontainers.image.title=RC Basic Image" \
    --label="org.opencontainers.image.description=Basic Alpine image for Daksha RC" \
    --label="org.opencontainers.image.platform=linux/arm64" \
    .

# Verify arm64 build
if ! podman image exists "rc-basic:${IMAGE_TAG}-arm64"; then
    echo "❌ Error: arm64 build failed"
    exit 1
fi
echo "✅ arm64 build completed"

# Define manifest names (these will be multi-platform)
MANIFEST_IMAGES=(
    "${IMAGE_NAME}:${IMAGE_TAG}"
    "${IMAGE_NAME}:${COMMIT_SHA}"
    "${IMAGE_NAME}:latest"
)

# Create manifests for each final image
echo "📋 Creating multi-platform manifests..."
for manifest_image in "${MANIFEST_IMAGES[@]}"; do
    echo "📋 Creating manifest: ${manifest_image}..."

    # Create manifest
    if podman manifest create "$manifest_image"; then
        echo "✅ Created manifest: ${manifest_image}"
    else
        echo "❌ Failed to create manifest: ${manifest_image}"
        exit 1
    fi

    # Determine the corresponding platform-specific image names
    if [[ "$manifest_image" == *":${IMAGE_TAG}" ]]; then
        AMD64_IMAGE="${IMAGE_NAME}:${IMAGE_TAG}-amd64"
        ARM64_IMAGE="${IMAGE_NAME}:${IMAGE_TAG}-arm64"
    elif [[ "$manifest_image" == *":${COMMIT_SHA}" ]]; then
        AMD64_IMAGE="${IMAGE_NAME}:${IMAGE_TAG}-amd64"
        ARM64_IMAGE="${IMAGE_NAME}:${IMAGE_TAG}-arm64"
    else  # latest
        AMD64_IMAGE="${IMAGE_NAME}:latest-amd64"
        ARM64_IMAGE="${IMAGE_NAME}:latest-arm64"
    fi

    # Add amd64 image to manifest
    echo "  Adding amd64 to manifest..."
    if podman manifest add "$manifest_image" "containers-storage:${AMD64_IMAGE}"; then
        echo "  ✅ Added amd64 to manifest"
    else
        echo "  ❌ Failed to add amd64 to manifest"
        exit 1
    fi

    # Add arm64 image to manifest
    echo "  Adding arm64 to manifest..."
    if podman manifest add "$manifest_image" "containers-storage:${ARM64_IMAGE}"; then
        echo "  ✅ Added arm64 to manifest"
    else
        echo "  ❌ Failed to add arm64 to manifest"
        exit 1
    fi

    echo "✅ Successfully created manifest: ${manifest_image}"
    echo
done

# Verify manifests were created
echo "🔍 Verifying manifests..."
for tag in "${IMAGE_TAG}" "${COMMIT_SHA}" "latest"; do
    if podman manifest exists "${IMAGE_NAME}:${tag}"; then
        echo "✅ Manifest ${IMAGE_NAME}:${tag} created successfully"
    else
        echo "❌ Failed to create manifest ${IMAGE_NAME}:${tag}"
        exit 1
    fi
done

echo ""
echo "✅ Successfully built multi-platform basic image:"
echo "   📦 ${IMAGE_NAME}:${IMAGE_TAG} (multi-arch)"
echo "   📦 ${IMAGE_NAME}:${IMAGE_TAG}-amd64"
echo "   📦 ${IMAGE_NAME}:${IMAGE_TAG}-arm64"
echo "   📦 ${IMAGE_NAME}:${COMMIT_SHA} (multi-arch)"
echo "   📦 ${IMAGE_NAME}:latest (multi-arch)"
echo "   📦 ${IMAGE_NAME}:latest-amd64"
echo "   📦 ${IMAGE_NAME}:latest-arm64"

# Show image sizes
echo ""
echo "📊 Image sizes:"
podman image inspect "${IMAGE_NAME}:${IMAGE_TAG}-amd64" --format "amd64: {{.Size}}" 2>/dev/null || echo "amd64: Size unavailable"
podman image inspect "${IMAGE_NAME}:${IMAGE_TAG}-arm64" --format "arm64: {{.Size}}" 2>/dev/null || echo "arm64: Size unavailable"

echo ""
echo "🎉 Multi-platform build completed successfully!"
