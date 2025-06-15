# Scripts Directory

This directory contains shell scripts that have been extracted from the main `Makefile.toml` for better organization and maintainability.

## Scripts Overview

### Kubectl Installation Scripts
- **`install-kubectl-linux.sh`** - Installs kubectl on Linux systems
- **`install-kubectl-mac.sh`** - Installs kubectl on macOS systems (supports both Intel and Apple Silicon)

### Git Management Scripts
- **`get-git-tag.sh`** - Retrieves the latest Git tag for versioning

### Docker Build Scripts
- **`clean-build-cache.sh`** - Cleans container build cache to free disk space
- **`build-image.sh`** - Builds Docker images with Git tag versioning
- **`push-image.sh`** - Pushes Docker images to container registry

### System Monitoring Scripts
- **`check-disk-space.sh`** - Comprehensive disk space monitoring for host and container environments

## Usage

These scripts are automatically executed by their corresponding `cargo make` tasks defined in `Makefile.toml`:

```bash
# Examples
cargo make install-kubectl    # Runs appropriate kubectl installer for your OS
cargo make get-git-tag       # Gets latest Git tag
cargo make clean-build-cache # Cleans container cache
cargo make build-image       # Builds Docker images
cargo make push-image        # Pushes images to registry
cargo make check-disk-space  # Analyzes disk usage
```

## Script Features

- **Cross-platform support** - Platform-specific scripts for Linux and macOS
- **Container engine detection** - Automatic detection and support for Docker and Podman
- **Colored output** - Enhanced readability with color-coded console output
- **Error handling** - Comprehensive error checking and user feedback
- **Space optimization** - Built-in disk space monitoring and cleanup capabilities

## File Permissions

All shell scripts (`.sh` files) have been made executable. If you need to manually set permissions:

```bash
chmod +x scripts/*.sh
```

## Customization

You can modify these scripts directly to customize behavior for your specific environment. The main `Makefile.toml` will automatically use your updated scripts without requiring additional configuration changes.
