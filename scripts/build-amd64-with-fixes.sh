#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🏗️  Building AMD64 image with ring crate fixes...${NC}"

# Check if we're on ARM64 (where emulation is needed)
HOST_ARCH=$(uname -m)
if [[ "$HOST_ARCH" != "arm64" && "$HOST_ARCH" != "aarch64" ]]; then
    echo -e "${YELLOW}⚠️  This script is optimized for ARM64 hosts cross-compiling to AMD64${NC}"
    echo -e "${YELLOW}⚠️  You may want to use the regular build script instead${NC}"
fi

# Check available disk space
echo -e "${YELLOW}📊 Checking available disk space...${NC}"
AVAILABLE_SPACE=$(df / | awk 'NR==2 {print $4}')
AVAILABLE_GB=$((AVAILABLE_SPACE / 1024 / 1024))

if [ $AVAILABLE_GB -lt 15 ]; then
    echo -e "${RED}⚠️  Low disk space: ${AVAILABLE_GB}GB available${NC}"
    echo -e "${YELLOW}💡 AMD64 emulation builds require more space${NC}"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    echo -e "${GREEN}✅ Sufficient disk space: ${AVAILABLE_GB}GB available${NC}"
fi

# Check if podman is available
if ! command -v podman >/dev/null 2>&1; then
    echo -e "${RED}❌ Podman not found. Please install podman${NC}"
    exit 1
fi

echo -e "${GREEN}🦭 Using Podman container engine${NC}"

# Setup QEMU emulation
echo -e "${YELLOW}🔧 Setting up QEMU emulation for AMD64...${NC}"
if ! podman run --rm --privileged multiarch/qemu-user-static --reset -p yes; then
    echo -e "${RED}❌ Failed to setup QEMU emulation${NC}"
    exit 1
fi

# Wait for QEMU to initialize
sleep 2

# Test QEMU emulation
echo -e "${YELLOW}🧪 Testing QEMU emulation...${NC}"
if timeout 30s podman run --rm --platform linux/amd64 alpine:latest uname -m; then
    echo -e "${GREEN}✅ QEMU emulation working correctly${NC}"
else
    echo -e "${YELLOW}⚠️  QEMU test failed, but continuing...${NC}"
fi

# Get tag from parameter or fallback to git tag
if [ -n "$TAG" ]; then
    GIT_TAG="$TAG"
    echo -e "${GREEN}📋 Using provided tag: ${GIT_TAG}${NC}"
elif GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null); then
    echo -e "${GREEN}📋 Using Git tag: ${GIT_TAG}${NC}"
else
    GIT_TAG="v0.0.0-dev"
    echo -e "${YELLOW}⚠️  No Git tags found, using default: ${GIT_TAG}${NC}"
fi

# Get commit SHA
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
echo -e "${GREEN}📋 Git commit: ${GIT_SHA}${NC}"

# Validate Dockerfile exists
if [ ! -f "rc-web/Dockerfile" ]; then
    echo -e "${RED}❌ Dockerfile not found at rc-web/Dockerfile${NC}"
    exit 1
fi

# Define image names
IMAGE_BASE="ghcr.io/daksha-rc/rc-web"
PRIMARY_IMAGE="${IMAGE_BASE}:${GIT_TAG}-amd64"

echo -e "${YELLOW}🔨 Building AMD64 image with ring fixes...${NC}"
echo -e "${YELLOW}📋 Target image: ${PRIMARY_IMAGE}${NC}"

# Set environment variables for ring crate fixes
export SQLX_OFFLINE=true
export RING_DISABLE_ASSEMBLY=1
export RING_PREGENERATE_ASM=0

# Build timeout and memory settings for emulation
BUILD_TIMEOUT="${BUILD_TIMEOUT:-7200}"  # 2 hours default
MEMORY_LIMIT="${MEMORY_LIMIT:-6g}"      # 6GB memory limit

echo -e "${YELLOW}⚙️  Build settings:${NC}"
echo -e "${BLUE}   Timeout: ${BUILD_TIMEOUT} seconds${NC}"
echo -e "${BLUE}   Memory limit: ${MEMORY_LIMIT}${NC}"
echo -e "${BLUE}   Ring assembly disabled: true${NC}"
echo -e "${BLUE}   SQLX offline: true${NC}"

# Clean up any existing AMD64 images
echo -e "${YELLOW}🧹 Cleaning up existing AMD64 images...${NC}"
podman rmi "${PRIMARY_IMAGE}" 2>/dev/null || true
podman rmi "${IMAGE_BASE}:${GIT_SHA}-amd64" 2>/dev/null || true
podman rmi "${IMAGE_BASE}:latest-amd64" 2>/dev/null || true

