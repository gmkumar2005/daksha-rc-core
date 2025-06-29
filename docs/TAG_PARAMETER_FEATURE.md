# TAG Parameter Feature Summary

## Overview

The TAG parameter feature allows you to specify custom version tags for container image builds, overriding the default git tag behavior. This provides flexibility for CI/CD pipelines, development workflows, and custom versioning schemes.

## How It Works

### Default Behavior (No TAG specified)
- Uses the latest git tag from `git describe --tags --abbrev=0`
- Falls back to `v0.0.0-dev` if no git tags exist
- Maintains backward compatibility with existing workflows

### Custom TAG Behavior
- When `TAG` environment variable is set, it overrides git tag detection
- All build scripts (`build-image.sh`, `build-image-all.sh`, `push-image.sh`) respect the TAG parameter
- Consistent tag usage across the entire build and push workflow

## Usage Patterns

### Environment Variable Syntax
```bash
# Single platform build
TAG=v1.0.0 cargo make build-image

# Multi-platform build
TAG=v2.0.0-beta cargo make build-image-all

# Build and push workflow
TAG=v1.5.0 cargo make build-and-push-all
```

### Cargo-make --env Flag Syntax
```bash
# Alternative syntax using --env flag
cargo make build-with-tag --env TAG=v1.0.0
cargo make build-all-with-tag --env TAG=v2.0.0-rc1
cargo make build-and-push-all-with-tag --env TAG=v1.0.0
```

## Image Tag Generation

When you specify `TAG=v1.2.3`, the following images are created:

### Platform-Specific Images
- `ghcr.io/daksha-rc/rc-web:v1.2.3-amd64`
- `ghcr.io/daksha-rc/rc-web:v1.2.3-arm64`
- `ghcr.io/daksha-rc/rc-web:latest-amd64`
- `ghcr.io/daksha-rc/rc-web:latest-arm64`

### Multi-Platform Manifests (from build-image-all)
- `ghcr.io/daksha-rc/rc-web:v1.2.3` (multi-platform manifest)
- `ghcr.io/daksha-rc/rc-web:latest` (multi-platform manifest)
- `ghcr.io/daksha-rc/rc-web:{commit-sha}` (commit-based tag)

## Use Cases

### Development Workflows
```bash
# Feature branch builds
TAG=feature-auth-$(git rev-parse --short HEAD) cargo make build-image

# Daily development builds
TAG=dev-$(date +%Y%m%d) cargo make build-image

# Personal development tags
TAG=dev-john-$(date +%H%M) cargo make build-image
```

### Release Management
```bash
# Release candidates
TAG=v1.0.0-rc.1 cargo make build-image-all

# Beta releases
TAG=v2.0.0-beta.3 cargo make build-and-push-all

# Production releases
TAG=v1.0.0 cargo make build-and-push-all
```

### CI/CD Integration

#### GitHub Actions
```yaml
- name: Build and Push
  env:
    TAG: ${{ github.ref_name }}
  run: cargo make build-and-push-all
```

#### GitLab CI
```yaml
build:
  script:
    - TAG=${CI_COMMIT_TAG:-${CI_COMMIT_SHORT_SHA}} cargo make build-and-push-all
```

#### Jenkins
```groovy
environment {
    TAG = "${BUILD_TAG}"
}
steps {
    sh 'cargo make build-and-push-all-clean'
}
```

## Supported Tasks

### Core Tasks (All support TAG parameter)
- `build-image` - Single platform build
- `build-image-all` - Multi-platform build
- `push-image` - Push images/manifests
- `build-and-push` - Build and push single platform
- `build-and-push-all` - Build and push multi-platform
- `build-and-push-clean` - Build and push with cleanup
- `build-and-push-all-clean` - Multi-platform build and push with cleanup

### Alternative Syntax Tasks
- `build-with-tag --env TAG=version`
- `build-all-with-tag --env TAG=version`
- `push-with-tag --env TAG=version`
- `build-and-push-with-tag --env TAG=version`
- `build-and-push-all-with-tag --env TAG=version`

## Technical Implementation

### Script Changes
1. **build-image.sh** - Added TAG parameter detection with git tag fallback
2. **build-image-all.sh** - Added TAG parameter detection with git tag fallback
3. **push-image.sh** - Added TAG parameter detection with git tag fallback

### Tag Resolution Logic
```bash
# Get tag from parameter or fallback to git tag
if [ -n "$TAG" ]; then
    GIT_TAG="$TAG"
    echo "Using provided tag: ${GIT_TAG}"
elif GIT_TAG=$(git describe --tags --abbrev=0 2>/dev/null); then
    echo "Using Git tag: ${GIT_TAG}"
else
    GIT_TAG="v0.0.0-dev"
    echo "No Git tags found, using default: ${GIT_TAG}"
fi
```

## Benefits

### Flexibility
- Override git tags when needed
- Support custom versioning schemes
- Enable development and feature branch builds

### CI/CD Integration
- Easy integration with various CI/CD systems
- Support for environment variable injection
- Consistent behavior across different platforms

### Development Workflow
- Test specific versions without creating git tags
- Build feature branches with descriptive names
- Create development builds with timestamps

### Backward Compatibility
- Existing workflows continue to work unchanged
- No breaking changes to existing commands
- Graceful fallback to git tag behavior

## Examples and Help

Run `cargo make build-examples` to see comprehensive usage examples and best practices.

## Best Practices

### Tag Naming Conventions
- **Production**: `v1.0.0`, `v2.1.3` (semantic versioning)
- **Release Candidates**: `v1.0.0-rc.1`, `v2.0.0-rc.2`
- **Beta/Alpha**: `v1.0.0-beta.1`, `v2.0.0-alpha.3`
- **Development**: `dev-20240624`, `dev-feature-name`
- **Feature Branches**: `feature-auth-123abc`, `bugfix-login-456def`

### CI/CD Integration Tips
- Use environment variables from your CI/CD system
- Fallback to commit SHA when no tag is available
- Use cleanup variants in resource-constrained environments
- Consider multi-platform builds for production releases

### Development Workflow
- Use descriptive development tags during feature development
- Include timestamps or commit hashes for uniqueness
- Test with custom tags before creating official git tags
- Use single-platform builds for faster development cycles

This feature provides powerful flexibility while maintaining the simplicity and backward compatibility of the existing build system.