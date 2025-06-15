#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

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

echo -e "${YELLOW}🏗️  Building Docker image for rc-web...${NC}"

# Get the latest git tag with fallbacks
# Uses semantic versioning tags (e.g., v0.0.9)
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

# Detect container engine (Docker or Podman)
if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
    if docker version | grep -q "podman\|Podman"; then
        CONTAINER_ENGINE="podman"
        echo -e "${GREEN}🦭 Using Podman container engine${NC}"
    else
        CONTAINER_ENGINE="docker"
        echo -e "${GREEN}🐳 Using Docker container engine${NC}"
    fi
elif command -v podman >/dev/null 2>&1; then
    CONTAINER_ENGINE="podman"
    echo -e "${GREEN}🦭 Using Podman container engine${NC}"
else
    echo -e "${RED}❌ No container engine found (docker or podman)${NC}"
    exit 1
fi

# Build Docker image with multiple tags for different use cases:
# - Git tag: for versioned releases
# - Git SHA: for commit-specific identification
# - latest: for development and default pulls
echo -e "${YELLOW}🔨 Building image with tags:${NC}"
echo -e "  - ghcr.io/daksha-rc/rc-web:${GIT_TAG}"
echo -e "  - ghcr.io/daksha-rc/rc-web:${GIT_SHA}"
echo -e "  - ghcr.io/daksha-rc/rc-web:latest"

# Build with appropriate container engine and space optimization
if [ "$CONTAINER_ENGINE" = "podman" ]; then
    # Podman build with space optimization
    echo -e "${GREEN}🦭 Building with Podman (space optimized)${NC}"
    podman build \
      -t "ghcr.io/daksha-rc/rc-web:${GIT_TAG}" \
      -t "ghcr.io/daksha-rc/rc-web:${GIT_SHA}" \
      -t "ghcr.io/daksha-rc/rc-web:latest" \
      --label "org.opencontainers.image.version=${GIT_TAG}" \
      --label "org.opencontainers.image.revision=${GIT_SHA}" \
      --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
      --squash-all \
      --rm \
      -f "rc-web/Dockerfile" \
      .
else
    # Docker build with BuildX detection and space optimization
    if docker buildx version >/dev/null 2>&1; then
        echo -e "${GREEN}✓ Using Docker BuildX (space optimized)${NC}"
        docker buildx build \
          -t "ghcr.io/daksha-rc/rc-web:${GIT_TAG}" \
          -t "ghcr.io/daksha-rc/rc-web:${GIT_SHA}" \
          -t "ghcr.io/daksha-rc/rc-web:latest" \
          --label "org.opencontainers.image.version=${GIT_TAG}" \
          --label "org.opencontainers.image.revision=${GIT_SHA}" \
          --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
          --squash \
          --rm \
          -f "rc-web/Dockerfile" \
          . \
          --load
    else
        echo -e "${YELLOW}⚠️  Using legacy Docker builder (space optimized)${NC}"
        docker build \
          -t "ghcr.io/daksha-rc/rc-web:${GIT_TAG}" \
          -t "ghcr.io/daksha-rc/rc-web:${GIT_SHA}" \
          -t "ghcr.io/daksha-rc/rc-web:latest" \
          --label "org.opencontainers.image.version=${GIT_TAG}" \
          --label "org.opencontainers.image.revision=${GIT_SHA}" \
          --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
          --squash \
          --rm \
          -f "rc-web/Dockerfile" \
          .
    fi
fi

# Clean up intermediate layers and build cache after successful build
echo -e "${YELLOW}🧹 Cleaning up intermediate build artifacts...${NC}"
if [ "$CONTAINER_ENGINE" = "podman" ]; then
    podman image prune -f
else
    docker image prune -f
fi

echo -e "${GREEN}✅ Successfully built Docker images:${NC}"
echo -e "  ${GREEN}✓${NC} ghcr.io/daksha-rc/rc-web:${GIT_TAG}"
echo -e "  ${GREEN}✓${NC} ghcr.io/daksha-rc/rc-web:${GIT_SHA}"
echo -e "  ${GREEN}✓${NC} ghcr.io/daksha-rc/rc-web:latest"

# Show image details for verification
echo -e "${YELLOW}📊 Image information:${NC}"
if [ "$CONTAINER_ENGINE" = "podman" ]; then
    podman images "ghcr.io/daksha-rc/rc-web" --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
else
    docker images "ghcr.io/daksha-rc/rc-web" --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
fi