#!/bin/bash
set -e

# Smart multi-platform build script that detects native vs emulated builds
# This script maintains full compatibility with existing cargo-make workflows
# while providing optimized build strategies based on the host architecture

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script metadata
SCRIPT_NAME="$(basename "$0")"
SCRIPT_VERSION="1.0.0"

# Default settings
BUILD_STRATEGY="auto"
TARGET_PLATFORMS="linux/amd64,linux/arm64"
FORCE_EMULATION=false
NATIVE_ONLY=false
VERBOSE=false

# Usage function
usage() {
    cat << EOF
${CYAN}🏗️  Smart Multi-Platform Build Script v${SCRIPT_VERSION}${NC}

This script intelligently chooses between native and emulated builds
based on your system architecture and available tools.

${YELLOW}Usage:${NC}
  $SCRIPT_NAME [OPTIONS] [COMMAND]

${YELLOW}Commands:${NC}
  build-single        Build for current platform only (default)
  build-multi         Build for multiple platforms
  build-and-push      Build and push for current platform
  build-and-push-all  Build and push for all platforms
  detect-only         Just show detection results, don't build

${YELLOW}Options:${NC}
  -s, --strategy STRATEGY    Build strategy: auto, native, emulated (default: auto)
  -p, --platforms PLATFORMS  Target platforms (default: linux/amd64,linux/arm64)
  -f, --force-emulation      Force emulation even if native build is available
  -n, --native-only          Only build for native platform
  -v, --verbose              Enable verbose output
  -h, --help                 Show this help message

${YELLOW}Examples:${NC}
  $SCRIPT_NAME                              # Auto-detect and build current platform
  $SCRIPT_NAME build-multi                  # Build for all platforms using best strategy
  $SCRIPT_NAME build-and-push-all          # Build and push all platforms
  $SCRIPT_NAME --native-only build-single  # Force native-only build
  $SCRIPT_NAME --verbose detect-only       # Show detailed detection info

${YELLOW}Environment Variables:${NC}
  TAG                Set custom tag (otherwise uses git tag)
  SQLX_OFFLINE      Set to 'true' for offline builds (default: true)
  PODMAN_TIMEOUT    Build timeout in seconds (default: 3600)

${YELLOW}Integration with cargo-make:${NC}
  This script works alongside existing cargo-make tasks:
  - cargo make build-image        # Uses this script with build-single
  - cargo make build-image-all    # Uses this script with build-multi
  - cargo make build-and-push-all # Uses this script with build-and-push-all

EOF
}

# Logging functions
log_info() {
    echo -e "${GREEN}ℹ️  $1${NC}"
}

log_warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

log_verbose() {
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${BLUE}🔍 $1${NC}"
    fi
}

log_step() {
    echo -e "${CYAN}🔧 $1${NC}"
}

# System detection functions
detect_host_arch() {
    local arch=$(uname -m)
    case $arch in
        x86_64|amd64)
            echo "amd64"
            ;;
        aarch64|arm64)
            echo "arm64"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

detect_container_engine() {
    if command -v podman >/dev/null 2>&1; then
        echo "podman"
    elif command -v docker >/dev/null 2>&1; then
        echo "docker"
    else
        echo "none"
    fi
}

check_qemu_support() {
    local engine="$1"

    if [[ "$engine" == "podman" ]]; then
        # Check if QEMU is available and binfmt is set up
        if command -v qemu-user-static >/dev/null 2>&1; then
            # Test if we can run a simple ARM64 container
            if timeout 30s podman run --rm --platform linux/arm64 alpine:latest uname -m >/dev/null 2>&1; then
                echo "available"
            else
                echo "needs-setup"
            fi
        else
            echo "missing"
        fi
    elif [[ "$engine" == "docker" ]]; then
        # Check Docker buildx support
        if docker buildx version >/dev/null 2>&1; then
            echo "available"
        else
            echo "missing"
        fi
    else
        echo "none"
    fi
}

check_native_build_support() {
    local host_arch="$1"
    local target_arch="$2"

    if [[ "$host_arch" == "$target_arch" ]]; then
        echo "native"
    else
        echo "cross"
    fi
}

# Build strategy determination
determine_build_strategy() {
    local host_arch="$1"
    local target_platforms="$2"
    local force_emulation="$3"
    local native_only="$4"

    log_verbose "Determining build strategy..."
    log_verbose "Host architecture: $host_arch"
    log_verbose "Target platforms: $target_platforms"
    log_verbose "Force emulation: $force_emulation"
    log_verbose "Native only: $native_only"

    # Parse target platforms
    IFS=',' read -ra PLATFORMS <<< "$target_platforms"

    local strategy_map=""
    local has_native=false
    local has_cross=false

    for platform in "${PLATFORMS[@]}"; do
        local arch=$(echo "$platform" | cut -d'/' -f2)
        local build_type=$(check_native_build_support "$host_arch" "$arch")

        if [[ "$build_type" == "native" ]]; then
            has_native=true
            strategy_map+="$arch:native "
        else
            has_cross=true
            if [[ "$native_only" == "true" ]]; then
                log_warn "Skipping $arch (native-only mode enabled)"
                continue
            fi
            strategy_map+="$arch:emulated "
        fi
    done

    log_verbose "Build strategy map: $strategy_map"
    echo "$strategy_map"
}

