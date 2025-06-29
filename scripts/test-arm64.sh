#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 ARM64 Build Capability Diagnostic${NC}"
echo "=========================================="

# Check if podman is available
if ! command -v podman >/dev/null 2>&1; then
    echo -e "${RED}❌ Podman not found${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Podman found${NC}"

# Test 1: Check podman info
echo -e "\n${YELLOW}📊 Podman System Information:${NC}"
podman info | grep -E "(arch|Architecture|os|OS)" | head -5

# Test 2: Check if QEMU is available for emulation
echo -e "\n${YELLOW}🔧 Checking ARM64 Emulation Support:${NC}"
if podman run --rm --platform=linux/arm64 alpine:latest uname -m 2>/dev/null; then
    echo -e "${GREEN}✅ ARM64 emulation working${NC}"
    ARM64_EMULATION=true
else
    echo -e "${RED}❌ ARM64 emulation not working${NC}"
    ARM64_EMULATION=false
fi

# Test 3: Try to pull a simple ARM64 image
echo -e "\n${YELLOW}📥 Testing ARM64 Image Pull:${NC}"
if podman pull --platform=linux/arm64 alpine:latest >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Can pull ARM64 images${NC}"
    ARM64_PULL=true
else
    echo -e "${RED}❌ Cannot pull ARM64 images${NC}"
    ARM64_PULL=false
fi

# Test 4: Try a simple ARM64 build
echo -e "\n${YELLOW}🏗️  Testing Simple ARM64 Build:${NC}"
TEMP_DIR=$(mktemp -d)
cat > "$TEMP_DIR/Dockerfile" << 'EOF'
FROM alpine:latest
RUN echo "Testing ARM64 build" > /test.txt
CMD ["cat", "/test.txt"]
EOF

if podman build --platform=linux/arm64 -t test-arm64:latest "$TEMP_DIR" >/dev/null 2>&1; then
    echo -e "${GREEN}✅ Simple ARM64 build successful${NC}"
    ARM64_BUILD=true

    # Clean up test image
    podman rmi test-arm64:latest >/dev/null 2>&1 || true
else
    echo -e "${RED}❌ Simple ARM64 build failed${NC}"
    ARM64_BUILD=false
fi

# Clean up temp directory
rm -rf "$TEMP_DIR"

# Test 5: Check available platforms
echo -e "\n${YELLOW}🌐 Available Build Platforms:${NC}"
if podman run --rm alpine:latest uname -m 2>/dev/null; then
    echo -e "Native architecture: $(podman run --rm alpine:latest uname -m 2>/dev/null)"
fi

# Summary and recommendations
echo -e "\n${BLUE}📋 Summary:${NC}"
echo "=========================================="

if [ "$ARM64_EMULATION" = true ] && [ "$ARM64_PULL" = true ] && [ "$ARM64_BUILD" = true ]; then
    echo -e "${GREEN}🎉 ARM64 builds should work perfectly!${NC}"
    echo -e "${GREEN}✅ Recommendation: Use 'cargo make build-image' for multi-platform builds${NC}"
elif [ "$ARM64_EMULATION" = true ] && [ "$ARM64_PULL" = true ]; then
    echo -e "${YELLOW}⚠️  ARM64 emulation works but builds may be slow or fail${NC}"
    echo -e "${YELLOW}💡 Recommendation: Use 'cargo make build-image-amd64' for development${NC}"
    echo -e "${YELLOW}💡 Try multi-platform builds for production: 'cargo make build-image'${NC}"
else
    echo -e "${RED}❌ ARM64 builds not supported on this system${NC}"
    echo -e "${RED}💡 Recommendation: Use 'cargo make build-image-amd64' exclusively${NC}"
fi

echo -e "\n${BLUE}🔧 Troubleshooting Tips:${NC}"
echo "----------------------------------------"

if [ "$ARM64_EMULATION" = false ]; then
    echo -e "${YELLOW}• ARM64 emulation not working:${NC}"
    echo "  - Try: podman machine stop && podman machine start"
    echo "  - Or: podman run --rm --privileged multiarch/qemu-user-static --reset -p yes"
fi

if [ "$ARM64_PULL" = false ]; then
    echo -e "${YELLOW}• Cannot pull ARM64 images:${NC}"
    echo "  - Check network connectivity"
    echo "  - Try: podman system reset (WARNING: removes all images)"
fi

if [ "$ARM64_BUILD" = false ]; then
    echo -e "${YELLOW}• ARM64 builds failing:${NC}"
    echo "  - ARM64 builds can be slow and resource-intensive"
    echo "  - Check available disk space: cargo make check-disk-space"
    echo "  - Use single-platform builds for development"
fi

echo -e "\n${GREEN}💡 Quick Commands:${NC}"
echo "• Single platform (fast): cargo make build-image-amd64"
echo "• Multi-platform (slower): cargo make build-image"
echo "• Force AMD64 only: BUILD_PLATFORMS=\"linux/amd64\" cargo make build-image"

if [ "$ARM64_EMULATION" = true ] && [ "$ARM64_BUILD" = true ]; then
    exit 0
else
    exit 1
fi