# Build the AMD64 image with ring fixes
echo -e "${GREEN}🦭 Building AMD64 image with emulation and ring fixes...${NC}"

if timeout "$BUILD_TIMEOUT" podman build \
    --platform linux/amd64 \
    --memory="$MEMORY_LIMIT" \
    --build-arg RING_DISABLE_ASSEMBLY=1 \
    --build-arg RING_PREGENERATE_ASM=0 \
    --build-arg RUSTFLAGS="-C opt-level=1 -C debuginfo=0" \
    --build-arg CARGO_BUILD_JOBS=1 \
    --build-arg CARGO_PROFILE_RELEASE_DEBUG=false \
    --build-arg CARGO_PROFILE_RELEASE_OPT_LEVEL=1 \
    -f "rc-web/Dockerfile" \
    -t "$PRIMARY_IMAGE" \
    --label "org.opencontainers.image.version=${GIT_TAG}" \
    --label "org.opencontainers.image.revision=${GIT_SHA}" \
    --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --label "org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc" \
    --label "org.opencontainers.image.description=Daksha RC Web Application (AMD64)" \
    --label "org.opencontainers.image.platform=amd64" \
    --squash-all \
    --rm \
    .; then
    echo -e "${GREEN}✅ Successfully built AMD64 image${NC}"
else
    echo -e "${RED}❌ Failed to build AMD64 image${NC}"
    echo -e "${YELLOW}💡 Common causes:${NC}"
    echo -e "${YELLOW}   - Ring crate compilation issues in QEMU${NC}"
    echo -e "${YELLOW}   - Insufficient memory or timeout${NC}"
    echo -e "${YELLOW}   - Network connectivity issues${NC}"
    echo -e "${YELLOW}💡 Try increasing BUILD_TIMEOUT or MEMORY_LIMIT:${NC}"
    echo -e "${BLUE}   BUILD_TIMEOUT=10800 MEMORY_LIMIT=8g $0${NC}"
    exit 1
fi

# Verify the image was created
if podman image exists "$PRIMARY_IMAGE"; then
    echo -e "${GREEN}✅ AMD64 image verified: ${PRIMARY_IMAGE}${NC}"
else
    echo -e "${RED}❌ AMD64 image not found after build${NC}"
    exit 1
fi

# Tag with additional variants
echo -e "${YELLOW}🏷️  Creating additional AMD64 tags...${NC}"
podman tag "$PRIMARY_IMAGE" "${IMAGE_BASE}:${GIT_SHA}-amd64"
podman tag "$PRIMARY_IMAGE" "${IMAGE_BASE}:latest-amd64"

# Show image details
echo -e "${YELLOW}📊 AMD64 image information:${NC}"
if IMAGE_SIZE=$(podman inspect "$PRIMARY_IMAGE" --format '{{.Size}}' 2>/dev/null); then
    if command -v numfmt >/dev/null 2>&1; then
        IMAGE_SIZE_HUMAN=$(echo "$IMAGE_SIZE" | numfmt --to=iec 2>/dev/null || echo "$IMAGE_SIZE bytes")
    else
        IMAGE_SIZE_HUMAN="$IMAGE_SIZE bytes"
    fi
    echo -e "${BLUE}   Size: ${IMAGE_SIZE_HUMAN}${NC}"
else
    echo -e "${YELLOW}   Size: Unable to determine${NC}"
fi

# Test the image
echo -e "${YELLOW}🧪 Testing AMD64 image...${NC}"
if timeout 60s podman run --rm "$PRIMARY_IMAGE" --help >/dev/null 2>&1; then
    echo -e "${GREEN}✅ AMD64 image test passed${NC}"
else
    echo -e "${YELLOW}⚠️  Image test failed or timed out (this may be expected)${NC}"
fi

# Clean up build artifacts
echo -e "${YELLOW}🧹 Cleaning up build artifacts...${NC}"
podman image prune -f

echo -e "${GREEN}✅ AMD64 build completed successfully!${NC}"
echo -e "${GREEN}📦 Created images:${NC}"
echo -e "${GREEN}   ✓ ${PRIMARY_IMAGE}${NC}"
echo -e "${GREEN}   ✓ ${IMAGE_BASE}:${GIT_SHA}-amd64${NC}"
echo -e "${GREEN}   ✓ ${IMAGE_BASE}:latest-amd64${NC}"
echo
echo -e "${BLUE}💡 Next steps:${NC}"
echo -e "${BLUE}   1. Test the AMD64 image: podman run --rm ${PRIMARY_IMAGE} --help${NC}"
echo -e "${BLUE}   2. Combine with ARM64 to create multi-platform manifest${NC}"
echo -e "${BLUE}   3. Push to registry: podman push ${PRIMARY_IMAGE}${NC}"
