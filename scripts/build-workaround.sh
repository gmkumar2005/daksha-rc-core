#!/bin/bash
set -e

# Practical Multi-Platform Build Workaround Script
# This script provides a reliable solution for the current ring crate / QEMU emulation issues

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🔧 Multi-Platform Build Workaround${NC}"
echo -e "${CYAN}==================================${NC}"
echo

# Detect current architecture
HOST_ARCH=$(uname -m)
case "$HOST_ARCH" in
    x86_64|amd64)
        NATIVE_PLATFORM="amd64"
        CROSS_PLATFORM="arm64"
        ;;
    aarch64|arm64)
        NATIVE_PLATFORM="arm64"
        CROSS_PLATFORM="amd64"
        ;;
    *)
        echo -e "${RED}❌ Unsupported architecture: $HOST_ARCH${NC}"
        exit 1
        ;;
esac

echo -e "${GREEN}🖥️  Host Architecture: $HOST_ARCH${NC}"
echo -e "${GREEN}⚡ Native Platform: $NATIVE_PLATFORM${NC}"
echo -e "${YELLOW}🔄 Cross Platform: $CROSS_PLATFORM${NC}"
echo

# Get tag information
if [ -n "$TAG" ]; then
    BUILD_TAG="$TAG"
    echo -e "${GREEN}📋 Using provided tag: ${BUILD_TAG}${NC}"
elif BUILD_TAG=$(git describe --tags --abbrev=0 2>/dev/null); then
    echo -e "${GREEN}📋 Using Git tag: ${BUILD_TAG}${NC}"
else
    BUILD_TAG="v0.0.0-dev"
    echo -e "${YELLOW}⚠️  No Git tags found, using default: ${BUILD_TAG}${NC}"
fi

GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
echo -e "${GREEN}📋 Git commit: ${GIT_SHA}${NC}"
echo

# Check what already exists
IMAGE_BASE="ghcr.io/daksha-rc/rc-web"
NATIVE_IMAGE="${IMAGE_BASE}:${BUILD_TAG}-${NATIVE_PLATFORM}"
CROSS_IMAGE="${IMAGE_BASE}:${BUILD_TAG}-${CROSS_PLATFORM}"

echo -e "${YELLOW}🔍 Checking existing images...${NC}"

NATIVE_EXISTS=false
CROSS_EXISTS=false

if command -v podman >/dev/null 2>&1; then
    if podman image exists "$NATIVE_IMAGE"; then
        echo -e "${GREEN}   ✅ Native image exists: ${NATIVE_IMAGE}${NC}"
        NATIVE_EXISTS=true
    else
        echo -e "${YELLOW}   ⚠️  Native image missing: ${NATIVE_IMAGE}${NC}"
    fi

    if podman image exists "$CROSS_IMAGE"; then
        echo -e "${GREEN}   ✅ Cross-platform image exists: ${CROSS_IMAGE}${NC}"
        CROSS_EXISTS=true
    else
        echo -e "${YELLOW}   ⚠️  Cross-platform image missing: ${CROSS_IMAGE}${NC}"
    fi
else
    echo -e "${RED}   ❌ Podman not found${NC}"
    exit 1
fi
echo

# Build strategy
echo -e "${CYAN}📋 Build Strategy${NC}"
echo -e "${CYAN}===============${NC}"

if [ "$NATIVE_EXISTS" = false ]; then
    echo -e "${YELLOW}🔨 Step 1: Build native $NATIVE_PLATFORM image${NC}"
    echo -e "${BLUE}   Command: ./scripts/build-image.sh${NC}"
    echo -e "${BLUE}   This should work reliably on your $HOST_ARCH system${NC}"
    echo

    # Build native image
    echo -e "${GREEN}🚀 Building native image...${NC}"
    if [ -f "scripts/build-image.sh" ]; then
        export SQLX_OFFLINE=true
        export TAG="$BUILD_TAG"
        if bash scripts/build-image.sh; then
            echo -e "${GREEN}✅ Native build completed successfully${NC}"
            NATIVE_EXISTS=true
        else
            echo -e "${RED}❌ Native build failed${NC}"
            exit 1
        fi
    else
        echo -e "${RED}❌ build-image.sh script not found${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}✅ Native $NATIVE_PLATFORM image already exists${NC}"
fi

echo

# Cross-platform strategy
if [ "$CROSS_EXISTS" = false ]; then
    echo -e "${YELLOW}🔄 Step 2: Handle cross-platform $CROSS_PLATFORM image${NC}"
    echo

    if [ "$CROSS_PLATFORM" = "amd64" ]; then
        echo -e "${RED}⚠️  AMD64 cross-compilation has known issues with ring crate in QEMU${NC}"
        echo -e "${YELLOW}💡 Recommended solutions:${NC}"
        echo -e "${BLUE}   Option A: Use GitHub Actions with native AMD64 runners${NC}"
        echo -e "${BLUE}   Option B: Build on an actual AMD64 machine${NC}"
        echo -e "${BLUE}   Option C: Use Docker Desktop with BuildKit (sometimes works better)${NC}"
        echo -e "${BLUE}   Option D: Skip AMD64 for now and deploy ARM64-only${NC}"
        echo

        echo -e "${YELLOW}🤖 For CI/CD: Use the hybrid workflow${NC}"
        echo -e "${BLUE}   File: .github/workflows/build-rc-web-image-hybrid.yml${NC}"
        echo -e "${BLUE}   This uses native runners for each architecture${NC}"

    else
        echo -e "${YELLOW}💡 ARM64 cross-compilation from AMD64 usually works better${NC}"
        echo -e "${BLUE}   You can try: BUILD_TIMEOUT=7200 ./scripts/build-amd64-with-fixes.sh${NC}"
    fi
