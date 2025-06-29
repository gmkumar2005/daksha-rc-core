#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🏗️  Building container image for rc-web (current platform only)...${NC}"

# Check available disk space before building
echo -e "${YELLOW}📊 Checking available disk space...${NC}"
AVAILABLE_SPACE=$(df / | awk 'NR==2 {print $4}')
AVAILABLE_GB=$((AVAILABLE_SPACE / 1024 / 1024))

if [ $AVAILABLE_GB -lt 10 ]; then
    echo -e "${RED}⚠️  Low disk space: ${AVAILABLE_GB}GB available${NC}"
    echo -e "${YELLOW}💡 Consider running 'cargo make clean-build-cache' first${NC}"
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

# Get the current platform with better error handling
echo -e "${YELLOW}🔍 Detecting current platform...${NC}"
if [ -f "scripts/get-platform.sh" ]; then
    if CURRENT_PLATFORM=$(bash scripts/get-platform.sh 2>&1); then
        echo -e "${GREEN}✅ Platform detected: ${CURRENT_PLATFORM}${NC}"
    else
        echo -e "${RED}❌ Platform detection script failed: ${CURRENT_PLATFORM}${NC}"
        echo -e "${YELLOW}💡 Falling back to manual detection...${NC}"
        # Manual platform detection as fallback
        case "$(uname -m)" in
            x86_64|amd64)
                CURRENT_PLATFORM="amd64"
                ;;
            aarch64|arm64)
                CURRENT_PLATFORM="arm64"
                ;;
            *)
                echo -e "${RED}❌ Unsupported architecture: $(uname -m)${NC}"
                exit 1
                ;;
        esac
        echo -e "${GREEN}✅ Manual detection successful: ${CURRENT_PLATFORM}${NC}"
    fi
else
    echo -e "${RED}❌ Platform detection script not found: scripts/get-platform.sh${NC}"
    exit 1
fi
echo -e "${GREEN}🏗️  Building for current platform: ${CURRENT_PLATFORM}${NC}"

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

# Get commit SHA for additional tagging and traceability
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
echo -e "${GREEN}📋 Git commit: ${GIT_SHA}${NC}"

# Validate Dockerfile exists before attempting build
if [ ! -f "rc-web/Dockerfile" ]; then
    echo -e "${RED}❌ Dockerfile not found at rc-web/Dockerfile${NC}"
    exit 1
fi

# Define image base name
IMAGE_BASE="ghcr.io/daksha-rc/rc-web"

# Define final image names for current platform
FINAL_IMAGES=(
    "${IMAGE_BASE}:${GIT_TAG}"
    "${IMAGE_BASE}:${GIT_TAG}-${CURRENT_PLATFORM}"
    "${IMAGE_BASE}:${GIT_SHA}-${CURRENT_PLATFORM}"
    "${IMAGE_BASE}:latest-${CURRENT_PLATFORM}"
    "${IMAGE_BASE}:latest"
)

echo -e "${YELLOW}🔨 Building image for ${CURRENT_PLATFORM}${NC}"
echo -e "${YELLOW}📋 Target images:${NC}"
for image in "${FINAL_IMAGES[@]}"; do
    echo -e "  - ${image}"
done

# Clean up any existing images with these names (but preserve base-builder stages)
echo -e "${YELLOW}🧹 Cleaning up existing images (preserving base-builder stages)...${NC}"
for image in "${FINAL_IMAGES[@]}"; do
    # Don't remove images that might be base-builder stages
    if [[ "$image" != *"base"* && "$image" != *"builder"* ]]; then
        podman rmi "$image" 2>/dev/null || true
    fi
done

# Build for current platform
echo -e "${GREEN}🦭 Building for ${CURRENT_PLATFORM}...${NC}"

# Build the image with the first tag
PRIMARY_IMAGE="${FINAL_IMAGES[0]}"
echo -e "${YELLOW}🔨 Building primary image: ${PRIMARY_IMAGE}${NC}"

# Set SQLX_OFFLINE for build
export SQLX_OFFLINE=true

