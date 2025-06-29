#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🏗️  Building base-builder stage for rc-web...${NC}"

# Check if podman is available
if ! command -v podman >/dev/null 2>&1; then
    echo -e "${RED}❌ Podman not found. Please install podman${NC}"
    exit 1
fi

echo -e "${GREEN}🦭 Using Podman container engine${NC}"

# Get the current platform
CURRENT_PLATFORM=$(bash scripts/get-platform.sh)
if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Failed to detect current platform${NC}"
    exit 1
fi
echo -e "${GREEN}🏗️  Building base-builder stage for platform: ${CURRENT_PLATFORM}${NC}"

# Validate Dockerfile exists before attempting build
if [ ! -f "rc-web/Dockerfile" ]; then
    echo -e "${RED}❌ Dockerfile not found at rc-web/Dockerfile${NC}"
    exit 1
fi

# Define base-builder image name
BASE_BUILDER_IMAGE="ghcr.io/daksha-rc/rc-web:base-builder-${CURRENT_PLATFORM}"

echo -e "${YELLOW}📋 Target base-builder image: ${BASE_BUILDER_IMAGE}${NC}"

# Check if base-builder stage already exists
if podman inspect "$BASE_BUILDER_IMAGE" >/dev/null 2>&1; then
    echo -e "${BLUE}ℹ️  Base-builder stage already exists: $BASE_BUILDER_IMAGE${NC}"
    read -p "Rebuild base-builder stage? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${GREEN}✅ Using existing base-builder stage${NC}"
        exit 0
    fi
fi

# Build the base-builder stage only
echo -e "${GREEN}🦭 Building base-builder stage for ${CURRENT_PLATFORM}...${NC}"

if podman build \
    --arch "$CURRENT_PLATFORM" \
    -f "rc-web/Dockerfile" \
    --target base-builder \
    -t "$BASE_BUILDER_IMAGE" \
    --label "org.opencontainers.image.title=RC Web Base Builder Stage" \
    --label "org.opencontainers.image.description=Base builder stage with build dependencies for Daksha RC Web Application" \
    --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --label "org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc" \
    --label "org.opencontainers.image.platform=${CURRENT_PLATFORM}" \
    --label "org.opencontainers.image.stage=base-builder" \
    --rm \
    .; then
    echo -e "${GREEN}✅ Successfully built base-builder stage for ${CURRENT_PLATFORM}${NC}"
else
    echo -e "${RED}❌ Failed to build base-builder stage for ${CURRENT_PLATFORM}${NC}"
    exit 1
fi

# Show final image details
echo -e "${YELLOW}📊 Base-builder stage information:${NC}"
echo -e "${YELLOW}Platform: ${CURRENT_PLATFORM}${NC}"
echo -e "${YELLOW}Image: ${BASE_BUILDER_IMAGE}${NC}"

# Show image size
IMAGE_SIZE=$(podman inspect "$BASE_BUILDER_IMAGE" --format '{{.Size}}' 2>/dev/null | numfmt --to=iec 2>/dev/null || echo "unknown")
echo -e "${YELLOW}Size: ${IMAGE_SIZE}${NC}"

echo -e "${GREEN}🎉 Base-builder stage completed successfully!${NC}"
echo -e "${BLUE}💡 This stage will be reused automatically in subsequent application builds${NC}"
echo -e "${BLUE}💡 Use 'cargo make build-image' to build the full application${NC}"
