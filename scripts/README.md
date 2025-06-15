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

### Kubernetes Scripts
- **`setup-kind-cluster.sh`** - Creates Kind cluster and installs Traefik ingress controller in traefik-system namespace
- **`deploy-demo-apps.sh`** - Deploys httpbin and whoami demo applications to Kind cluster
- **`copy-tls-cert.sh`** - Helper script to copy TLS certificates between Kubernetes namespaces

### Database Scripts
- **`install-cnpg.sh`** - Installs CloudNativePG (CNPG) CRDs and waits for availability

### Application Scripts
- **`deploy-rc-app.sh`** - Deploys rc-app using Helm with health and readiness checks

## Usage

These scripts are automatically executed by their corresponding `cargo make` tasks defined in `Makefile.toml`:

```bash
# Examples
cargo make install-kubectl    # Runs appropriate kubectl installer for your OS
cargo make setup-kind-cluster # Creates Kind cluster with Traefik in traefik-system namespace
cargo make deploy-demo-apps   # Deploys httpbin and whoami apps
cargo make install-cnpg       # Installs CloudNativePG CRDs
cargo make deploy-rc-app      # Deploys rc-app with health checks
cargo make kind-demo          # Complete Kind cluster setup with demo apps
cargo make full-demo          # Complete demo environment setup
cargo make get-git-tag       # Gets latest Git tag
cargo make clean-build-cache # Cleans container cache
cargo make build-image       # Builds Docker images
cargo make push-image        # Pushes images to registry
cargo make check-disk-space  # Analyzes disk usage
```

## Script Features

- **Cross-platform support** - Platform-specific scripts for Linux and macOS
- **Container engine detection** - Automatic detection and support for Docker and Podman
- **Kubernetes cluster management** - Automated Kind cluster setup with Traefik ingress in dedicated namespace
- **Database management** - CloudNativePG installation and configuration
- **Application deployment** - Helm-based rc-app deployment with health checks
- **TLS certificate management** - Automatic wildcard certificate creation and distribution
- **Demo environment** - Complete demo setup with all components
- **Colored output** - Enhanced readability with color-coded console output
- **Error handling** - Comprehensive error checking and user feedback
- **Space optimization** - Built-in disk space monitoring and cleanup capabilities

## File Permissions

All shell scripts (`.sh` files) have been made executable. If you need to manually set permissions:

```bash
chmod +x scripts/*.sh
```

## Kubernetes Architecture

The Kubernetes setup creates a well-organized namespace structure:

- **`traefik-system`** - Traefik ingress controller and dashboard
- **`cnpg-system`** - CloudNativePG operator
- **`whoami`** - whoami demo application
- **`httpbin`** - httpbin demo application
- **`default`** - rc-app application

Each namespace contains its own TLS certificates and middleware resources for proper isolation.

## Customization

You can modify these scripts directly to customize behavior for your specific environment. The main `Makefile.toml` will automatically use your updated scripts without requiring additional configuration changes.