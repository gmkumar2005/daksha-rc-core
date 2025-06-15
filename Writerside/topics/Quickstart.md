# Quickstart Guide: Deploy daksha-rc-core with cargo make

## Overview

This guide provides a step-by-step walkthrough to deploy the complete **daksha-rc-core** ecosystem using `cargo make` commands. You'll set up a local Kubernetes cluster with Traefik ingress, deploy demo applications, install CloudNativePG, and finally deploy the rc-app with full health monitoring.

## Prerequisites

Before starting, ensure you have the following installed:

### Required Tools
- **Rust toolchain** (version 1.86.0 or later)
  - [Install Rust](https://www.rust-lang.org/tools/install)
- **Git** for version control
  - [Install Git](https://git-scm.com/downloads)
- **Kind** (Kubernetes in Docker)
  - [Install Kind](https://kind.sigs.k8s.io/docs/user/quick-start/#installation)
- **Helm** (Kubernetes package manager)
  - [Install Helm](https://helm.sh/docs/intro/install/)
- **Container Engine**: Docker or Podman
  - [Install Docker](https://docs.docker.com/get-docker/) or [Install Podman](https://podman.io/getting-started/installation)

- **cargo-make** - Rust task runner and build tool
  - [Installation Guide](https://github.com/sagiegurari/cargo-make#installation)
  - [Documentation](https://sagiegurari.github.io/cargo-make/)
- **kubectl** - Kubernetes command-line tool
  - [Installation Guide](https://kubernetes.io/docs/tasks/tools/install-kubectl/)
  - [Documentation](https://kubernetes.io/docs/reference/kubectl/)

## Quick Setup Commands

For the impatient, run these commands in sequence:

```bash
# Clone and setup
git clone https://github.com/Daksha-RC/daksha-rc-core.git
cd daksha-rc-core

# Complete deployment (one command does it all)
cargo make full-demo
```

## Step-by-Step Deployment

### Step 1: Clone the Repository

```bash
git clone https://github.com/Daksha-RC/daksha-rc-core.git
cd daksha-rc-core
```

### Step 2: Install kubectl (if needed)

```bash
cargo make install-kubectl
```

**Expected output:**
- ✅ kubectl installation for your platform (Linux/macOS)
- ✅ Verification that kubectl is working

### Step 3: Setup Kind Cluster with Traefik

```bash
cargo make setup-kind-cluster
```

**What this does:**
- 🏗️ Creates a Kind Kubernetes cluster
- 🚀 Installs Traefik ingress controller in `traefik-system` namespace
- 🔐 Generates wildcard TLS certificates for `*.127.0.0.1.nip.io`
- 🖥️ Sets up Traefik dashboard
- ⏳ Waits for all components to be ready

**Expected output:**
```
✅ Kind cluster with Traefik setup complete!

📋 Cluster Information:
Cluster: kind
Context: kind-kind
Traefik Namespace: traefik-system
Traefik Dashboard: https://dashboard.127.0.0.1.nip.io
```

### Step 4: Deploy Demo Applications

```bash
cargo make deploy-demo-apps
```

**What this does:**
- 🤖 Deploys whoami application in `whoami` namespace
- 🌐 Deploys httpbin application in `httpbin` namespace
- 🔐 Copies TLS certificates to application namespaces
- ⏳ Waits for deployments to be ready
- 🧪 Tests application endpoints

**Expected output:**
```
✅ Demo applications deployment complete!

📋 Application URLs:
• httpbin: https://httpbin.127.0.0.1.nip.io
• whoami:  https://whoami.127.0.0.1.nip.io
• Traefik Dashboard: https://dashboard.127.0.0.1.nip.io
```

### Step 5: Install CloudNativePG

```bash
cargo make install-cnpg
```

**What this does:**
- 📦 Adds CloudNativePG Helm repository
- 🗄️ Installs CNPG operator in `cnpg-system` namespace
- ⏳ Waits for operator to be ready (120s timeout)
- 📊 Shows CNPG status and version

**Expected output:**
```
✅ CloudNativePG (CNPG) is ready!

💡 Next steps:
  • Deploy rc-app: cargo make deploy-rc-app
```

### Step 6: Deploy RC-App

```bash
cargo make deploy-rc-app
```

**What this does:**
- ✅ Validates CNPG is installed and ready
- 🚀 Deploys rc-app using Helm chart from `k8s/rc-app`
- ⏳ Waits for deployment to be available (120s timeout)
- 🧪 Performs comprehensive health checks with retries
- 📊 Shows deployment status and resource information

**Expected output:**
```
✅ rc-app deployment and health checks complete!

📋 Application Information:
• Application URL: https://rc.127.0.0.1.nip.io
• Health endpoint: https://rc.127.0.0.1.nip.io/healthz
• Helm release: dev
• Namespace: default
```

## Alternative: One-Command Full Deployment

For a complete end-to-end deployment, use:

```bash
cargo make full-demo
```

This single command runs all the above steps in sequence:
1. `setup-kind-cluster`
2. `deploy-demo-apps`
3. `install-cnpg`
4. `deploy-rc-app`

## Verification and Testing

### Test All Applications

```bash
# Test whoami
curl -k https://whoami.127.0.0.1.nip.io/

# Test httpbin
curl -k https://httpbin.127.0.0.1.nip.io/get

# Test rc-app health
curl -k https://rc.127.0.0.1.nip.io/healthz

# Access Traefik dashboard
open https://dashboard.127.0.0.1.nip.io
```

### Check Cluster Status

```bash
# View all resources across namespaces
kubectl get all -A

# Check specific namespaces
kubectl get all -n traefik-system
kubectl get all -n whoami
kubectl get all -n httpbin
kubectl get all -n cnpg-system
kubectl get all -n default
```

### Monitor Deployments

```bash
# Watch rc-app pods
kubectl logs -l app.kubernetes.io/instance=dev -f

# Check health endpoint
watch -n 2 "curl -k -s https://rc.127.0.0.1.nip.io/healthz"
```

## Application Architecture

After successful deployment, you'll have:

### Namespaces
- **`traefik-system`** - Traefik ingress controller and dashboard
- **`cnpg-system`** - CloudNativePG operator for PostgreSQL
- **`whoami`** - Demo application showing request details
- **`httpbin`** - HTTP testing and debugging service
- **`default`** - Main rc-app application

### Applications
| Application | URL | Purpose |
|-------------|-----|---------|
| **RC-App** | https://rc.127.0.0.1.nip.io | Main application with health endpoints |
| **Traefik Dashboard** | https://dashboard.127.0.0.1.nip.io | Ingress controller management |
| **whoami** | https://whoami.127.0.0.1.nip.io | Request echo service |
| **httpbin** | https://httpbin.127.0.0.1.nip.io | HTTP testing utilities |

### Health Endpoints
- **RC-App Health**: https://rc.127.0.0.1.nip.io/healthz
- **RC-App Readiness**: https://rc.127.0.0.1.nip.io/readyz

## Troubleshooting

### Common Issues

#### 1. Kind Cluster Creation Fails
```bash
# Check if kind is installed
kind version

# Check if Docker/Podman is running
docker info  # or: podman info
```

#### 2. Traefik Not Ready
```bash
# Check Traefik pods
kubectl get pods -n traefik-system

# Check Traefik logs
kubectl logs -n traefik-system -l app.kubernetes.io/name=traefik
```

#### 3. Applications Not Accessible
```bash
# Check IngressRoutes
kubectl get ingressroute -A

# Check TLS certificates
kubectl get secrets -A | grep tls

# Copy TLS certificates if missing
./scripts/copy-tls-cert.sh whoami httpbin
```

#### 4. RC-App Health Check Fails
```bash
# Check rc-app pod status
kubectl get pods -l app.kubernetes.io/instance=dev

# Check rc-app logs
kubectl logs -l app.kubernetes.io/instance=dev

# Check service and ingress
kubectl get svc,ingressroute -l app.kubernetes.io/instance=dev
```

### Recovery Commands

```bash
# Clean restart
kind delete cluster
cargo make setup-kind-cluster

# Redeploy applications
cargo make deploy-demo-apps
cargo make deploy-rc-app

# Check disk space (if builds fail)
cargo make check-disk-space
cargo make clean-build-cache
```

## Development Workflow

### Building and Testing

```bash
# Build the project
cargo make build

# Run tests
cargo make test

# Build Docker images
cargo make build-image

# Push images (if registry configured)
cargo make push-image
```

### Managing the Demo Environment

```bash
# Start from scratch
cargo make full-demo

# Deploy only infrastructure
cargo make kind-demo

# Deploy only applications
cargo make deploy-demo-apps
cargo make deploy-rc-app

# Clean up everything
kind delete cluster
```

## Next Steps

1. **Explore the API**: Visit https://rc.127.0.0.1.nip.io for API documentation
2. **Check Logs**: Monitor application behavior with `kubectl logs`
3. **Scale Applications**: Modify replica counts in Helm values
4. **Add PostgreSQL**: Use CNPG to create PostgreSQL clusters
5. **Custom Configuration**: Modify `k8s/rc-app/values.yaml` for customization

## Additional Resources

- **Scripts Documentation**: See `scripts/README.md` for detailed script information
- **Kubernetes Manifests**: Explore `k8s/manual/` for resource definitions
- **Helm Charts**: Check `k8s/rc-app/` for application configuration
- **Build Configuration**: Review `Makefile.toml` for available tasks

---

🎉 **Congratulations!** You now have a fully functional daksha-rc-core deployment with monitoring, ingress, and database capabilities.