# Environment setup
setup_build_environment() {
    local container_engine="$1"
    local qemu_support="$2"

    log_step "Setting up build environment..."

    # Set default environment variables
    export SQLX_OFFLINE="${SQLX_OFFLINE:-true}"
    export PODMAN_TIMEOUT="${PODMAN_TIMEOUT:-3600}"
    export CARGO_TERM_COLOR="${CARGO_TERM_COLOR:-always}"

    # Determine tag
    if [ -n "$TAG" ]; then
        export BUILD_TAG="$TAG"
        log_info "Using provided tag: $BUILD_TAG"
    elif BUILD_TAG=$(git describe --tags --abbrev=0 2>/dev/null); then
        export BUILD_TAG="$BUILD_TAG"
        log_info "Using Git tag: $BUILD_TAG"
    else
        export BUILD_TAG="v0.0.0-dev"
        log_warn "No Git tags found, using default: $BUILD_TAG"
    fi

    # Setup QEMU if needed
    if [[ "$qemu_support" == "needs-setup" ]]; then
        log_step "Setting up QEMU emulation..."

        if [[ "$container_engine" == "podman" ]]; then
            # Setup QEMU for podman
            sudo podman run --rm --privileged multiarch/qemu-user-static --reset -p yes
            sleep 2
            log_info "QEMU emulation setup complete"
        fi
    fi

    log_verbose "Environment setup complete"
    log_verbose "SQLX_OFFLINE: $SQLX_OFFLINE"
    log_verbose "BUILD_TAG: $BUILD_TAG"
    log_verbose "PODMAN_TIMEOUT: $PODMAN_TIMEOUT"
}

# Build execution functions
execute_native_build() {
    local arch="$1"
    local container_engine="$2"

    log_step "Executing native build for $arch..."

    # Use existing cargo-make infrastructure
    if command -v cargo >/dev/null 2>&1 && command -v cargo-make >/dev/null 2>&1; then
        log_info "Using cargo-make for native build"
        export TAG="$BUILD_TAG"
        cargo make build-image
    else
        log_warn "cargo-make not available, using direct container build"
        execute_direct_build "$arch" "$container_engine" "native"
    fi
}

execute_emulated_build() {
    local arch="$1"
    local container_engine="$2"

    log_step "Executing emulated build for $arch..."

    # Use direct container build with emulation-specific flags
    execute_direct_build "$arch" "$container_engine" "emulated"
}

execute_direct_build() {
    local arch="$1"
    local container_engine="$2"
    local build_type="$3"

    log_verbose "Direct build: $arch ($build_type)"

    # Verify Dockerfile exists
    if [ ! -f "rc-web/Dockerfile" ]; then
        log_error "Dockerfile not found at rc-web/Dockerfile"
        exit 1
    fi

    # Determine platform
    local platform="linux/$arch"

    # Set build arguments based on build type
    local build_args=""
    if [[ "$build_type" == "emulated" && "$arch" == "arm64" ]]; then
        build_args="--build-arg RING_DISABLE_ASSEMBLY=1 --build-arg RING_PREGENERATE_ASM=0 --build-arg RUSTFLAGS='-C opt-level=1' --build-arg CARGO_BUILD_JOBS=1"
    fi

    # Build the image
    local image_name="ghcr.io/daksha-rc/rc-web:${BUILD_TAG}-${arch}"

    log_info "Building image: $image_name"
    log_verbose "Platform: $platform"
    log_verbose "Build args: $build_args"

    if [[ "$container_engine" == "podman" ]]; then
        # Use timeout to prevent hangs
        if timeout "$PODMAN_TIMEOUT" podman build \
            --arch "$platform" \
            --memory=4g \
            $build_args \
            -f "rc-web/Dockerfile" \
            -t "$image_name" \
            --label "org.opencontainers.image.version=${BUILD_TAG}" \
            --label "org.opencontainers.image.revision=$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')" \
            --label "org.opencontainers.image.created=$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
            --label "org.opencontainers.image.source=https://github.com/daksha-rc/daksha-rc" \
            --label "org.opencontainers.image.description=Daksha RC Web Application" \
            --label "org.opencontainers.image.platform=$arch" \
            --squash-all \
            --rm \
            .; then
            log_info "Successfully built $arch image"
        else
            log_error "Failed to build $arch image"
            return 1
        fi
    else
        log_error "Docker builds not yet implemented in this script"
        return 1
    fi
}

