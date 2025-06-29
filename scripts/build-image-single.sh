#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🏗️  Building single-platform container image for rc-web...${NC}"

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

# Get the latest git tag with fallbacks
if GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null); then
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

# Get platform from environment variable or default to current architecture
if [ -n "$BUILD_PLATFORM" ]; then
    PLATFORM="$BUILD_PLATFORM"
elif [ -n "$BUILD_PLATFORMS" ]; then
    # Take first platform from BUILD_PLATFORMS if set
    IFS=',' read -ra PLATFORM_ARRAY <<< "$BUILD_PLATFORMS"
    PLATFORM="${PLATFORM_ARRAY[0]}"
else
    # Default to current architecture
    if [[ $(uname -m) == "arm64" || $(uname -m) == "aarch64" ]]; then
        PLATFORM="linux/arm64"
    else
        PLATFORM="linux/amd64"
    fi
fi

echo -e "${BLUE}🔨 Building for platform: ${PLATFORM}${NC}"

# Define final image names
FINAL_IMAGES=(
    "ghcr.io/daksha-rc/rc-web:${GIT_TAG}"
    "ghcr.io/daksha-rc/rc-web:${GIT_SHA}"
    "ghcr.io/daksha-rc/rc-web:latest"
)

echo -e "${YELLOW}📋 Target images:${NC}"
for image in "${FINAL_IMAGES[@]}"; do
    echo -e "  - ${image}"
done

# Clean up any existing images and manifests with these names
echo -e "${YELLOW}🧹 Cleaning up existing images and manifests...${NC}"
for image in "${FINAL_IMAGES[@]}"; do
    # Try to remove as manifest first, then as regular image
    podman manifest rm "$image" 2>/dev/null || true
    podman rmi "$image" 2>/dev/null || true
done

# Build the image for the specified platform
echo -e "${GREEN}🦭 Building image for platform: ${PLATFORM}${NC}"

# Create temporary image name
platform_safe=$(echo "$PLATFORM" | tr '/' '-')
temp_image="localhost/rc-web-single-temp:${platform_safe}-$(date +%s)"

echo -e "${YELLOW}  Building ${PLATFORM} image...${NC}"

if podman build \
    --platform "${PLATFORM}" \
    -f "rc-web/Dockerfile" \
    -t "${temp_image}" \
    --label "org.opencontainers.image.version=${GIT_TAG}" \
    --label "org.opencontainers.image.revision=${GIT_SHA}" \
    --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --label "org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc" \
    --label "org.opencontainers.image.description=Daksha RC Web Application" \
    --label "org.opencontainers.image.platform=${PLATFORM}" \
    --squash-all \
    --rm \
    .; then

    # Verify the image was actually created
    if podman inspect "$temp_image" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ Successfully built ${PLATFORM} image: ${temp_image}${NC}"

        # Tag the image with final names (no manifest creation for single platform)
        echo -e "${GREEN}📋 Creating final image tags...${NC}"
        for final_image in "${FINAL_IMAGES[@]}"; do
            echo -e "${YELLOW}Tagging: ${final_image}${NC}"
            podman tag "$temp_image" "$final_image"
            echo -e "${GREEN}✅ Tagged: ${final_image}${NC}"
        done

        # Clean up temporary image
        echo -e "${YELLOW}🧹 Cleaning up temporary image...${NC}"
        podman rmi "$temp_image" 2>/dev/null || true

    else
        echo -e "${RED}❌ ${PLATFORM} build completed but image not found: ${temp_image}${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ Failed to build ${PLATFORM} image${NC}"
    exit 1
fi

# Clean up intermediate layers and build cache
echo -e "${YELLOW}🧹 Cleaning up intermediate build artifacts...${NC}"
podman image prune -f

echo -e "${GREEN}✅ Successfully built single-platform container images:${NC}"
for final_image in "${FINAL_IMAGES[@]}"; do
    echo -e "  ${GREEN}✓${NC} ${final_image}"
done

# Show image details
echo -e "${YELLOW}📊 Image information:${NC}"
for final_image in "${FINAL_IMAGES[@]}"; do
    echo -e "${YELLOW}${final_image}:${NC}"
    podman inspect "$final_image" --format "  Platform: {{.Os}}/{{.Architecture}}" 2>/dev/null || echo "  Platform info not available"
    size=$(podman images --format "table {{.Size}}" --filter "reference=${final_image}" | tail -n 1)
    echo -e "  Size: ${size}"
    echo
done

echo -e "${GREEN}🎉 Single-platform build completed successfully!${NC}"
echo -e "${BLUE}💡 Built for platform: ${PLATFORM}${NC}"
echo -e "${BLUE}💡 No manifests created - direct image tags only${NC}"
