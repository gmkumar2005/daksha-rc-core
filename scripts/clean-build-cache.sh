#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🧹 Cleaning container build cache...${NC}"

# Check if podman is available
if ! command -v podman >/dev/null 2>&1; then
    echo -e "${RED}❌ Podman not found. Please install podman${NC}"
    exit 1
fi

echo -e "${GREEN}🦭 Using Podman${NC}"

# Show current disk usage
echo -e "${YELLOW}📊 Current container storage usage:${NC}"
podman system df

# Clean up unused containers, images, volumes, and manifests
echo -e "${YELLOW}🗑️  Cleaning unused containers...${NC}"
podman container prune -f

# Check for base-builder and intermediate images that should be preserved
echo -e "${BLUE}🔍 Checking for base-builder and intermediate images to preserve...${NC}"
INTERMEDIATE_IMAGES=$(podman images --format "{{.Repository}}:{{.Tag}}" | grep -E "(base-builder|planner|cacher|builder)" || true)
if [ -n "$INTERMEDIATE_IMAGES" ]; then
    echo -e "${GREEN}📦 Preserving intermediate build images:${NC}"
    echo "$INTERMEDIATE_IMAGES" | while read -r image; do
        if [ -n "$image" ]; then
            echo -e "  ${GREEN}✓${NC} $image"
        fi
    done
else
    echo -e "${YELLOW}ℹ️  No intermediate build images found to preserve${NC}"
fi

echo -e "${YELLOW}🗑️  Cleaning unused images (preserving intermediate build stages)...${NC}"
# Use regular prune without -a to preserve tagged images and intermediate stages
podman image prune -f

echo -e "${YELLOW}🗑️  Cleaning unused volumes...${NC}"
podman volume prune -f

echo -e "${YELLOW}🗑️  Cleaning unused networks...${NC}"
podman network prune -f

# Clean up build cache and temporary files
echo -e "${YELLOW}🗑️  Cleaning build cache...${NC}"
podman system prune -af

# Clean up any orphaned manifests (if any exist)
echo -e "${YELLOW}🗑️  Cleaning unused manifests...${NC}"
# List all manifests and remove any that might be dangling
podman manifest ls --format "{{.Name}}" 2>/dev/null | while read -r manifest; do
    if [ -n "$manifest" ]; then
        # Check if manifest has any associated images, if not it might be dangling
        if ! podman manifest inspect "$manifest" >/dev/null 2>&1; then
            echo -e "  Removing dangling manifest: $manifest"
            podman manifest rm "$manifest" 2>/dev/null || true
        fi
    fi
done

echo -e "${GREEN}✅ Cleanup complete!${NC}"
echo -e "${YELLOW}📊 Storage usage after cleanup:${NC}"
podman system df

echo -e "${GREEN}🎉 Container build cache cleaned successfully!${NC}"
