#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🏗️  Building multi-platform container images for rc-web (amd64 & arm64)...${NC}"
echo -e "${BLUE}💡 Using smart build script for optimized native/emulated builds${NC}"

# Check if smart build script exists
SMART_SCRIPT="$(dirname "$0")/build-image-smart.sh"
if [ -f "$SMART_SCRIPT" ]; then
    echo -e "${GREEN}🧠 Using smart build script: $SMART_SCRIPT${NC}"
    USE_SMART_SCRIPT=true
else
    echo -e "${YELLOW}⚠️  Smart build script not found, using legacy method${NC}"
    USE_SMART_SCRIPT=false
fi

# Check available disk space before building
echo -e "${YELLOW}📊 Checking available disk space...${NC}"
AVAILABLE_SPACE=$(df / | awk 'NR==2 {print $4}')
AVAILABLE_GB=$((AVAILABLE_SPACE / 1024 / 1024))

if [ $AVAILABLE_GB -lt 15 ]; then
    echo -e "${RED}⚠️  Low disk space: ${AVAILABLE_GB}GB available${NC}"
    echo -e "${YELLOW}💡 Multi-platform builds require more space. Consider running 'cargo make clean-build-cache' first${NC}"
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

# Define platform-specific image names
PLATFORM_IMAGES=(
    "${IMAGE_BASE}:${GIT_TAG}-amd64"
    "${IMAGE_BASE}:${GIT_TAG}-arm64"
    "${IMAGE_BASE}:${GIT_SHA}-amd64"
    "${IMAGE_BASE}:${GIT_SHA}-arm64"
    "${IMAGE_BASE}:latest-amd64"
    "${IMAGE_BASE}:latest-arm64"
)

# Define final manifest names (these will be multi-platform)
MANIFEST_IMAGES=(
    "${IMAGE_BASE}:${GIT_TAG}"
    "${IMAGE_BASE}:${GIT_SHA}"
    "${IMAGE_BASE}:latest"
)

echo -e "${YELLOW}🔨 Building multi-platform images for amd64 and arm64${NC}"
echo -e "${YELLOW}📋 Platform-specific images:${NC}"
for image in "${PLATFORM_IMAGES[@]}"; do
    echo -e "  - ${image}"
done
echo -e "${YELLOW}📋 Multi-platform manifests:${NC}"
for image in "${MANIFEST_IMAGES[@]}"; do
    echo -e "  - ${image}"
done

# Execute build using smart script or legacy method
if [[ "$USE_SMART_SCRIPT" == "true" ]]; then
    echo -e "${GREEN}🧠 Delegating to smart build script...${NC}"

    # Export tag for smart script
    export TAG="$GIT_TAG"

    # Execute smart build with multi-platform option
    if bash "$SMART_SCRIPT" --verbose build-multi; then
        echo -e "${GREEN}✅ Smart build completed successfully${NC}"
    else
        echo -e "${RED}❌ Smart build failed${NC}"
        exit 1
    fi

    # Tag images with additional variants
    echo -e "${YELLOW}🏷️  Creating additional image tags...${NC}"

    # Tag with commit SHA and latest variants
    if podman image exists "${IMAGE_BASE}:${GIT_TAG}-amd64"; then
        podman tag "${IMAGE_BASE}:${GIT_TAG}-amd64" "${IMAGE_BASE}:${GIT_SHA}-amd64"
        podman tag "${IMAGE_BASE}:${GIT_TAG}-amd64" "${IMAGE_BASE}:latest-amd64"
        echo -e "${GREEN}✅ Tagged amd64 image variants${NC}"
    fi

    if podman image exists "${IMAGE_BASE}:${GIT_TAG}-arm64"; then
        podman tag "${IMAGE_BASE}:${GIT_TAG}-arm64" "${IMAGE_BASE}:${GIT_SHA}-arm64"
        podman tag "${IMAGE_BASE}:${GIT_TAG}-arm64" "${IMAGE_BASE}:latest-arm64"
        echo -e "${GREEN}✅ Tagged arm64 image variants${NC}"
    fi

