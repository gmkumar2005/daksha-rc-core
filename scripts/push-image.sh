#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🚀 Pushing Docker images for rc-web...${NC}"

# Get the latest git tag with fallbacks (must match build-image logic)
if GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null); then
    echo -e "${GREEN}📋 Using Git tag: ${GIT_TAG}${NC}"
else
    GIT_TAG="v0.0.0-dev"
    echo -e "${YELLOW}⚠️  No Git tags found, using default: ${GIT_TAG}${NC}"
fi

# Get commit SHA for consistency with build-image
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Define the images that should have been built by build-image task
IMAGES=(
    "ghcr.io/daksha-rc/rc-web:${GIT_TAG}"
    "ghcr.io/daksha-rc/rc-web:${GIT_SHA}"
    "ghcr.io/daksha-rc/rc-web:latest"
)

# Detect container engine for push operations
if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    if docker version | grep -q "podman\|Podman"; then
        CONTAINER_ENGINE="podman"
    else
        CONTAINER_ENGINE="docker"
    fi
elif command -v podman >/dev/null 2>&1; then
    CONTAINER_ENGINE="podman"
else
    echo -e "${RED}❌ No container engine found (docker or podman)${NC}"
    exit 1
fi

# Validate all required images exist locally before attempting to push
echo -e "${YELLOW}🔍 Checking local images...${NC}"
for img in "${IMAGES[@]}"; do
    if [ "$CONTAINER_ENGINE" = "podman" ]; then
        if podman image inspect "$img" >/dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $img"
        else
            echo -e "  ${RED}✗${NC} $img (not found locally)"
            echo -e "${RED}❌ Please run 'cargo make build-image' first${NC}"
            exit 1
        fi
    else
        if docker image inspect "$img" >/dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} $img"
        else
            echo -e "  ${RED}✗${NC} $img (not found locally)"
            echo -e "${RED}❌ Please run 'cargo make build-image' first${NC}"
            exit 1
        fi
    fi
done

# Push each image to the registry with individual error handling
echo -e "${YELLOW}📤 Pushing images:${NC}"
for img in "${IMAGES[@]}"; do
    echo -e "${YELLOW}  Pushing $img...${NC}"
    if [ "$CONTAINER_ENGINE" = "podman" ]; then
        if podman push "$img"; then
            echo -e "  ${GREEN}✓${NC} Successfully pushed $img"
        else
            echo -e "  ${RED}✗${NC} Failed to push $img"
            exit 1
        fi
    else
        if docker push "$img"; then
            echo -e "  ${GREEN}✓${NC} Successfully pushed $img"
        else
            echo -e "  ${RED}✗${NC} Failed to push $img"
            exit 1
        fi
    fi
done

echo -e "${GREEN}✅ All images pushed successfully!${NC}"
echo -e "${YELLOW}📋 Pushed images:${NC}"
for img in "${IMAGES[@]}"; do
    echo -e "  ${GREEN}✓${NC} $img"
done