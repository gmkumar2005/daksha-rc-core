#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}💾 Comprehensive Disk Space Analysis${NC}"
echo "=========================================="

# Check macOS host disk space
echo -e "\n${YELLOW}🖥️  macOS Host System:${NC}"
HOST_USAGE=$(df -h / | awk 'NR==2 {print $5}' | sed 's/%//')
HOST_AVAIL=$(df -h / | awk 'NR==2 {print $4}')
echo "Available space: $HOST_AVAIL"
echo "Usage: $HOST_USAGE%"

if [ $HOST_USAGE -gt 90 ]; then
    echo -e "${RED}⚠️  CRITICAL: Host disk is ${HOST_USAGE}% full!${NC}"
    echo -e "${RED}💡 Recommend immediate cleanup of Downloads, Caches, etc.${NC}"
elif [ $HOST_USAGE -gt 80 ]; then
    echo -e "${YELLOW}⚠️  WARNING: Host disk is ${HOST_USAGE}% full${NC}"
else
    echo -e "${GREEN}✅ Host disk space is healthy${NC}"
fi

# Check if Podman is available
if command -v podman >/dev/null 2>&1; then
    echo -e "\n${YELLOW}🦭 Podman Machine Analysis:${NC}"

    # Get Podman machine info
    if podman machine list --format json | jq -e '.[0]' >/dev/null 2>&1; then
        MACHINE_NAME=$(podman machine list --format json | jq -r '.[0].Name')
        MACHINE_DISK=$(podman machine list --format json | jq -r '.[0].DiskSize')
        MACHINE_STATE=$(podman machine list --format json | jq -r '.[0].Running')

        echo "Machine: $MACHINE_NAME"
        echo "Allocated: ${MACHINE_DISK}GB"
        echo "State: $MACHINE_STATE"

        if [ "$MACHINE_STATE" = "true" ]; then
            echo -e "\n${YELLOW}📊 VM Internal Disk Usage:${NC}"
            VM_INFO=$(podman machine ssh -- df -h /sysroot | tail -n 1)
            VM_USAGE=$(echo $VM_INFO | awk '{print $5}' | sed 's/%//')
            VM_AVAIL=$(echo $VM_INFO | awk '{print $4}')
            VM_USED=$(echo $VM_INFO | awk '{print $3}')

            echo "VM disk used: $VM_USED / ${MACHINE_DISK}GB ($VM_USAGE%)"
            echo "VM available: $VM_AVAIL"

            if [ $VM_USAGE -gt 85 ]; then
                echo -e "${RED}⚠️  CRITICAL: VM disk is ${VM_USAGE}% full!${NC}"
                echo -e "${RED}💡 Run 'cargo make clean-build-cache' immediately${NC}"
            elif [ $VM_USAGE -gt 70 ]; then
                echo -e "${YELLOW}⚠️  WARNING: VM disk is ${VM_USAGE}% full${NC}"
            else
                echo -e "${GREEN}✅ VM disk space is healthy${NC}"
            fi

            # Container storage analysis
            echo -e "\n${YELLOW}📦 Container Storage Usage:${NC}"
            podman system df

            # Show manifest storage if any exist
            echo -e "\n${YELLOW}📋 Manifest Storage:${NC}"
            MANIFEST_COUNT=$(podman manifest ls --format "{{.Name}}" 2>/dev/null | wc -l | tr -d ' ')
            if [ "$MANIFEST_COUNT" -gt 0 ]; then
                echo "Active manifests: $MANIFEST_COUNT"
                podman manifest ls --format "table {{.Name}}\t{{.Size}}" 2>/dev/null || echo "  No detailed manifest info available"
            else
                echo "No active manifests"
            fi

        else
            echo -e "${YELLOW}⚠️  Podman machine is not running${NC}"
            echo -e "${BLUE}💡 Start with: podman machine start${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  No Podman machines found${NC}"
        echo -e "${BLUE}💡 Initialize with: podman machine init${NC}"
    fi
else
    echo -e "\n${RED}❌ Podman not available${NC}"
    echo -e "${BLUE}💡 Install Podman: brew install podman${NC}"
fi

# Storage recommendations
echo -e "\n${BLUE}💡 Storage Recommendations:${NC}"
echo "----------------------------------------"

if [ $HOST_USAGE -gt 90 ]; then
    echo -e "${RED}🚨 URGENT HOST CLEANUP NEEDED:${NC}"
    echo "  • Empty Trash: sudo rm -rf ~/.Trash/*"
    echo "  • Clear Downloads: rm -rf ~/Downloads/*"
    echo "  • Clear Caches: rm -rf ~/Library/Caches/*"
    echo "  • iOS Simulators: xcrun simctl delete unavailable"
    echo "  • Xcode Archives: rm -rf ~/Library/Developer/Xcode/Archives/*"
fi

echo -e "${GREEN}🧹 Podman Cleanup Commands:${NC}"
echo "  • Clean all: cargo make clean-build-cache"
echo "  • Check space: cargo make check-disk-space"
echo "  • Build with cleanup: cargo make build-image-clean"
echo "  • Manual cleanup: podman system prune -af"
echo "  • Reset machine: podman machine stop && podman machine rm"

# Calculate safe build space for multi-platform builds
SAFE_BUILD_SPACE=20  # Increased for multi-platform builds
if [ -n "$VM_AVAIL" ]; then
    VM_AVAIL_GB=$(echo $VM_AVAIL | sed 's/G//' | sed 's/\..*//')  # Remove decimal part
    if [ "$VM_AVAIL_GB" -lt $SAFE_BUILD_SPACE ]; then
        echo -e "\n${RED}⚠️  WARNING: Less than ${SAFE_BUILD_SPACE}GB available for multi-platform builds${NC}"
        echo -e "${YELLOW}💡 Multi-platform builds require more space - consider cleaning container storage${NC}"
    fi
fi

# Show current build strategy recommendation
echo -e "\n${BLUE}🏗️  Build Strategy Recommendations:${NC}"
if [ -n "$VM_AVAIL_GB" ] && [ "$VM_AVAIL_GB" -lt 10 ]; then
    echo -e "${RED}🚨 Use: cargo make build-and-push-clean (with aggressive cleanup)${NC}"
elif [ -n "$VM_AVAIL_GB" ] && [ "$VM_AVAIL_GB" -lt 20 ]; then
    echo -e "${YELLOW}⚠️  Use: cargo make build-image-clean (with cleanup)${NC}"
else
    echo -e "${GREEN}✅ Safe to use: cargo make build-image (normal build)${NC}"
fi

echo -e "\n${GREEN}✅ Disk space analysis complete${NC}"