if podman build \
    --platform "linux/${CURRENT_PLATFORM}" \
    -f "rc-web/Dockerfile" \
    -t "$PRIMARY_IMAGE" \
    --label "org.opencontainers.image.version=${GIT_TAG}" \
    --label "org.opencontainers.image.revision=${GIT_SHA}" \
    --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --label "org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc" \
    --label "org.opencontainers.image.description=Daksha RC Web Application" \
    --label "org.opencontainers.image.platform=${CURRENT_PLATFORM}" \
    --squash-all \
    --rm \
    .; then
    echo -e "${GREEN}✅ Successfully built ${CURRENT_PLATFORM} image${NC}"

    # Verify the image was created
    if podman image exists "$PRIMARY_IMAGE"; then
        echo -e "${GREEN}✅ Primary image verified: ${PRIMARY_IMAGE}${NC}"
    else
        echo -e "${RED}❌ Primary image not found after build: ${PRIMARY_IMAGE}${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ Failed to build ${CURRENT_PLATFORM} image${NC}"
    echo -e "${YELLOW}💡 Check the build logs above for specific errors${NC}"
    echo -e "${YELLOW}💡 Common issues:${NC}"
    echo -e "${YELLOW}   - Insufficient disk space${NC}"
    echo -e "${YELLOW}   - Missing dependencies in Dockerfile${NC}"
    echo -e "${YELLOW}   - Network connectivity issues${NC}"
    echo -e "${YELLOW}   - Ring crate compilation issues (for cross-platform builds)${NC}"
    exit 1
fi

# Tag the image with all other names
for ((i=1; i<${#FINAL_IMAGES[@]}; i++)); do
    echo -e "${YELLOW}🏷️  Tagging as ${FINAL_IMAGES[i]}...${NC}"
    if podman tag "$PRIMARY_IMAGE" "${FINAL_IMAGES[i]}"; then
        echo -e "${GREEN}✅ Tagged as ${FINAL_IMAGES[i]}${NC}"
    else
        echo -e "${RED}❌ Failed to tag as ${FINAL_IMAGES[i]}${NC}"
        exit 1
    fi
done

# Clean up intermediate layers and build cache
echo -e "${YELLOW}🧹 Cleaning up intermediate build artifacts...${NC}"
podman image prune -f

echo -e "${GREEN}✅ Successfully built container images:${NC}"
for image in "${FINAL_IMAGES[@]}"; do
    echo -e "  ${GREEN}✓${NC} ${image}"
done

# Show final image details
echo -e "${YELLOW}📊 Image information:${NC}"
echo -e "${YELLOW}Platform: ${CURRENT_PLATFORM}${NC}"
echo -e "${YELLOW}Base image: ${PRIMARY_IMAGE}${NC}"

# Show image size and details
if IMAGE_SIZE=$(podman inspect "$PRIMARY_IMAGE" --format '{{.Size}}' 2>/dev/null); then
    if command -v numfmt >/dev/null 2>&1; then
        IMAGE_SIZE_HUMAN=$(echo "$IMAGE_SIZE" | numfmt --to=iec 2>/dev/null || echo "$IMAGE_SIZE bytes")
    else
        IMAGE_SIZE_HUMAN="$IMAGE_SIZE bytes"
    fi
    echo -e "${YELLOW}Size: ${IMAGE_SIZE_HUMAN}${NC}"
else
    echo -e "${YELLOW}Size: Unable to determine${NC}"
fi

# Show all created images
echo -e "${YELLOW}📋 All created images:${NC}"
for image in "${FINAL_IMAGES[@]}"; do
    if podman image exists "$image"; then
        echo -e "${GREEN}   ✅ ${image}${NC}"
    else
        echo -e "${RED}   ❌ ${image}${NC}"
    fi
done

echo -e "${GREEN}🎉 Single-platform build completed successfully!${NC}"
echo -e "${BLUE}💡 Built for current platform: ${CURRENT_PLATFORM}${NC}"
echo -e "${BLUE}💡 Use 'cargo make build-image-all' to build for multiple platforms${NC}"
