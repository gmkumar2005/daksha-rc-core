#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🧹 Cleaning container build cache...${NC}"

# Detect container engine
if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    if docker version | grep -q "podman\|Podman"; then
        CONTAINER_ENGINE="podman"
    else
        CONTAINER_ENGINE="docker"
    fi
elif command -v podman >/dev/null 2>&1; then
    CONTAINER_ENGINE="podman"
else
    echo -e "${RED}❌ No container engine found${NC}"
    exit 1
fi

echo -e "${GREEN}Using $CONTAINER_ENGINE${NC}"

# Show current disk usage
echo -e "${YELLOW}📊 Current container storage usage:${NC}"
$CONTAINER_ENGINE system df

# Clean up unused containers, images, and volumes
echo -e "${YELLOW}🗑️  Cleaning unused containers...${NC}"
$CONTAINER_ENGINE container prune -f

echo -e "${YELLOW}🗑️  Cleaning unused images...${NC}"
$CONTAINER_ENGINE image prune -af

echo -e "${YELLOW}🗑️  Cleaning unused volumes...${NC}"
$CONTAINER_ENGINE volume prune -f

echo -e "${GREEN}✅ Cleanup complete!${NC}"
echo -e "${YELLOW}📊 Storage usage after cleanup:${NC}"
$CONTAINER_ENGINE system df