#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}🔍 Daksha RC Web Build Debugging Script${NC}"
echo -e "${CYAN}=====================================${NC}"
echo

# Function to run a command and show its output
run_debug_command() {
    local description="$1"
    local command="$2"

    echo -e "${YELLOW}🔧 ${description}${NC}"
    echo -e "${BLUE}   Command: ${command}${NC}"

    if eval "$command"; then
        echo -e "${GREEN}   ✅ Success${NC}"
    else
        local exit_code=$?
        echo -e "${RED}   ❌ Failed with exit code: ${exit_code}${NC}"
    fi
    echo
}

# Function to check if a file exists and show its info
check_file() {
    local file="$1"
    local description="$2"

    echo -e "${YELLOW}📁 Checking ${description}: ${file}${NC}"
    if [ -f "$file" ]; then
        echo -e "${GREEN}   ✅ File exists${NC}"
        echo -e "${BLUE}   Size: $(du -h "$file" | cut -f1)${NC}"
        if [ -x "$file" ]; then
            echo -e "${GREEN}   ✅ File is executable${NC}"
        else
            echo -e "${YELLOW}   ⚠️  File is not executable${NC}"
        fi
    else
        echo -e "${RED}   ❌ File does not exist${NC}"
    fi
    echo
}

# Section 1: Environment Information
echo -e "${CYAN}📋 Section 1: Environment Information${NC}"
echo -e "${CYAN}====================================${NC}"

echo -e "${YELLOW}🖥️  System Information:${NC}"
echo -e "   OS: $(uname -s)"
echo -e "   Architecture: $(uname -m)"
echo -e "   Kernel: $(uname -r)"
echo -e "   Hostname: $(hostname)"
echo -e "   User: $(whoami)"
echo -e "   Working Directory: $(pwd)"
echo

echo -e "${YELLOW}💾 Disk Space Information:${NC}"
df -h
echo

echo -e "${YELLOW}🧠 Memory Information:${NC}"
free -h
echo

# Section 2: Tool Availability
echo -e "${CYAN}📋 Section 2: Tool Availability${NC}"
echo -e "${CYAN}==============================${NC}"

tools=("podman" "docker" "cargo" "cargo-make" "git" "bash")
for tool in "${tools[@]}"; do
    echo -e "${YELLOW}🔧 Checking ${tool}:${NC}"
    if command -v "$tool" >/dev/null 2>&1; then
        version=$(${tool} --version 2>/dev/null | head -n1 || echo "Version unknown")
        echo -e "${GREEN}   ✅ Available: ${version}${NC}"
    else
        echo -e "${RED}   ❌ Not available${NC}"
    fi
done
echo

# Section 3: Project Structure
echo -e "${CYAN}📋 Section 3: Project Structure${NC}"
echo -e "${CYAN}==============================${NC}"

important_files=(
    "Cargo.toml:Workspace Cargo.toml"
    "rc-web/Cargo.toml:RC Web Cargo.toml"
    "rc-web/Dockerfile:RC Web Dockerfile"
    "Makefile.toml:Cargo Make configuration"
    "scripts/build-image.sh:Build script"
    "scripts/build-image-all.sh:Multi-platform build script"
    "scripts/get-platform.sh:Platform detection script"
)

for file_info in "${important_files[@]}"; do
    file=$(echo "$file_info" | cut -d':' -f1)
    description=$(echo "$file_info" | cut -d':' -f2)
    check_file "$file" "$description"
done

# Section 4: Git Information
echo -e "${CYAN}📋 Section 4: Git Information${NC}"
echo -e "${CYAN}===========================${NC}"

run_debug_command "Git status" "git status --porcelain"
run_debug_command "Git current branch" "git branch --show-current"
run_debug_command "Git latest commit" "git log -1 --oneline"
run_debug_command "Git tags" "git tag -l | tail -10"
run_debug_command "Git describe tags" "git describe --tags --abbrev=0"

# Section 5: Platform and Tag Detection
echo -e "${CYAN}📋 Section 5: Platform and Tag Detection${NC}"
echo -e "${CYAN}=====================================${NC}"

echo -e "${YELLOW}🏗️  Platform Detection:${NC}"
if [ -f "scripts/get-platform.sh" ]; then
    if platform=$(bash scripts/get-platform.sh 2>&1); then
        echo -e "${GREEN}   ✅ Platform detected: ${platform}${NC}"
    else
        echo -e "${RED}   ❌ Platform detection failed: ${platform}${NC}"
    fi
else
    echo -e "${RED}   ❌ get-platform.sh script not found${NC}"
fi

echo -e "${YELLOW}🏷️  Tag Detection:${NC}"
if [ -n "$TAG" ]; then
    echo -e "${GREEN}   ✅ TAG environment variable set: ${TAG}${NC}"
