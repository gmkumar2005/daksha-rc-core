#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🏗️  Building multi-platform container image for rc-web...${NC}"

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

# Define image base name
IMAGE_BASE="ghcr.io/daksha-rc/rc-web"

# Define final image names (these will be manifests)
FINAL_IMAGES=(
    "${IMAGE_BASE}:${GIT_TAG}"
    "${IMAGE_BASE}:${GIT_SHA}"
    "${IMAGE_BASE}:latest"
)

echo -e "${YELLOW}🔨 Building multi-platform images for amd64 and arm64${NC}"
echo -e "${YELLOW}📋 Target images:${NC}"
for image in "${FINAL_IMAGES[@]}"; do
    echo -e "  - ${image}"
done

# Clean up any existing images and manifests with these names
echo -e "${YELLOW}🧹 Cleaning up existing images and manifests...${NC}"
for image in "${FINAL_IMAGES[@]}"; do
    # Remove manifest if it exists
    if podman manifest exists "$image" 2>/dev/null; then
        echo -e "${YELLOW}  Removing existing manifest: ${image}${NC}"
        podman manifest rm "$image" 2>/dev/null || true
    fi
    # Remove image if it exists
    podman rmi "$image" 2>/dev/null || true
done

# Clean up existing platform-specific images
podman rmi "${IMAGE_BASE}:amd64" 2>/dev/null || true
podman rmi "${IMAGE_BASE}:arm64" 2>/dev/null || true

# Build for amd64
echo -e "${GREEN}🦭 Building for amd64...${NC}"
if podman build \
    --arch amd64 \
    -f "rc-web/Dockerfile" \
    -t "${IMAGE_BASE}:amd64" \
    --label "org.opencontainers.image.version=${GIT_TAG}" \
    --label "org.opencontainers.image.revision=${GIT_SHA}" \
    --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --label "org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc" \
    --label "org.opencontainers.image.description=Daksha RC Web Application" \
    --squash-all \
    --rm \
    .; then
    echo -e "${GREEN}✅ Successfully built amd64 image${NC}"
else
    echo -e "${RED}❌ Failed to build amd64 image${NC}"
    exit 1
fi

# Build for arm64
echo -e "${GREEN}🦭 Building for arm64...${NC}"
if podman build \
    --arch arm64 \
    -f "rc-web/Dockerfile" \
    -t "${IMAGE_BASE}:arm64" \
    --label "org.opencontainers.image.version=${GIT_TAG}" \
    --label "org.opencontainers.image.revision=${GIT_SHA}" \
    --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --label "org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc" \
    --label "org.opencontainers.image.description=Daksha RC Web Application" \
    --squash-all \
    --rm \
    .; then
    echo -e "${GREEN}✅ Successfully built arm64 image${NC}"
else
    echo -e "${RED}❌ Failed to build arm64 image${NC}"
    exit 1
fi

# Create manifests for each final image
for final_image in "${FINAL_IMAGES[@]}"; do
    echo -e "${YELLOW}📋 Creating manifest: ${final_image}${NC}"

    # Create manifest
    if podman manifest create "$final_image"; then
        echo -e "${GREEN}✅ Created manifest: ${final_image}${NC}"
    else
        echo -e "${RED}❌ Failed to create manifest: ${final_image}${NC}"
        exit 1
    fi

    # Add amd64 image to manifest
    echo -e "${YELLOW}  Adding amd64 to manifest...${NC}"
    if podman manifest add "$final_image" "containers-storage:${IMAGE_BASE}:amd64"; then
        echo -e "${GREEN}  ✅ Added amd64 to manifest${NC}"
    else
        echo -e "${RED}  ❌ Failed to add amd64 to manifest${NC}"
        exit 1
    fi

    # Add arm64 image to manifest
    echo -e "${YELLOW}  Adding arm64 to manifest...${NC}"
    if podman manifest add "$final_image" "containers-storage:${IMAGE_BASE}:arm64"; then
        echo -e "${GREEN}  ✅ Added arm64 to manifest${NC}"
    else
        echo -e "${RED}  ❌ Failed to add arm64 to manifest${NC}"
        exit 1
    fi

    # Inspect the manifest
    echo -e "${YELLOW}  Inspecting manifest: ${final_image}${NC}"
    if podman manifest inspect "$final_image"; then
        echo -e "${GREEN}  ✅ Manifest inspection completed${NC}"
    else
        echo -e "${RED}  ❌ Failed to inspect manifest${NC}"
    fi
    echo
done

# Clean up intermediate layers and build cache
echo -e "${YELLOW}🧹 Cleaning up intermediate build artifacts...${NC}"
podman image prune -f

echo -e "${GREEN}✅ Successfully built container images:${NC}"
for final_image in "${FINAL_IMAGES[@]}"; do
    echo -e "  ${GREEN}✓${NC} ${final_image}"
done

# Show final manifest details
echo -e "${YELLOW}📊 Final manifest information:${NC}"
for final_image in "${FINAL_IMAGES[@]}"; do
    echo -e "${YELLOW}${final_image}:${NC}"
    if command -v jq >/dev/null 2>&1; then
        # Get manifest details and parse with jq
        manifest_json=$(podman manifest inspect "$final_image" 2>/dev/null || echo "{}")
        platform_count=$(echo "$manifest_json" | jq '.manifests | length' 2>/dev/null || echo "0")
        echo -e "  Platforms: ${platform_count}"
        # List platforms
        echo "$manifest_json" | jq -r '.manifests[]? | "    - \(.platform.os)/\(.platform.architecture)"' 2>/dev/null || true
    else
        # Fallback without jq
        echo -e "  Multi-platform manifest (install 'jq' for detailed platform info)"
    fi
    echo
done

echo -e "${GREEN}🎉 Multi-platform build completed successfully!${NC}"
echo -e "${BLUE}💡 Manifests created with amd64 and arm64 support${NC}"
echo -e "${BLUE}💡 Images will automatically select the correct platform when pulled${NC}"
