#!/bin/bash

# push-basic-image.sh
# Push basic images and manifests to container registry
# Usage: ./push-basic-image.sh [TAG]
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

echo "🚀 Pushing basic images to registry..."
echo "📦 Base image: ${IMAGE_NAME}"
echo "🏷️  Tag: ${IMAGE_TAG}"
echo "📋 Commit: ${COMMIT_SHA}"

# Check if required images exist locally
echo "🔍 Verifying local images exist..."
MISSING_IMAGES=()

for arch in amd64 arm64; do
    if ! podman image exists "${IMAGE_NAME}:${IMAGE_TAG}-${arch}"; then
        MISSING_IMAGES+=("${IMAGE_NAME}:${IMAGE_TAG}-${arch}")
    fi
    if ! podman image exists "${IMAGE_NAME}:latest-${arch}"; then
        MISSING_IMAGES+=("${IMAGE_NAME}:latest-${arch}")
    fi
done

# Check if manifests exist
for tag in "${IMAGE_TAG}" "${COMMIT_SHA}" "latest"; do
    if ! podman manifest exists "${IMAGE_NAME}:${tag}"; then
        MISSING_IMAGES+=("${IMAGE_NAME}:${tag} (manifest)")
    fi
done

if [ ${#MISSING_IMAGES[@]} -gt 0 ]; then
    echo "❌ Error: The following images/manifests are missing locally:"
    for img in "${MISSING_IMAGES[@]}"; do
        echo "   - $img"
    done
    echo ""
    echo "Please run 'cargo make build-basic-image-all' first to build all required images and manifests."
    exit 1
fi

echo "✅ All required images and manifests found locally"

# Push platform-specific images
echo ""
echo "📤 Pushing platform-specific images..."

for arch in amd64 arm64; do
    echo "📤 Pushing ${arch} images..."

    # Push versioned image
    echo "   Pushing ${IMAGE_NAME}:${IMAGE_TAG}-${arch}..."
    podman push "${IMAGE_NAME}:${IMAGE_TAG}-${arch}"

    # Push latest image
    echo "   Pushing ${IMAGE_NAME}:latest-${arch}..."
    podman push "${IMAGE_NAME}:latest-${arch}"

    echo "✅ ${arch} images pushed successfully"
done

# Push multi-platform manifests
echo ""
echo "📤 Pushing multi-platform manifests..."

# Push version manifest
echo "📤 Pushing ${IMAGE_NAME}:${IMAGE_TAG} manifest..."
podman manifest push "${IMAGE_NAME}:${IMAGE_TAG}"

# Push commit SHA manifest
echo "📤 Pushing ${IMAGE_NAME}:${COMMIT_SHA} manifest..."
podman manifest push "${IMAGE_NAME}:${COMMIT_SHA}"

# Push latest manifest
echo "📤 Pushing ${IMAGE_NAME}:latest manifest..."
podman manifest push "${IMAGE_NAME}:latest"

echo ""
echo "✅ Successfully pushed all basic images and manifests:"
echo "   📦 ${IMAGE_NAME}:${IMAGE_TAG} (multi-arch)"
echo "   📦 ${IMAGE_NAME}:${IMAGE_TAG}-amd64"
echo "   📦 ${IMAGE_NAME}:${IMAGE_TAG}-arm64"
echo "   📦 ${IMAGE_NAME}:${COMMIT_SHA} (multi-arch)"
echo "   📦 ${IMAGE_NAME}:latest (multi-arch)"
echo "   📦 ${IMAGE_NAME}:latest-amd64"
echo "   📦 ${IMAGE_NAME}:latest-arm64"

echo ""
echo "🎉 Push completed successfully!"
echo ""
echo "🔗 You can now pull the images using:"
echo "   podman pull ${IMAGE_NAME}:${IMAGE_TAG}"
echo "   podman pull ${IMAGE_NAME}:latest"