else
    echo -e "${YELLOW}   ⚠️  TAG environment variable not set${NC}"
fi

if git_tag=$(git describe --tags --abbrev=0 2>/dev/null); then
    echo -e "${GREEN}   ✅ Git tag available: ${git_tag}${NC}"
else
    echo -e "${YELLOW}   ⚠️  No git tags found${NC}"
fi

if git_sha=$(git rev-parse --short HEAD 2>/dev/null); then
    echo -e "${GREEN}   ✅ Git SHA available: ${git_sha}${NC}"
else
    echo -e "${RED}   ❌ Git SHA detection failed${NC}"
fi
echo

# Section 6: Container Images
echo -e "${CYAN}📋 Section 6: Container Images${NC}"
echo -e "${CYAN}===========================${NC}"

echo -e "${YELLOW}🐳 All Container Images:${NC}"
if command -v podman >/dev/null 2>&1; then
    echo -e "${BLUE}   Podman images:${NC}"
    podman images || echo -e "${RED}   Failed to list podman images${NC}"
    echo
fi

if command -v docker >/dev/null 2>&1; then
    echo -e "${BLUE}   Docker images:${NC}"
    docker images || echo -e "${RED}   Failed to list docker images${NC}"
    echo
fi

echo -e "${YELLOW}🔍 RC Web Related Images:${NC}"
if command -v podman >/dev/null 2>&1; then
    echo -e "${BLUE}   RC Web images (podman):${NC}"
    podman images | grep -E "(rc-web|daksha)" || echo -e "${YELLOW}   No RC Web images found${NC}"
    echo
fi

# Section 7: Environment Variables
echo -e "${CYAN}📋 Section 7: Environment Variables${NC}"
echo -e "${CYAN}================================${NC}"

important_vars=(
    "TAG"
    "SQLX_OFFLINE"
    "CARGO_TERM_COLOR"
    "RUST_LOG"
    "RUST_BACKTRACE"
    "HOME"
    "USER"
    "PATH"
)

for var in "${important_vars[@]}"; do
    if [ -n "${!var}" ]; then
        echo -e "${GREEN}   ✅ ${var}=${!var}${NC}"
    else
        echo -e "${YELLOW}   ⚠️  ${var} not set${NC}"
    fi
done
echo

# Section 8: Cargo Information
echo -e "${CYAN}📋 Section 8: Cargo Information${NC}"
echo -e "${CYAN}===========================${NC}"

if command -v cargo >/dev/null 2>&1; then
    run_debug_command "Cargo version" "cargo --version"
    run_debug_command "Rust toolchain" "rustc --version"
    run_debug_command "Cargo make version" "cargo make --version"

    echo -e "${YELLOW}📦 Cargo Make Tasks:${NC}"
    if cargo make --list-all-steps 2>/dev/null | grep -E "(build-|image)" | head -20; then
        echo -e "${GREEN}   ✅ Cargo make tasks listed above${NC}"
    else
        echo -e "${RED}   ❌ Failed to list cargo make tasks${NC}"
    fi
    echo
fi

# Section 9: Build Dependencies
echo -e "${CYAN}📋 Section 9: Build Dependencies${NC}"
echo -e "${CYAN}==============================${NC}"

echo -e "${YELLOW}🔗 Cargo Dependencies Check:${NC}"
if [ -f "Cargo.lock" ]; then
    echo -e "${GREEN}   ✅ Cargo.lock exists${NC}"
    ring_version=$(grep -A 1 "name = \"ring\"" Cargo.lock | grep "version" | head -1 | sed 's/.*"\(.*\)".*/\1/' || echo "not found")
    echo -e "${BLUE}   Ring crate version: ${ring_version}${NC}"
else
    echo -e "${RED}   ❌ Cargo.lock not found${NC}"
fi

echo -e "${YELLOW}🏗️  Build Tool Dependencies:${NC}"
build_deps=("cc" "cmake" "make" "pkg-config")
for dep in "${build_deps[@]}"; do
    if command -v "$dep" >/dev/null 2>&1; then
        echo -e "${GREEN}   ✅ ${dep} available${NC}"
    else
        echo -e "${YELLOW}   ⚠️  ${dep} not found${NC}"
    fi
done
echo

# Section 10: Test Build Commands
echo -e "${CYAN}📋 Section 10: Test Build Commands${NC}"
echo -e "${CYAN}===============================${NC}"

echo -e "${YELLOW}🧪 Testing Basic Commands:${NC}"
run_debug_command "Test platform detection" "bash scripts/get-platform.sh"
run_debug_command "Test git tag detection" "git describe --tags --abbrev=0"
run_debug_command "Test podman connectivity" "podman version"

# Section 11: Dockerfile Analysis
echo -e "${CYAN}📋 Section 11: Dockerfile Analysis${NC}"
echo -e "${CYAN}===============================${NC}"