# Main execution functions
execute_build_command() {
    local command="$1"
    local strategy_map="$2"
    local container_engine="$3"

    log_step "Executing build command: $command"

    case "$command" in
        "build-single")
            # Build for current platform only
            local host_arch=$(detect_host_arch)
            if [[ "$strategy_map" == *"$host_arch:native"* ]]; then
                execute_native_build "$host_arch" "$container_engine"
            elif [[ "$strategy_map" == *"$host_arch:emulated"* ]]; then
                execute_emulated_build "$host_arch" "$container_engine"
            else
                log_error "No build strategy found for host architecture: $host_arch"
                exit 1
            fi
            ;;
        "build-multi")
            # Build for all platforms in strategy map
            local success=true
            while read -r entry; do
                if [[ -n "$entry" ]]; then
                    local arch=$(echo "$entry" | cut -d':' -f1)
                    local strategy=$(echo "$entry" | cut -d':' -f2)

                    if [[ "$strategy" == "native" ]]; then
                        execute_native_build "$arch" "$container_engine" || success=false
                    elif [[ "$strategy" == "emulated" ]]; then
                        execute_emulated_build "$arch" "$container_engine" || success=false
                    fi
                fi
            done <<< "${strategy_map// /$'\n'}"

            if [[ "$success" != "true" ]]; then
                log_error "One or more builds failed"
                exit 1
            fi
            ;;
        "build-and-push"|"build-and-push-all")
            # Build and push (delegate to cargo-make if available)
            if command -v cargo-make >/dev/null 2>&1; then
                export TAG="$BUILD_TAG"
                if [[ "$command" == "build-and-push" ]]; then
                    cargo make build-and-push
                else
                    cargo make build-and-push-all
                fi
            else
                log_error "cargo-make required for push operations"
                exit 1
            fi
            ;;
        "detect-only")
            # Just show detection results
            log_info "Detection complete - no build executed"
            ;;
        *)
            log_error "Unknown command: $command"
            exit 1
            ;;
    esac
}

# Main function
main() {
    local command="build-single"

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -s|--strategy)
                BUILD_STRATEGY="$2"
                shift 2
                ;;
            -p|--platforms)
                TARGET_PLATFORMS="$2"
                shift 2
                ;;
            -f|--force-emulation)
                FORCE_EMULATION=true
                shift
                ;;
            -n|--native-only)
                NATIVE_ONLY=true
                shift
                ;;
            -v|--verbose)
                VERBOSE=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            build-single|build-multi|build-and-push|build-and-push-all|detect-only)
                command="$1"
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    # Display header
    echo -e "${CYAN}🏗️  Smart Multi-Platform Build Script v${SCRIPT_VERSION}${NC}"
    echo -e "${CYAN}================================================${NC}"
    echo

    # System detection
    log_step "Detecting system capabilities..."

    local host_arch=$(detect_host_arch)
    local container_engine=$(detect_container_engine)
    local qemu_support=$(check_qemu_support "$container_engine")

    log_info "Host architecture: $host_arch"
    log_info "Container engine: $container_engine"
    log_info "QEMU support: $qemu_support"

    # Validate system requirements
    if [[ "$container_engine" == "none" ]]; then
        log_error "No container engine found. Please install podman or docker."
        exit 1
    fi

    if [[ "$host_arch" == "unknown" ]]; then
        log_error "Unknown host architecture: $(uname -m)"
        exit 1
    fi

    # Determine build strategy
    local strategy_map=$(determine_build_strategy "$host_arch" "$TARGET_PLATFORMS" "$FORCE_EMULATION" "$NATIVE_ONLY")

    if [[ -z "$strategy_map" ]]; then
        log_error "No valid build strategy could be determined"
        exit 1
    fi

    # Display build plan
    echo
    log_step "Build plan:"
    while read -r entry; do
        if [[ -n "$entry" ]]; then
            local arch=$(echo "$entry" | cut -d':' -f1)
            local strategy=$(echo "$entry" | cut -d':' -f2)
            local icon="🏗️"
            [[ "$strategy" == "native" ]] && icon="⚡"
            [[ "$strategy" == "emulated" ]] && icon="🔄"
            log_info "$icon $arch: $strategy build"
        fi
    done <<< "${strategy_map// /$'\n'}"
    echo

    # Exit early if detect-only
    if [[ "$command" == "detect-only" ]]; then
        log_info "Detection complete"
        exit 0
    fi

    # Setup environment
    setup_build_environment "$container_engine" "$qemu_support"

    # Execute build
    execute_build_command "$command" "$strategy_map" "$container_engine"

    # Success
    echo
    log_info "Build completed successfully! 🎉"
    log_info "Command executed: $command"
    log_info "Tag used: $BUILD_TAG"
    echo
    log_info "You can now use the built images:"
    while read -r entry; do
        if [[ -n "$entry" ]]; then
            local arch=$(echo "$entry" | cut -d':' -f1)
            echo -e "  ${GREEN}podman pull ghcr.io/daksha-rc/rc-web:${BUILD_TAG}-${arch}${NC}"
        fi
    done <<< "${strategy_map// /$'\n'}"
}

# Execute main function
main "$@"