else
    # Legacy build method
    echo -e "${YELLOW}🔧 Using legacy build method...${NC}"

    # Clean up any existing images and manifests with these names (but preserve base-builder stages)
    echo -e "${YELLOW}🧹 Cleaning up existing images and manifests (preserving base-builder stages)...${NC}"
    for image in "${PLATFORM_IMAGES[@]}" "${MANIFEST_IMAGES[@]}"; do
        # Remove manifest if it exists
        if podman manifest exists "$image" 2>/dev/null; then
            echo -e "${YELLOW}  Removing existing manifest: ${image}${NC}"
            podman manifest rm "$image" 2>/dev/null || true
        fi
        # Don't remove images that might be base-builder stages
        if [[ "$image" != *"base"* && "$image" != *"builder"* ]]; then
            podman rmi "$image" 2>/dev/null || true
        fi
    done

    # Build for amd64
    echo -e "${GREEN}🦭 Building for amd64...${NC}"
    if podman build \
        --arch amd64 \
        -f "rc-web/Dockerfile" \
        -t "${IMAGE_BASE}:${GIT_TAG}-amd64" \
        --label "org.opencontainers.image.version=${GIT_TAG}" \
        --label "org.opencontainers.image.revision=${GIT_SHA}" \
        --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        --label "org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc" \
        --label "org.opencontainers.image.description=Daksha RC Web Application" \
        --label "org.opencontainers.image.platform=amd64" \
        --squash-all \
        --rm \
        .; then
        echo -e "${GREEN}✅ Successfully built amd64 image${NC}"
    else
        echo -e "${RED}❌ Failed to build amd64 image${NC}"
        exit 1
    fi

    # Tag amd64 image with other amd64 variants
    podman tag "${IMAGE_BASE}:${GIT_TAG}-amd64" "${IMAGE_BASE}:${GIT_SHA}-amd64"
    podman tag "${IMAGE_BASE}:${GIT_TAG}-amd64" "${IMAGE_BASE}:latest-amd64"

    # Build for arm64 with improved error handling
    echo -e "${GREEN}🦭 Building for arm64...${NC}"

    # Set memory limits and timeout for ARM64 build
    ARM64_BUILD_TIMEOUT="${ARM64_BUILD_TIMEOUT:-3600}"  # 1 hour default
    ARM64_MEMORY_LIMIT="${ARM64_MEMORY_LIMIT:-4g}"

    if timeout "$ARM64_BUILD_TIMEOUT" podman build \
        --arch arm64 \
        --memory="$ARM64_MEMORY_LIMIT" \
        --build-arg RING_DISABLE_ASSEMBLY=1 \
        --build-arg RING_PREGENERATE_ASM=0 \
        --build-arg RUSTFLAGS="-C opt-level=1" \
        --build-arg CARGO_BUILD_JOBS=1 \
        -f "rc-web/Dockerfile" \
        -t "${IMAGE_BASE}:${GIT_TAG}-arm64" \
        --label "org.opencontainers.image.version=${GIT_TAG}" \
        --label "org.opencontainers.image.revision=${GIT_SHA}" \
        --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        --label "org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc" \
        --label "org.opencontainers.image.description=Daksha RC Web Application" \
        --label "org.opencontainers.image.platform=arm64" \
        --squash-all \
        --rm \
        .; then
        echo -e "${GREEN}✅ Successfully built arm64 image${NC}"
    else
        echo -e "${RED}❌ Failed to build arm64 image${NC}"
        echo -e "${YELLOW}💡 ARM64 build failed. This is often due to ring crate compilation issues in QEMU emulation.${NC}"
        echo -e "${YELLOW}💡 Consider using native ARM64 runners or the smart build script for better compatibility.${NC}"
        exit 1
    fi

    # Tag arm64 image with other arm64 variants
    podman tag "${IMAGE_BASE}:${GIT_TAG}-arm64" "${IMAGE_BASE}:${GIT_SHA}-arm64"
    podman tag "${IMAGE_BASE}:${GIT_TAG}-arm64" "${IMAGE_BASE}:latest-arm64"