else
    echo -e "${GREEN}✅ Cross-platform $CROSS_PLATFORM image already exists${NC}"
fi

echo

# Multi-platform manifest creation
if [ "$NATIVE_EXISTS" = true ] && [ "$CROSS_EXISTS" = true ]; then
    echo -e "${CYAN}📦 Creating Multi-Platform Manifests${NC}"
    echo -e "${CYAN}===================================${NC}"

    MANIFEST_IMAGES=(
        "${IMAGE_BASE}:${BUILD_TAG}"
        "${IMAGE_BASE}:latest"
    )

    for manifest in "${MANIFEST_IMAGES[@]}"; do
        echo -e "${YELLOW}📋 Creating manifest: ${manifest}${NC}"

        # Remove existing manifest if it exists
        podman manifest rm "$manifest" 2>/dev/null || true

        # Create new manifest
        if podman manifest create "$manifest"; then
            echo -e "${GREEN}   ✅ Created manifest${NC}"

            # Add platform-specific images
            if podman manifest add "$manifest" "$NATIVE_IMAGE"; then
                echo -e "${GREEN}   ✅ Added $NATIVE_PLATFORM image${NC}"
            else
                echo -e "${RED}   ❌ Failed to add $NATIVE_PLATFORM image${NC}"
            fi

            if podman manifest add "$manifest" "$CROSS_IMAGE"; then
                echo -e "${GREEN}   ✅ Added $CROSS_PLATFORM image${NC}"
            else
                echo -e "${RED}   ❌ Failed to add $CROSS_PLATFORM image${NC}"
            fi

            echo -e "${GREEN}   ✅ Multi-platform manifest ready: ${manifest}${NC}"
        else
            echo -e "${RED}   ❌ Failed to create manifest${NC}"
        fi
        echo
    done
fi

# Summary and next steps
echo -e "${CYAN}📊 Summary${NC}"
echo -e "${CYAN}=========${NC}"

echo -e "${GREEN}✅ Available Images:${NC}"
if [ "$NATIVE_EXISTS" = true ]; then
    echo -e "${GREEN}   ✓ ${NATIVE_IMAGE}${NC}"
fi
if [ "$CROSS_EXISTS" = true ]; then
    echo -e "${GREEN}   ✓ ${CROSS_IMAGE}${NC}"
fi

if [ "$NATIVE_EXISTS" = true ] && [ "$CROSS_EXISTS" = true ]; then
    echo -e "${GREEN}   ✓ ${IMAGE_BASE}:${BUILD_TAG} (multi-platform)${NC}"
    echo -e "${GREEN}   ✓ ${IMAGE_BASE}:latest (multi-platform)${NC}"
fi

echo

echo -e "${CYAN}🚀 Next Steps${NC}"
echo -e "${CYAN}==========${NC}"

if [ "$NATIVE_EXISTS" = true ] && [ "$CROSS_EXISTS" = false ]; then
    echo -e "${YELLOW}1. You have a working $NATIVE_PLATFORM image${NC}"
    echo -e "${YELLOW}2. For $CROSS_PLATFORM, consider using CI/CD with native runners${NC}"
    echo -e "${YELLOW}3. Deploy the $NATIVE_PLATFORM image for now:${NC}"
    echo -e "${BLUE}     podman run -p 8080:8080 ${NATIVE_IMAGE}${NC}"
    echo

    echo -e "${YELLOW}🎯 For complete multi-platform support:${NC}"
    echo -e "${BLUE}   • Set up GitHub Actions with the hybrid workflow${NC}"
    echo -e "${BLUE}   • Use native runners for each architecture${NC}"
    echo -e "${BLUE}   • File: .github/workflows/build-rc-web-image-hybrid.yml${NC}"

elif [ "$NATIVE_EXISTS" = true ] && [ "$CROSS_EXISTS" = true ]; then
    echo -e "${GREEN}🎉 Complete multi-platform setup ready!${NC}"
    echo -e "${GREEN}   • Native $NATIVE_PLATFORM build: ✅${NC}"
    echo -e "${GREEN}   • Cross-platform $CROSS_PLATFORM build: ✅${NC}"
    echo -e "${GREEN}   • Multi-platform manifests: ✅${NC}"
    echo

    echo -e "${BLUE}🚀 Deploy commands:${NC}"
    echo -e "${BLUE}     # Multi-platform (auto-selects architecture)${NC}"
    echo -e "${BLUE}     podman run -p 8080:8080 ${IMAGE_BASE}:${BUILD_TAG}${NC}"
    echo -e "${BLUE}     ${NC}"
    echo -e "${BLUE}     # Platform-specific${NC}"
    echo -e "${BLUE}     podman run -p 8080:8080 ${NATIVE_IMAGE}${NC}"
    echo

    echo -e "${BLUE}📤 Push to registry:${NC}"
    echo -e "${BLUE}     podman push ${IMAGE_BASE}:${BUILD_TAG}${NC}"
    echo -e "${BLUE}     podman push ${IMAGE_BASE}:latest${NC}"

else
    echo -e "${RED}❌ No images built successfully${NC}"
    echo -e "${YELLOW}💡 Try running: ./scripts/build-image.sh${NC}"
fi

echo
echo -e "${CYAN}💡 Integration with existing workflows:${NC}"
echo -e "${BLUE}   • This script works alongside cargo-make${NC}"
echo -e "${BLUE}   • Use 'cargo make build-image' for single platform${NC}"
echo -e "${BLUE}   • Use this script for multi-platform with workarounds${NC}"
echo -e "${BLUE}   • Migrate to hybrid GitHub Actions for full automation${NC}"

echo
echo -e "${GREEN}✅ Workaround script completed!${NC}"
