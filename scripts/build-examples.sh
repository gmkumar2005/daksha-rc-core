#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🎯 Build Examples - Using TAG Parameter${NC}"
echo -e "${BLUE}This script shows examples of how to use the TAG parameter with build tasks${NC}"
echo

# Function to show example commands
show_example() {
    local title="$1"
    local command="$2"
    local description="$3"

    echo -e "${YELLOW}📋 ${title}${NC}"
    echo -e "${BLUE}   Description: ${description}${NC}"
    echo -e "${GREEN}   Command: ${command}${NC}"
    echo
}

echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo -e "${YELLOW}  Basic Usage Examples${NC}"
echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo

show_example \
    "Build with Git Tag (Default)" \
    "cargo make build-image" \
    "Uses the latest git tag automatically"

show_example \
    "Build with Custom Tag" \
    "TAG=v1.2.3 cargo make build-image" \
    "Build with specific version tag"

show_example \
    "Build Multi-Platform with Custom Tag" \
    "TAG=v2.0.0-beta cargo make build-image-all" \
    "Build for both amd64 and arm64 with beta tag"

echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo -e "${YELLOW}  Build and Push Examples${NC}"
echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo

show_example \
    "Build and Push Current Platform" \
    "TAG=v1.5.0 cargo make build-and-push" \
    "Build for current platform and push to registry"

show_example \
    "Build and Push Multi-Platform" \
    "TAG=v1.5.0 cargo make build-and-push-all" \
    "Build for both platforms and push to registry"

show_example \
    "Build and Push with Cleanup" \
    "TAG=v1.5.0 cargo make build-and-push-all-clean" \
    "Clean cache, build multi-platform, and push"

echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo -e "${YELLOW}  Alternative Syntax Examples${NC}"
echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo

show_example \
    "Using --env Flag" \
    "cargo make build-with-tag --env TAG=v1.0.0" \
    "Alternative syntax using cargo-make --env flag"

show_example \
    "Multi-Platform with --env" \
    "cargo make build-all-with-tag --env TAG=v2.0.0-rc1" \
    "Multi-platform build using --env syntax"

show_example \
    "Complete Workflow" \
    "cargo make build-and-push-all-with-tag --env TAG=v1.0.0" \
    "Complete build and push workflow with custom tag"

echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo -e "${YELLOW}  Special Tag Examples${NC}"
echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo

show_example \
    "Development Build" \
    "TAG=dev-$(date +%Y%m%d) cargo make build-image" \
    "Build with date-based development tag"

show_example \
    "Feature Branch Build" \
    "TAG=feature-auth-$(git rev-parse --short HEAD) cargo make build-image" \
    "Build with feature branch and commit hash"

show_example \
    "Release Candidate" \
    "TAG=v1.0.0-rc.$(date +%Y%m%d%H%M) cargo make build-image-all" \
    "Build release candidate with timestamp"

echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo -e "${YELLOW}  Resulting Image Tags${NC}"
echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo

echo -e "${BLUE}When you specify TAG=v1.2.3, the following images are created:${NC}"
echo
echo -e "${GREEN}Platform-specific images:${NC}"
echo -e "  • ghcr.io/daksha-rc/rc-web:v1.2.3-amd64"
echo -e "  • ghcr.io/daksha-rc/rc-web:v1.2.3-arm64"
echo -e "  • ghcr.io/daksha-rc/rc-web:latest-amd64"
echo -e "  • ghcr.io/daksha-rc/rc-web:latest-arm64"
echo
echo -e "${GREEN}Multi-platform manifests (from build-image-all):${NC}"
echo -e "  • ghcr.io/daksha-rc/rc-web:v1.2.3"
echo -e "  • ghcr.io/daksha-rc/rc-web:latest"
echo -e "  • ghcr.io/daksha-rc/rc-web:{commit-sha}"
echo

echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo -e "${YELLOW}  CI/CD Examples${NC}"
echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo

show_example \
    "GitHub Actions Release" \
    "TAG=\${{ github.ref_name }} cargo make build-and-push-all" \
    "Use GitHub release tag in CI pipeline"

show_example \
    "Jenkins Build" \
    "TAG=\${BUILD_TAG} cargo make build-and-push-all-clean" \
    "Use Jenkins build tag with cleanup"

show_example \
    "GitLab CI" \
    "TAG=\${CI_COMMIT_TAG:-\${CI_COMMIT_SHORT_SHA}} cargo make build-and-push" \
    "Use GitLab tag or fallback to commit SHA"

echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo -e "${YELLOW}  Tips and Best Practices${NC}"
echo -e "${YELLOW}═══════════════════════════════════════${NC}"
echo

echo -e "${BLUE}💡 Tips:${NC}"
echo -e "  • If TAG is not specified, the latest git tag is used automatically"
echo -e "  • Use semantic versioning (v1.2.3) for release builds"
echo -e "  • Use descriptive tags for development builds (dev-feature-name)"
echo -e "  • Multi-platform builds take longer but create universal images"
echo -e "  • Use *-clean variants when disk space is limited"
echo

echo -e "${BLUE}🚀 Quick Start:${NC}"
echo -e "  1. ${GREEN}TAG=v1.0.0 cargo make build-image${NC}     # Fast single-platform build"
echo -e "  2. ${GREEN}TAG=v1.0.0 cargo make build-image-all${NC} # Multi-platform build"
echo -e "  3. ${GREEN}TAG=v1.0.0 cargo make push-image${NC}      # Push to registry"
echo

echo -e "${GREEN}🎉 Ready to build with custom tags!${NC}"