fi

# Create manifests for each final image
for manifest_image in "${MANIFEST_IMAGES[@]}"; do
    echo -e "${YELLOW}📋 Creating manifest: ${manifest_image}${NC}"

    # Create manifest
    if podman manifest create "$manifest_image"; then
        echo -e "${GREEN}✅ Created manifest: ${manifest_image}${NC}"
    else
        echo -e "${RED}❌ Failed to create manifest: ${manifest_image}${NC}"
        exit 1
    fi

    # Determine the corresponding platform-specific image names
    if [[ "$manifest_image" == *":${GIT_TAG}" ]]; then
        AMD64_IMAGE="${IMAGE_BASE}:${GIT_TAG}-amd64"
        ARM64_IMAGE="${IMAGE_BASE}:${GIT_TAG}-arm64"
    elif [[ "$manifest_image" == *":${GIT_SHA}" ]]; then
        AMD64_IMAGE="${IMAGE_BASE}:${GIT_SHA}-amd64"
        ARM64_IMAGE="${IMAGE_BASE}:${GIT_SHA}-arm64"
    else  # latest
        AMD64_IMAGE="${IMAGE_BASE}:latest-amd64"
        ARM64_IMAGE="${IMAGE_BASE}:latest-arm64"
    fi

    # Add amd64 image to manifest
    echo -e "${YELLOW}  Adding amd64 to manifest...${NC}"
    if podman manifest add "$manifest_image" "containers-storage:${AMD64_IMAGE}"; then
        echo -e "${GREEN}  ✅ Added amd64 to manifest${NC}"
    else
        echo -e "${RED}  ❌ Failed to add amd64 to manifest${NC}"
        exit 1
    fi

    # Add arm64 image to manifest
    echo -e "${YELLOW}  Adding arm64 to manifest...${NC}"
    if podman manifest add "$manifest_image" "containers-storage:${ARM64_IMAGE}"; then
        echo -e "${GREEN}  ✅ Added arm64 to manifest${NC}"
    else
        echo -e "${RED}  ❌ Failed to add arm64 to manifest${NC}"
        exit 1
    fi

    # Inspect the manifest
    echo -e "${YELLOW}  Inspecting manifest: ${manifest_image}${NC}"
    if podman manifest inspect "$manifest_image" >/dev/null 2>&1; then
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
echo -e "${GREEN}Platform-specific images:${NC}"
for image in "${PLATFORM_IMAGES[@]}"; do
    echo -e "  ${GREEN}✓${NC} ${image}"
done
echo -e "${GREEN}Multi-platform manifests:${NC}"
for image in "${MANIFEST_IMAGES[@]}"; do
    echo -e "  ${GREEN}✓${NC} ${image}"
done

# Show final manifest details
echo -e "${YELLOW}📊 Final manifest information:${NC}"
for manifest_image in "${MANIFEST_IMAGES[@]}"; do
    echo -e "${YELLOW}${manifest_image}:${NC}"
    if command -v jq >/dev/null 2>&1; then
        # Get manifest details and parse with jq
        manifest_json=$(podman manifest inspect "$manifest_image" 2>/dev/null || echo "{}")
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
if [[ "$USE_SMART_SCRIPT" == "true" ]]; then
    echo -e "${BLUE}💡 Built using smart script with optimized native/emulated strategy${NC}"
else
    echo -e "${BLUE}💡 Built using legacy method with QEMU emulation${NC}"
fi
echo -e "${BLUE}💡 Platform-specific images created with -amd64 and -arm64 suffixes${NC}"
echo -e "${BLUE}💡 Multi-platform manifests created with amd64 and arm64 support${NC}"
echo -e "${BLUE}💡 Images will automatically select the correct platform when pulled${NC}"
echo -e "${CYAN}💡 Local builds remain fully compatible with existing cargo-make workflows${NC}"
