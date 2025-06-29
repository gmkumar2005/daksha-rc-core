#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🚀 Pushing container images and manifests for rc-web...${NC}"

# Get tag from parameter or fallback to git tag (must match build-image logic)
if [ -n "$TAG" ]; then
    GIT_TAG="$TAG"
    echo -e "${GREEN}📋 Using provided tag: ${GIT_TAG}${NC}"
elif GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null); then
    echo -e "${GREEN}📋 Using Git tag: ${GIT_TAG}${NC}"
else
    GIT_TAG="v0.0.0-dev"
    echo -e "${YELLOW}⚠️  No Git tags found, using default: ${GIT_TAG}${NC}"
fi

# Get commit SHA for consistency with build-image
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Check if podman is available
if ! command -v podman >/dev/null 2>&1; then
    echo -e "${RED}❌ Podman not found. Please install podman${NC}"
    exit 1
fi

echo -e "${GREEN}🦭 Using Podman container engine${NC}"

# Define image base name
IMAGE_BASE="ghcr.io/daksha-rc/rc-web"

# Define all possible images that could exist
PLATFORM_IMAGES=(
    "${IMAGE_BASE}:${GIT_TAG}-amd64"
    "${IMAGE_BASE}:${GIT_TAG}-arm64"
    "${IMAGE_BASE}:${GIT_SHA}-amd64"
    "${IMAGE_BASE}:${GIT_SHA}-arm64"
    "${IMAGE_BASE}:latest-amd64"
    "${IMAGE_BASE}:latest-arm64"
)

MANIFEST_IMAGES=(
    "${IMAGE_BASE}:${GIT_TAG}"
    "${IMAGE_BASE}:${GIT_SHA}"
    "${IMAGE_BASE}:latest"
)

# Arrays to store what actually exists locally
EXISTING_PLATFORM_IMAGES=()
EXISTING_MANIFEST_IMAGES=()

# Check what platform-specific images exist locally
echo -e "${YELLOW}🔍 Checking local platform-specific images...${NC}"
for image in "${PLATFORM_IMAGES[@]}"; do
    if podman inspect "$image" >/dev/null 2>&1; then
        echo -e "  ${GREEN}✓${NC} $image"
        EXISTING_PLATFORM_IMAGES+=("$image")
    else
        echo -e "  ${BLUE}○${NC} $image (not found)"
    fi
done

# Check what manifests exist locally
echo -e "${YELLOW}🔍 Checking local manifests...${NC}"
for image in "${MANIFEST_IMAGES[@]}"; do
    if podman manifest inspect "$image" >/dev/null 2>&1; then
        echo -e "  ${GREEN}✓${NC} $image (manifest)"
        # Show platforms for verification
        platforms=$(podman manifest inspect "$image" --format json 2>/dev/null | jq -r '.manifests[]? | "\(.platform.os)/\(.platform.architecture)"' 2>/dev/null | tr '\n' ', ' | sed 's/,$//' || echo "unknown")
        if [ -n "$platforms" ] && [ "$platforms" != "unknown" ]; then
            echo -e "    Platforms: $platforms"
        fi
        EXISTING_MANIFEST_IMAGES+=("$image")
    elif podman inspect "$image" >/dev/null 2>&1; then
        echo -e "  ${GREEN}✓${NC} $image (single image)"
        EXISTING_MANIFEST_IMAGES+=("$image")
    else
        echo -e "  ${BLUE}○${NC} $image (not found)"
    fi
done

# Check if we have anything to push
TOTAL_IMAGES=$((${#EXISTING_PLATFORM_IMAGES[@]} + ${#EXISTING_MANIFEST_IMAGES[@]}))
if [ $TOTAL_IMAGES -eq 0 ]; then
    echo -e "${RED}❌ No images found locally to push${NC}"
    echo -e "${YELLOW}💡 Please run 'cargo make build-image' or 'cargo make build-image-all' first${NC}"
    exit 1
fi

echo -e "${GREEN}📊 Found ${TOTAL_IMAGES} images/manifests to push${NC}"

# Push platform-specific images
if [ ${#EXISTING_PLATFORM_IMAGES[@]} -gt 0 ]; then
    echo -e "${YELLOW}📤 Pushing platform-specific images:${NC}"
    for image in "${EXISTING_PLATFORM_IMAGES[@]}"; do
        echo -e "${YELLOW}  Pushing $image...${NC}"
        if podman push "$image"; then
            echo -e "  ${GREEN}✓${NC} Successfully pushed $image"
        else
            echo -e "  ${RED}✗${NC} Failed to push $image"
            exit 1
        fi
    done
fi

# Push manifests and single images
if [ ${#EXISTING_MANIFEST_IMAGES[@]} -gt 0 ]; then
    echo -e "${YELLOW}📤 Pushing manifests and images:${NC}"
    for image in "${EXISTING_MANIFEST_IMAGES[@]}"; do
        echo -e "${YELLOW}  Pushing $image...${NC}"

        # Check if it's a manifest or single image
        if podman manifest inspect "$image" >/dev/null 2>&1; then
            # It's a manifest
            if podman manifest push "$image"; then
                echo -e "  ${GREEN}✓${NC} Successfully pushed manifest $image"
            else
                echo -e "  ${RED}✗${NC} Failed to push manifest $image"
                exit 1
            fi
        else
            # It's a single image
            if podman push "$image"; then
                echo -e "  ${GREEN}✓${NC} Successfully pushed image $image"
            else
                echo -e "  ${RED}✗${NC} Failed to push image $image"
                exit 1
            fi
        fi
    done
fi

echo -e "${GREEN}✅ All images and manifests pushed successfully!${NC}"

# Summary
echo -e "${YELLOW}📋 Push summary:${NC}"
if [ ${#EXISTING_PLATFORM_IMAGES[@]} -gt 0 ]; then
    echo -e "${GREEN}Platform-specific images pushed:${NC}"
    for image in "${EXISTING_PLATFORM_IMAGES[@]}"; do
        echo -e "  ${GREEN}✓${NC} $image"
    done
fi

if [ ${#EXISTING_MANIFEST_IMAGES[@]} -gt 0 ]; then
    echo -e "${GREEN}Manifests/images pushed:${NC}"
    for image in "${EXISTING_MANIFEST_IMAGES[@]}"; do
        if podman manifest inspect "$image" >/dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $image (multi-platform manifest)"
        else
            echo -e "  ${GREEN}✓${NC} $image (single platform)"
        fi
    done
fi

echo -e "${GREEN}🎉 Push completed successfully!${NC}"
echo -e "${BLUE}💡 Images are now available in the registry${NC}"
