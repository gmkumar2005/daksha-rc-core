#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🚀 Pushing multi-platform container manifests for rc-web...${NC}"

# Get the latest git tag with fallbacks (must match build-image logic)
if GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null); then
    echo -e "${GREEN}📋 Using Git tag: ${GIT_TAG}${NC}"
else
    GIT_TAG="v0.0.0-dev"
    echo -e "${YELLOW}⚠️  No Git tags found, using default: ${GIT_TAG}${NC}"
fi

# Get commit SHA for consistency with build-image
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Define the manifests that should have been built by build-image task
MANIFESTS=(
    "ghcr.io/daksha-rc/rc-web:${GIT_TAG}"
    "ghcr.io/daksha-rc/rc-web:${GIT_SHA}"
    "ghcr.io/daksha-rc/rc-web:latest"
)

# Check if podman is available
if ! command -v podman >/dev/null 2>&1; then
    echo -e "${RED}❌ Podman not found. Please install podman${NC}"
    exit 1
fi

echo -e "${GREEN}🦭 Using Podman container engine${NC}"

# Validate all required manifests exist locally before attempting to push
echo -e "${YELLOW}🔍 Checking local manifests...${NC}"
for manifest in "${MANIFESTS[@]}"; do
    if podman manifest inspect "$manifest" >/dev/null 2>&1; then
        echo -e "  ${GREEN}✓${NC} $manifest"
        # Show platforms for verification
        platforms=$(podman manifest inspect "$manifest" --format json | jq -r '.manifests[] | "\(.platform.os)/\(.platform.architecture)"' 2>/dev/null | tr '\n' ', ' | sed 's/,$//')
        if [ -n "$platforms" ]; then
            echo -e "    Platforms: $platforms"
        fi
    else
        echo -e "  ${RED}✗${NC} $manifest (not found locally)"
        echo -e "${RED}❌ Please run 'cargo make build-image' first${NC}"
        exit 1
    fi
done

# Push each manifest to the registry with individual error handling
echo -e "${YELLOW}📤 Pushing manifests:${NC}"
for manifest in "${MANIFESTS[@]}"; do
    echo -e "${YELLOW}  Pushing $manifest...${NC}"
    if podman manifest push "$manifest"; then
        echo -e "  ${GREEN}✓${NC} Successfully pushed $manifest"
    else
        echo -e "  ${RED}✗${NC} Failed to push $manifest"
        exit 1
    fi
done

echo -e "${GREEN}✅ All manifests pushed successfully!${NC}"
echo -e "${YELLOW}📋 Pushed manifests:${NC}"
for manifest in "${MANIFESTS[@]}"; do
    echo -e "  ${GREEN}✓${NC} $manifest (multi-platform: linux/amd64, linux/arm64)"
done

echo -e "${GREEN}🎉 Multi-platform push completed successfully!${NC}"