if [ -f "rc-web/Dockerfile" ]; then
    echo -e "${YELLOW}📋 Dockerfile Information:${NC}"
    echo -e "${GREEN}   ✅ Dockerfile exists${NC}"
    echo -e "${BLUE}   Lines: $(wc -l < rc-web/Dockerfile)${NC}"
    echo -e "${BLUE}   Base images used:${NC}"
    grep -E "^FROM" rc-web/Dockerfile | sed 's/^/      /' || echo "      None found"
    echo
else
    echo -e "${RED}   ❌ Dockerfile not found at rc-web/Dockerfile${NC}"
fi

# Section 12: Recent Build Artifacts
echo -e "${CYAN}📋 Section 12: Recent Build Artifacts${NC}"
echo -e "${CYAN}===================================${NC}"

echo -e "${YELLOW}🔍 Looking for recent build artifacts:${NC}"
if [ -d "target" ]; then
    echo -e "${GREEN}   ✅ Target directory exists${NC}"
    echo -e "${BLUE}   Size: $(du -sh target | cut -f1)${NC}"
    echo -e "${BLUE}   Recent files:${NC}"
    find target -name "rc-web" -type f -mtime -1 2>/dev/null | head -5 | sed 's/^/      /' || echo "      No recent rc-web binaries found"
else
    echo -e "${YELLOW}   ⚠️  Target directory not found${NC}"
fi
echo

# Section 13: Troubleshooting Recommendations
echo -e "${CYAN}📋 Section 13: Troubleshooting Recommendations${NC}"
echo -e "${CYAN}===========================================${NC}"

echo -e "${YELLOW}💡 Based on the analysis above, here are some recommendations:${NC}"
echo

# Check for common issues
has_podman=$(command -v podman >/dev/null 2>&1 && echo "yes" || echo "no")
has_dockerfile=$([ -f "rc-web/Dockerfile" ] && echo "yes" || echo "no")
has_cargo_make=$(command -v cargo-make >/dev/null 2>&1 && echo "yes" || echo "no")
has_platform_script=$([ -f "scripts/get-platform.sh" ] && echo "yes" || echo "no")

if [ "$has_podman" = "no" ]; then
    echo -e "${RED}❌ Podman is not available${NC}"
    echo -e "${YELLOW}   Solution: Install podman${NC}"
    echo -e "${BLUE}   Command: sudo apt-get install podman (Ubuntu/Debian)${NC}"
    echo -e "${BLUE}   Command: brew install podman (macOS)${NC}"
    echo
fi

if [ "$has_dockerfile" = "no" ]; then
    echo -e "${RED}❌ Dockerfile is missing${NC}"
    echo -e "${YELLOW}   Solution: Ensure you're in the correct directory${NC}"
    echo -e "${BLUE}   Expected location: rc-web/Dockerfile${NC}"
    echo
fi

if [ "$has_cargo_make" = "no" ]; then
    echo -e "${RED}❌ cargo-make is not available${NC}"
    echo -e "${YELLOW}   Solution: Install cargo-make${NC}"
    echo -e "${BLUE}   Command: cargo install cargo-make${NC}"
    echo
fi

if [ "$has_platform_script" = "no" ]; then
    echo -e "${RED}❌ Platform detection script is missing${NC}"
    echo -e "${YELLOW}   Solution: Ensure scripts/get-platform.sh exists and is executable${NC}"
    echo -e "${BLUE}   Command: chmod +x scripts/get-platform.sh${NC}"
    echo
fi

# Check disk space
available_gb=$(df / | awk 'NR==2 {print int($4/1024/1024)}')
if [ "$available_gb" -lt 10 ]; then
    echo -e "${RED}❌ Low disk space: ${available_gb}GB available${NC}"
    echo -e "${YELLOW}   Solution: Free up disk space or run cleanup${NC}"
    echo -e "${BLUE}   Command: cargo make clean-build-cache${NC}"
    echo -e "${BLUE}   Command: podman system prune -a${NC}"
    echo
fi

echo -e "${GREEN}✅ Debug analysis complete!${NC}"
echo
echo -e "${CYAN}🔧 Next Steps:${NC}"
echo -e "${YELLOW}1. Address any red (❌) issues shown above${NC}"
echo -e "${YELLOW}2. If all looks good, try running the build command again${NC}"
echo -e "${YELLOW}3. If the build still fails, check the specific error messages${NC}"
echo -e "${YELLOW}4. Consider running: cargo make clean && cargo make build-image${NC}"
echo
echo -e "${BLUE}📝 Save this output for troubleshooting:${NC}"
echo -e "${BLUE}   ./scripts/debug-build.sh > debug_output.txt 2>&1${NC}"
