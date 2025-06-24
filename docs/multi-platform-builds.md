# Multi-Platform Container Builds with Podman

This document describes the multi-platform container build system for the Daksha RC project, which uses Podman to build container images that support both AMD64 and ARM64 architectures.

## Overview

The build system has been refactored to:
- Use **Podman exclusively** (Docker support removed)
- Build **multi-platform container images** for `linux/amd64` and `linux/arm64`
- Use **Podman manifests** for cross-platform compatibility
- Provide **optimized build workflows** for different disk space scenarios

## Architecture Support

All container images are built to support:
- **linux/amd64** - Intel/AMD 64-bit processors
- **linux/arm64** - ARM 64-bit processors (including Apple Silicon)

## Prerequisites

### System Requirements
- macOS or Linux
- Podman installed and configured
- At least 20GB available disk space for multi-platform builds

### Installing Podman
```bash
# macOS
brew install podman

# Initialize and start Podman machine
podman machine init
podman machine start
```

## Build Commands

### Basic Build Workflow

1. **Build multi-platform images:**
   ```bash
   cargo make build-image
   ```

2. **Build single-platform images:**
   ```bash
   # AMD64 only (Intel/AMD processors)
   cargo make build-image-amd64
   
   # ARM64 only (Apple Silicon, ARM servers)
   cargo make build-image-arm64
   ```

3. **Push to registry:**
   ```bash
   cargo make push-image
   ```

4. **Build and push in one command:**
   ```bash
   # Multi-platform
   cargo make build-and-push
   
   # Single platform
   cargo make build-and-push-amd64
   cargo make build-and-push-arm64
   ```
</text>

<old_text>
3. **Manual cleanup:**
   ```bash
   cargo make clean-build-cache
   ```

### Space-Optimized Workflows

For environments with limited disk space:

1. **Build with cleanup:**
   ```bash
   cargo make build-image-clean
   ```

2. **Build and push with aggressive cleanup:**
   ```bash
   cargo make build-and-push-clean
   ```

3. **Manual cleanup:**
   ```bash
   cargo make clean-build-cache
   ```

## Generated Container Tags

Each build creates three manifest tags:

1. **Git Tag**: `ghcr.io/daksha-rc/rc-web:v1.2.3`
2. **Git SHA**: `ghcr.io/daksha-rc/rc-web:abc1234`
3. **Latest**: `ghcr.io/daksha-rc/rc-web:latest`

Each manifest contains images for the built platforms:
- **Multi-platform builds**: Both `linux/amd64` and `linux/arm64`
- **Single-platform builds**: Only the specified platform
- **Failed builds**: Only successfully built platforms (with warnings)

## How Multi-Platform Builds Work

### Podman Manifest System

The build process uses Podman's manifest feature:

```bash
# Build for multiple platforms and create manifest
podman build \
  -f rc-web/Dockerfile \
  --platform linux/amd64,linux/arm64 \
  --manifest ghcr.io/daksha-rc/rc-web:v1.2.3 \
  .
```

### Manifest Structure

Each manifest contains:
- **Platform-specific images** for amd64 and arm64
- **OCI-compliant labels** with metadata
- **Consistent tagging** across platforms

## Disk Space Management

### Monitoring Disk Usage

Check available space before building:
```bash
cargo make check-disk-space
```

This command provides:
- Host system disk usage
- Podman VM disk usage
- Container storage analysis
- Build recommendations

### Space Requirements

| Build Type | Minimum Space | Recommended | Use Case |
|------------|---------------|-------------|----------|
| Single platform (AMD64) | 8GB | 12GB | Development, CI |
| Single platform (ARM64) | 10GB | 15GB | Apple Silicon, ARM servers |
| Multi-platform | 15GB | 20GB | Production releases |
| With cleanup | 6GB | 10GB | Resource-constrained environments |

### Cleanup Strategies

1. **Automatic cleanup** (runs after builds):
   - Removes intermediate build layers
   - Prunes unused images
   - Cleans up dangling manifests

2. **Manual cleanup**:
   ```bash
   cargo make clean-build-cache
   ```

3. **Emergency cleanup**:
   ```bash
   podman system prune -af
   ```

## Troubleshooting

### Common Issues

#### "No space left on device"
```bash
# Check available space
cargo make check-disk-space

# Clean up aggressively
cargo make clean-build-cache

# Use space-optimized build
cargo make build-image-clean

# Fallback to single platform
cargo make build-image-amd64
```

