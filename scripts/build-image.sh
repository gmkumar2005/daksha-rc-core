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

# Define platforms with environment variable control
if [ -n "$BUILD_PLATFORMS" ]; then
    IFS=',' read -ra PLATFORMS <<< "$BUILD_PLATFORMS"
    echo -e "${BLUE}💡 Using custom platforms from BUILD_PLATFORMS: ${PLATFORMS[*]}${NC}"
else
    PLATFORMS=("linux/amd64" "linux/arm64")
fi

# Define final image names (these will be manifests)
FINAL_IMAGES=(
    "ghcr.io/daksha-rc/rc-web:${GIT_TAG}"
    "ghcr.io/daksha-rc/rc-web:${GIT_SHA}"
    "ghcr.io/daksha-rc/rc-web:latest"
)

if [ ${#PLATFORMS[@]} -eq 1 ]; then
    echo -e "${YELLOW}🔨 Building single-platform image for: ${PLATFORMS[*]}${NC}"
else
    echo -e "${YELLOW}🔨 Building multi-platform images for platforms: ${PLATFORMS[*]}${NC}"
fi

echo -e "${YELLOW}📋 Target images:${NC}"
for image in "${FINAL_IMAGES[@]}"; do
    echo -e "  - ${image}"
done

# Clean up any existing images and manifests with these names
echo -e "${YELLOW}🧹 Cleaning up existing images and manifests...${NC}"
for image in "${FINAL_IMAGES[@]}"; do
    podman manifest rm "$image" 2>/dev/null || true
    podman rmi "$image" 2>/dev/null || true
done

# Build approach: build all platforms first, then create manifests
TEMP_IMAGES=()
TEMP_PLATFORMS=()
BUILD_SUCCESS=true

# Build images for each platform
for platform in "${PLATFORMS[@]}"; do
    echo -e "${GREEN}🦭 Building for platform: ${platform}${NC}"

    # Create temporary image name for this platform
    platform_safe=$(echo "$platform" | tr '/' '-')
    temp_image="localhost/rc-web-temp:${platform_safe}-$(date +%s)"

    echo -e "${YELLOW}  Building ${platform} image...${NC}"

    if podman build \
        --platform "${platform}" \
        -f "rc-web/Dockerfile" \
        -t "${temp_image}" \
        --label "org.opencontainers.image.version=${GIT_TAG}" \
        --label "org.opencontainers.image.revision=${GIT_SHA}" \
        --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        --label "org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc" \
        --label "org.opencontainers.image.description=Daksha RC Web Application" \
        --squash-all \
        --rm \
        .; then

        # Verify the image was actually created
        if podman inspect "$temp_image" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ Successfully built ${platform} image: ${temp_image}${NC}"
            TEMP_IMAGES+=("${temp_image}")
            TEMP_PLATFORMS+=("${platform}")
        else
            echo -e "${RED}❌ ${platform} build completed but image not found: ${temp_image}${NC}"
            echo -e "${YELLOW}⚠️  Continuing with other platforms...${NC}"
        fi
    else
        echo -e "${RED}❌ Failed to build ${platform} image${NC}"
        echo -e "${YELLOW}⚠️  Continuing with other platforms...${NC}"
        # Don't break, continue with other platforms
    fi
done

# Check if we have any successful builds
if [ ${#TEMP_IMAGES[@]} -gt 0 ]; then
    echo -e "${GREEN}📋 Successfully built ${#TEMP_IMAGES[@]} platform(s): ${TEMP_PLATFORMS[*]}${NC}"
    echo -e "${GREEN}📋 Creating final images/manifests...${NC}"

    for final_image in "${FINAL_IMAGES[@]}"; do
        echo -e "${YELLOW}Creating: ${final_image}${NC}"

        if [ ${#TEMP_IMAGES[@]} -eq 1 ]; then
            # Single platform - just tag the image
            temp_image_name="${TEMP_IMAGES[0]}"
            podman tag "$temp_image_name" "$final_image"
            echo -e "${GREEN}✅ Tagged single-platform image${NC}"
        else
            # Multi-platform - create manifest
            podman manifest create "$final_image"

            for i in "${!TEMP_IMAGES[@]}"; do
                temp_image_name="${TEMP_IMAGES[$i]}"
                platform="${TEMP_PLATFORMS[$i]}"

                echo -e "${YELLOW}  Adding ${platform} to manifest...${NC}"

                # Verify image exists before adding
                if ! podman inspect "$temp_image_name" >/dev/null 2>&1; then
                    echo -e "${RED}  ❌ Image ${temp_image_name} not found, skipping${NC}"
                    continue
                fi

                # Show image details for debugging
                echo -e "${BLUE}    Image: ${temp_image_name}${NC}"

                if podman manifest add "$final_image" "$temp_image_name" 2>&1; then
                    echo -e "${GREEN}  ✅ Added ${platform}${NC}"
                else
                    echo -e "${RED}  ❌ Failed to add ${platform} - manifest add error${NC}"
                    echo -e "${YELLOW}  Attempting to inspect manifest state...${NC}"
                    podman manifest inspect "$final_image" >/dev/null 2>&1 || echo -e "${RED}    Manifest appears corrupted${NC}"
                fi
            done
        fi
    done

    # Clean up temporary images
    echo -e "${YELLOW}🧹 Cleaning up temporary images...${NC}"
    for temp_image_name in "${TEMP_IMAGES[@]}"; do
        podman rmi "$temp_image_name" 2>/dev/null || true
    done

else
    echo -e "${RED}❌ No successful builds for any platform!${NC}"
    echo -e "${YELLOW}💡 Troubleshooting suggestions:${NC}"
    echo -e "  - Try single platform: cargo make build-image-amd64${NC}"
    echo -e "  - Check disk space: cargo make check-disk-space${NC}"
    echo -e "  - Check Podman setup: podman info${NC}"
    exit 1
fi

# Clean up intermediate layers and build cache
echo -e "${YELLOW}🧹 Cleaning up intermediate build artifacts...${NC}"
podman image prune -f

echo -e "${GREEN}✅ Successfully built container images:${NC}"
for final_image in "${FINAL_IMAGES[@]}"; do
    echo -e "  ${GREEN}✓${NC} ${final_image}"
done

# Show image/manifest details
echo -e "${YELLOW}📊 Image information:${NC}"
for final_image in "${FINAL_IMAGES[@]}"; do
    echo -e "${YELLOW}${final_image}:${NC}"

    # Try to inspect as manifest first, then as regular image
    if podman manifest inspect "$final_image" >/dev/null 2>&1; then
        platform_count=$(podman manifest inspect "$final_image" --format json | jq '.manifests | length' 2>/dev/null || echo "unknown")
        echo -e "  Type: Multi-platform manifest (${platform_count} platforms)"

        if command -v jq >/dev/null 2>&1; then
            podman manifest inspect "$final_image" --format json | jq -r '.manifests[]? | "  Platform: \(.platform.os)/\(.platform.architecture)"' 2>/dev/null || echo "  Platform details not available"
        else
            echo -e "  Platform details: Install 'jq' for detailed platform info"
        fi
    else
        echo -e "  Type: Single-platform image"
        podman inspect "$final_image" --format "  Platform: {{.Os}}/{{.Architecture}}" 2>/dev/null || echo "  Platform info not available"
    fi
    echo
done

echo -e "${GREEN}🎉 Build completed successfully!${NC}"

if [ ${#TEMP_PLATFORMS[@]} -gt 1 ]; then
    echo -e "${BLUE}💡 Multi-platform manifests created - they will automatically select the correct platform when pulled${NC}"
elif [ ${#TEMP_PLATFORMS[@]} -eq 1 ]; then
    echo -e "${BLUE}💡 Single-platform image created for: ${TEMP_PLATFORMS[0]}${NC}"
fi

if [ ${#TEMP_PLATFORMS[@]} -lt ${#PLATFORMS[@]} ]; then
    failed_platforms=()
    for platform in "${PLATFORMS[@]}"; do
        if [[ ! " ${TEMP_PLATFORMS[*]} " =~ " ${platform} " ]]; then
            failed_platforms+=("${platform}")
        fi
    done
    echo -e "${YELLOW}⚠️  Some platforms failed to build: ${failed_platforms[*]}${NC}"
    echo -e "${YELLOW}💡 To retry failed platforms, run the build again or use single-platform tasks${NC}"
fi
