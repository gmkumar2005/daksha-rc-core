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

# Check if colima is available
if ! command -v colima >/dev/null 2>&1; then
    echo -e "\n${RED}❌ Error: colima not found${NC}"
    echo -e "${BLUE}💡 Install colima with: brew install colima${NC}"
    echo -e "${BLUE}💡 Or visit: https://github.com/abiosoft/colima${NC}"
    exit 1
fi

# Check if nerdctl is available
if ! command -v nerdctl >/dev/null 2>&1; then
    echo -e "\n${RED}❌ Error: nerdctl not found${NC}"
    echo -e "${BLUE}💡 Install nerdctl with: brew install nerdctl${NC}"
    echo -e "${BLUE}💡 Or visit: https://github.com/containerd/nerdctl${NC}"
    exit 1
fi

echo -e "\n${YELLOW}🐋 Colima VM Analysis:${NC}"

# Check colima status - use different approach
if colima status >/dev/null 2>&1; then
    COLIMA_STATUS=$(colima status 2>&1)
    echo "Status: Running"

    # Extract runtime from status output
    RUNTIME=$(echo "$COLIMA_STATUS" | grep "runtime:" | awk '{print $2}' || echo "N/A")
    ARCH=$(echo "$COLIMA_STATUS" | grep "arch:" | awk '{print $2}' || echo "N/A")
    KUBERNETES=$(echo "$COLIMA_STATUS" | grep "kubernetes:" | awk '{print $2}' || echo "N/A")
    echo "Runtime: $RUNTIME"
    echo "Architecture: $ARCH"
    echo "Kubernetes: $KUBERNETES"

    # Get VM info from colima config
    if colima list --format json >/dev/null 2>&1; then
        VM_INFO=$(colima list --format json | jq -r '.[0]')
        VM_DISK=$(echo $VM_INFO | jq -r '.disk // "N/A"')
        VM_MEMORY=$(echo $VM_INFO | jq -r '.memory // "N/A"')

        echo "Allocated disk: ${VM_DISK}"
        echo "Memory: ${VM_MEMORY}"
    else
        echo "Allocated disk: Check with 'colima list'"
    fi

    echo -e "\n${YELLOW}📊 VM Internal Disk Usage:${NC}"
    VM_DISK_INFO=$(colima ssh -- df -h / | tail -n 1)
    VM_USAGE=$(echo $VM_DISK_INFO | awk '{print $5}' | sed 's/%//')
    VM_AVAIL=$(echo $VM_DISK_INFO | awk '{print $4}')
    VM_USED=$(echo $VM_DISK_INFO | awk '{print $3}')

    echo "VM disk used: $VM_USED ($VM_USAGE%)"
    echo "VM available: $VM_AVAIL"

    if [ $VM_USAGE -gt 85 ]; then
        echo -e "${RED}⚠️  CRITICAL: VM disk is ${VM_USAGE}% full!${NC}"
        echo -e "${RED}💡 Run 'cargo make clean-build-cache' immediately${NC}"
    elif [ $VM_USAGE -gt 70 ]; then
        echo -e "${YELLOW}⚠️  WARNING: VM disk is ${VM_USAGE}% full${NC}"
    else
        echo -e "${GREEN}✅ VM disk space is healthy${NC}"
    fi

    # Container storage analysis using nerdctl
    echo -e "\n${YELLOW}📦 Container Storage Usage:${NC}"

    # Show images with size information (default namespace)
    echo -e "\n${YELLOW}📋 Container Images (default namespace):${NC}"
    IMAGE_COUNT=$(nerdctl images -q 2>/dev/null | wc -l | tr -d ' ')
    if [ "$IMAGE_COUNT" -gt 0 ]; then
        echo "Total images: $IMAGE_COUNT"
        nerdctl images --format "table {{.Repository}}	{{.Tag}}	{{.Size}}" 2>/dev/null | head -10
        if [ "$IMAGE_COUNT" -gt 10 ]; then
            echo "... and $(($IMAGE_COUNT - 10)) more"
        fi

        # Calculate total image size (approximate)
        echo -e "\n${YELLOW}💾 Storage Summary (default):${NC}"
        TOTAL_SIZE=$(nerdctl images --format "{{.Size}}" 2>/dev/null | sed 's/[A-Za-z]//g' | awk '{sum += $1} END {print sum}' || echo "N/A")
        if [ "$TOTAL_SIZE" != "N/A" ]; then
            echo "Approximate total image size: ${TOTAL_SIZE}MB"
        fi
    else
        echo "No images found"
    fi

    # Show containers (default namespace)
    echo -e "\n${YELLOW}📦 Container Information (default):${NC}"
    CONTAINER_COUNT=$(nerdctl ps -aq 2>/dev/null | wc -l | tr -d ' ')
    if [ "$CONTAINER_COUNT" -gt 0 ]; then
        echo "Total containers: $CONTAINER_COUNT"
        nerdctl ps --format "table {{.Names}}	{{.Status}}	{{.Size}}" 2>/dev/null | head -5
        if [ "$CONTAINER_COUNT" -gt 5 ]; then
            echo "... and $(($CONTAINER_COUNT - 5)) more"
        fi
    else
        echo "No containers found"
    fi

    # Check k8s.io namespace - test if it exists by checking for images
    echo -e "\n${YELLOW}☸️  Kubernetes Namespace (k8s.io):${NC}"

    # Test if k8s.io namespace exists by trying to list images
    if nerdctl --namespace k8s.io images -q >/dev/null 2>&1; then
        K8S_IMAGE_COUNT=$(nerdctl --namespace k8s.io images -q 2>/dev/null | wc -l | tr -d ' ')
        if [ "$K8S_IMAGE_COUNT" -gt 0 ]; then
            echo "K8s images: $K8S_IMAGE_COUNT"
            nerdctl --namespace k8s.io images --format "table {{.Repository}}	{{.Tag}}	{{.Size}}" 2>/dev/null | head -10
            if [ "$K8S_IMAGE_COUNT" -gt 10 ]; then
                echo "... and $(($K8S_IMAGE_COUNT - 10)) more"
            fi

            # Calculate k8s image size
            echo -e "\n${YELLOW}💾 K8s Storage Summary:${NC}"
            K8S_TOTAL_SIZE=$(nerdctl --namespace k8s.io images --format "{{.Size}}" 2>/dev/null | sed 's/[A-Za-z]//g' | awk '{sum += $1} END {print sum}' || echo "N/A")
            if [ "$K8S_TOTAL_SIZE" != "N/A" ]; then
                echo "Approximate k8s image size: ${K8S_TOTAL_SIZE}MB"
            fi
        else
            echo "No k8s images found"
        fi

        # Show k8s containers
        K8S_CONTAINER_COUNT=$(nerdctl --namespace k8s.io ps -aq 2>/dev/null | wc -l | tr -d ' ')
        if [ "$K8S_CONTAINER_COUNT" -gt 0 ]; then
            echo "K8s containers: $K8S_CONTAINER_COUNT"
            nerdctl --namespace k8s.io ps --format "table {{.Names}}	{{.Status}}	{{.Size}}" 2>/dev/null | head -5
            if [ "$K8S_CONTAINER_COUNT" -gt 5 ]; then
                echo "... and $(($K8S_CONTAINER_COUNT - 5)) more"
            fi
        else
            echo "No k8s containers found"
        fi
    else
        echo -e "${BLUE}ℹ️  k8s.io namespace not accessible or doesn't exist${NC}"
    fi

else
    echo -e "${YELLOW}⚠️  Colima is not running${NC}"
    echo -e "${BLUE}💡 Start with: colima start${NC}"
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

echo -e "${GREEN}🧹 Container Cleanup Commands:${NC}"
echo "  • Check space: cargo make check-disk-space"
echo "  • Manual cleanup: nerdctl system prune -af"
echo "  • Remove unused images: nerdctl image prune -af"
echo "  • Remove all stopped containers: nerdctl container prune -f"
echo "  • Clean k8s namespace: nerdctl --namespace k8s.io system prune -af"
echo "  • Reset VM: colima stop && colima delete && colima start"

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