#### "ARM64 build timeout or failure"
```bash
# Build AMD64 only
cargo make build-image-amd64

# Or use environment variable
BUILD_PLATFORMS="linux/amd64" cargo make build-image

# Check ARM64 emulation
podman run --rm --platform=linux/arm64 alpine uname -m
```

#### "Manifest not found"
```bash
# Rebuild the images
cargo make build-image

# Check manifest status
podman manifest inspect ghcr.io/daksha-rc/rc-web:latest

# List all manifests
podman images --filter reference="ghcr.io/daksha-rc/rc-web"
```

#### "Platform not supported"
```bash
# Check Podman version (requires >= 4.0)
podman --version

# Verify QEMU emulation is available
podman run --rm --privileged multiarch/qemu-user-static --reset -p yes

# Fallback to native platform only
cargo make build-image-amd64
```

### Verifying Builds

1. **Check local manifests:**
   ```bash
   podman manifest ls
   ```

2. **Inspect manifest details:**
   ```bash
   podman manifest inspect ghcr.io/daksha-rc/rc-web:latest
   ```

3. **Verify platforms:**
   ```bash
   podman manifest inspect ghcr.io/daksha-rc/rc-web:latest | jq '.manifests[].platform'
   ```

## CI/CD Integration

### GitHub Actions Example

```yaml
- name: Build and Push Multi-Platform Images
  run: |
    cargo make build-and-push-clean
  env:
    REGISTRY_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Build Matrix Strategy

The build system automatically handles:
- Platform detection
- Manifest creation
- Cross-platform compatibility
- Registry pushing

## Best Practices

### Development Workflow

1. **Local testing:**
   ```bash
   # Quick single-platform build for development
   cargo make build-image-amd64
   
   # Or manual build for current platform
   podman build -f rc-web/Dockerfile -t rc-web:dev .
   ```

2. **Platform-specific testing:**
   ```bash
   # Test AMD64 build
   cargo make build-image-amd64
   
   # Test ARM64 build (if needed)
   cargo make build-image-arm64
   ```

3. **Pre-release builds:**
   ```bash
   # Full multi-platform build before tagging
   cargo make build-image
   
   # Fallback if multi-platform fails
   cargo make build-image-amd64
   ```

4. **Release workflow:**
   ```bash
   # Tag and build release
   git tag v1.2.3
   
   # Try multi-platform first
   cargo make build-and-push || cargo make build-and-push-amd64
   ```

### Performance Optimization

1. **Use cargo-chef** (already configured in Dockerfile)
2. **Layer caching** with Podman build cache
3. **Manifest reuse** for incremental builds
4. **Cleanup automation** to prevent disk space issues

## Migration from Docker

### Key Changes

| Docker | Podman |
|--------|--------|
| `docker build` | `podman build --manifest` |
| `docker push` | `podman manifest push` |
| `docker buildx` | Built-in multi-platform support |
| BuildKit cache | Podman build cache |
| `docker build --platform` | `BUILD_PLATFORMS` environment variable |

### Available Build Tasks

| Task | Platform | Use Case |
|------|----------|----------|
| `build-image` | AMD64 + ARM64 | Production releases |
| `build-image-amd64` | AMD64 only | Development, CI |
| `build-image-arm64` | ARM64 only | Apple Silicon testing |
| `build-and-push` | Multi-platform | Full deployment |
| `build-and-push-amd64` | AMD64 only | Quick deployment |

### Removed Dependencies

- Docker Desktop
- Docker BuildX
- Docker Compose (for builds)

### Benefits

- **Native multi-platform** support
- **Rootless containers** by default
- **Better resource management**
- **OCI compliance**
- **No daemon required**

## Monitoring and Metrics

### Build Performance

Track build times and resource usage:
```bash
# Time the build process
time cargo make build-image

# Monitor resource usage
cargo make check-disk-space
```

### Registry Usage

Monitor pushed manifest sizes:
```bash
# Check manifest details
podman manifest inspect ghcr.io/daksha-rc/rc-web:latest

# View platform-specific sizes
podman images ghcr.io/daksha-rc/rc-web --format "table {{.Repository}}:{{.Tag}}\t{{.Size}}\t{{.ID}}"
```

## Support

For issues with the multi-platform build system:

1. Check disk space: `cargo make check-disk-space`
2. Review build logs for platform-specific errors
3. Verify Podman configuration: `podman info`
4. Clean and rebuild: `cargo make build-and-push-clean`

For questions or improvements, please open an issue in the project repository.