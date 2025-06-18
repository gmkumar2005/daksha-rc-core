# Developer Guide

## Overview

This guide provides a step-by-step walkthrough to deploy the complete **daksha-rc-core** ecosystem using `cargo make`
commands. You'll set up a local Kubernetes cluster with Traefik ingress, deploy demo applications, install
CloudNativePG, and finally deploy the rc-app with full health monitoring.

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
    - [Install Docker](https://docs.docker.com/get-docker/)
      or [Install Podman](https://podman.io/getting-started/installation)

- **cargo-make** - Rust task runner and build tool
    - [Installation Guide](https://github.com/sagiegurari/cargo-make#installation)
    - [Documentation](https://sagiegurari.github.io/cargo-make/)
- **kubectl** - Kubernetes command-line tool
    - [Installation Guide](https://kubernetes.io/docs/tasks/tools/install-kubectl/)
    - [Documentation](https://kubernetes.io/docs/reference/kubectl/)

### Tools for debugging

- **mirrord** - Local development with Kubernetes environment
    - [Installation Guide](https://mirrord.dev/docs/overview/quick-start/)
    - [Documentation](https://mirrord.dev/docs/)
    - Required for: `cargo make debug`

## Quick Setup Commands

For the impatient, run these commands in sequence:

```bash
# Clone and setup
git clone https://github.com/Daksha-RC/daksha-rc-core.git
cd daksha-rc-core

# Complete deployment (one command does it all)
cargo make full-demo

# Start debugging (after deployment)
cargo make debug
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

## Database Connection Scripts

The project includes three PostgreSQL connection scripts for different use cases:

### Direct Database Connection (`connect-postgres.sh`)

For direct database access with an interactive psql session:

```bash
# Connect to PostgreSQL database
./scripts/connect-postgres.sh dev
```

**What this script does:**
- ✅ Checks CNPG operator status
- 🔍 Finds PostgreSQL pod using label `cnpg.io/podRole=instance`
- 🔐 Extracts credentials from Kubernetes secret `dev-rc-app-database-app`
- 🚀 Establishes port forwarding to the PostgreSQL pod
- 💻 Launches interactive psql session
- 🧹 Automatically cleans up port forwarding on exit

**Example output:**
```
Connecting to PostgreSQL for release: dev
Using secret: dev-rc-app-database-app
Checking CNPG operator status...
Found PostgreSQL pod: dev-rc-app-database-1 in namespace: default
Database: app
Username: app

Connection URLs:
----------------------------------------
Regular PostgreSQL URL:
postgresql://app:password123@localhost:5432/app

JDBC URL:
jdbc:postgresql://localhost:5432/app?user=app&password=password123

Connection Details:
Host: localhost
Port: 5432
Database: app
Username: app
Password: [hidden - available in environment]
----------------------------------------

Connecting to PostgreSQL database...
You can now run SQL commands. Type \q to exit.
```

### Persistent Port Forwarding (`portforward-postgres.sh`)

For maintaining a persistent database connection without launching psql:

```bash
# Start persistent port forwarding (default port 5432)
./scripts/portforward-postgres.sh dev

# Or use a custom local port
./scripts/portforward-postgres.sh dev 15432
```

**What this script does:**
- 🔄 Maintains persistent port forwarding connection
- 📊 Monitors connection health every 10 seconds
- 🔧 Automatically recovers from connection failures
- 📋 Displays connection URLs for external tools
- ⏳ Runs until manually stopped with Ctrl+C

**Example output:**
```
=========================================
PERSISTENT PORT FORWARDING ACTIVE
=========================================
Connection URLs:
----------------------------------------
Regular PostgreSQL URL:
postgresql://app:password123@localhost:5432/app

JDBC URL:
jdbc:postgresql://localhost:5432/app?user=app&password=password123

Connection Details:
Host: localhost
Port: 5432
Database: app
Username: app
Password: [available in connection URLs above]
----------------------------------------
Port forwarding PID: 12345
=========================================

Press Ctrl+C to stop port forwarding

Monitoring port forwarding... (checking every 10s)
```

### Pod Terminal Access (`shell-postgres.sh`)

For direct access to the PostgreSQL pod terminal with an interactive shell:

```bash
# Connect to PostgreSQL pod terminal
./scripts/shell-postgres.sh dev
```

**What this script does:**
- ✅ Checks CNPG operator status
- 🔍 Finds PostgreSQL pod using label `cnpg.io/podRole=instance`
- 🖥️ Connects directly to the pod terminal with interactive bash
- 🔧 Provides access to all PostgreSQL tools within the pod
- 🧹 No port forwarding or secrets required

**Example output:**
```
=========================================
CONNECTING TO POSTGRESQL POD TERMINAL
=========================================
Pod: dev-rc-app-database-1
Namespace: default
Shell: Interactive bash session
=========================================

You are now connected to the PostgreSQL pod terminal.
Available commands:
  - psql: Connect to PostgreSQL directly
  - pg_dump: Backup database
  - pg_restore: Restore database
  - Standard Linux commands (ls, cat, tail, etc.)

Type 'exit' to leave the pod terminal.
----------------------------------------

postgres@dev-rc-app-database-1:/$
```

### Database Reset Script (`database_reset.sql`)

For resetting the database to a clean state during development and testing:

```bash
# Connect to database and run the reset script
./scripts/connect-postgres.sh dev
# Then in the psql session:
\i /path/to/rc-web/tests/api-tests/database_reset.sql
```

**What this script does:**
- 🗑️ Drops all application tables: `definition`, `definitions`, `event`, `event_listener`, `event_sequence`
- 🔧 Removes custom functions: `event_store_begin_epoch()`, `event_store_current_epoch()`, `notify_event_listener()`
- 📊 Drops sequences: `event_sequence_event_id_seq`
- 🧹 Completely cleans the database schema for fresh starts

**Script contents:**
```sql
drop table definition;
drop table definitions;
drop table event;
drop table event_listener;
drop table event_sequence;
drop function event_store_begin_epoch ();
drop function event_store_current_epoch ();
drop function notify_event_listener ();
drop sequence event_sequence_event_id_seq;
```

**When to use:**
- 🧪 Before running integration tests
- 🔄 When switching between different development branches
- 🐛 When debugging database-related issues
- 🆕 When you need a completely fresh database state

**Alternative execution methods:**
```bash
# Method 1: Direct execution via psql
psql "postgresql://app:password@localhost:5432/app" -f rc-web/tests/api-tests/database_reset.sql

# Method 2: Using port forwarding and external psql
./scripts/portforward-postgres.sh dev &
psql "postgresql://app:password@localhost:5432/app" -f rc-web/tests/api-tests/database_reset.sql

# Method 3: Copy script to pod and execute
kubectl cp rc-web/tests/api-tests/database_reset.sql dev-rc-app-database-1:/tmp/reset.sql
./scripts/shell-postgres.sh dev
# Then in pod: psql -f /tmp/reset.sql
```

### Use Cases

**Use `connect-postgres.sh` when:**
- You need direct SQL access for debugging
- Running database migrations or admin tasks
- Exploring database schema and data
- One-time database operations

**Use `portforward-postgres.sh` when:**
- Connecting external database tools (pgAdmin, DBeaver, etc.)
- Running applications that need database access
- Long-running database connections
- Development with persistent database connectivity

**Use `shell-postgres.sh` when:**
- You need direct access to the PostgreSQL server environment
- Running database administration tasks (pg_dump, pg_restore)
- Debugging PostgreSQL server configuration
- Inspecting pod filesystem and logs
- Performing manual database operations within the pod

**Use `database_reset.sql` when:**
- Starting integration tests that require a clean database
- Switching between development branches with different schemas
- Debugging database-related issues by resetting to a known state
- Clearing test data after development sessions
- Preparing for fresh application deployments

### Connection URLs

Both scripts provide connection URLs in multiple formats:

- **Regular PostgreSQL URL**: `postgresql://username:password@localhost:5432/database`
- **JDBC URL**: `jdbc:postgresql://localhost:5432/database?user=username&password=password`
- **Individual connection details** for manual configuration

These URLs can be used with:
- Database management tools (pgAdmin, DBeaver, TablePlus)
- Application configuration files
- Development environments
- CI/CD pipelines for database operations

## Application Architecture

After successful deployment, you'll have:

### Namespaces

- **`traefik-system`** - Traefik ingress controller and dashboard
- **`cnpg-system`** - CloudNativePG operator for PostgreSQL
- **`whoami`** - Demo application showing request details
- **`httpbin`** - HTTP testing and debugging service
- **`default`** - Main rc-app application

### Applications

| Application           | URL                                | Purpose                                |
|-----------------------|------------------------------------|----------------------------------------|
| **RC-App**            | https://rc.127.0.0.1.nip.io        | Main application with health endpoints |
| **Traefik Dashboard** | https://dashboard.127.0.0.1.nip.io | Ingress controller management          |
| **whoami**            | https://whoami.127.0.0.1.nip.io    | Request echo service                   |
| **httpbin**           | https://httpbin.127.0.0.1.nip.io   | HTTP testing utilities                 |

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

#### 5. Debug Session Issues

```bash
# Ensure mirrord is installed
mirrord --version

# Check if deployment has single replica
kubectl get deployment dev-rc-app -o jsonpath='{.spec.replicas}'

# Scale to single replica if needed
kubectl scale deployment dev-rc-app --replicas=1

# Verify pod is running
kubectl get pods -l app.kubernetes.io/instance=dev
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

# Reset debugging environment
kubectl scale deployment dev-rc-app --replicas=1
cargo make debug
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

### Debugging with mirrord

For advanced debugging and development, you can use `mirrord` to run your local application while connecting to the
Kubernetes cluster environment:

```bash
# Start debug session with mirrord
cargo make debug
```

**What this does:**

- 🔍 Automatically discovers the rc-app pod in the cluster
- ✅ Validates the deployment has exactly one replica
- 🔗 Uses mirrord to mirror traffic from the Kubernetes pod to your local application
- 🐛 Runs the application locally with debug logging (`RUST_LOG=rc_web=debug`)

**Prerequisites for debugging:**

- **mirrord** must be installed: [Installation Guide](https://mirrord.dev/docs/overview/quick-start/)
- RC-app must be deployed: `cargo make deploy-rc-app`
- Deployment should have exactly 1 replica (default configuration)

**Example debug session:**

```bash
$ cargo make debug
🔍 Starting debug session with mirrord
==========================================
🔍 Checking for rc-app deployment...
✅ Found deployment: dev-rc-app
🔍 Verifying deployment has single pod...
✅ Deployment has 1 replica
⏳ Waiting for deployment to be ready...
✅ Deployment is ready
🔍 Getting pod name...
✅ Found pod: dev-rc-app-ffc4969db-4zjcv
✅ Pod is running
🚀 Starting debug session...
Command: RUST_LOG=rc_web=debug mirrord exec --target pod/dev-rc-app-ffc4969db-4zjcv cargo run

# Your local application now runs with cluster environment
```

**Benefits of mirrord debugging:**

- **Environment parity**: Your local app runs with the same environment variables, secrets, and network access as the
  cluster
- **Real traffic**: Test with actual Kubernetes traffic patterns
- **Database access**: Connect to the same PostgreSQL database as the cluster
- **Service discovery**: Access other services in the cluster seamlessly

**Troubleshooting debug issues:**

```bash
# Check if deployment exists
kubectl get deployment dev-rc-app

# Scale to single replica if needed
kubectl scale deployment dev-rc-app --replicas=1

# Check pod status
kubectl get pods -l app.kubernetes.io/instance=dev

# View pod logs
kubectl logs -l app.kubernetes.io/instance=dev -f
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

1. **Explore the API**: Visit https://rc.127.0.0.1.nip.io/scalar for API documentation
2. **Debug Locally**: Use `cargo make debug` for local development with cluster environment using mirrord
3. **Database Access**: Use `./scripts/connect-postgres.sh dev` for direct database access
4. **Persistent Database Connection**: Use `./scripts/portforward-postgres.sh dev` for external database tools
5. **Pod Terminal Access**: Use `./scripts/shell-postgres.sh dev` for direct pod shell access
5. **Check Logs**: Monitor application behavior with `kubectl logs`
6. **Scale Applications**: Modify replica counts in Helm values
7. **Add PostgreSQL**: Use CNPG to create PostgreSQL clusters
8. **Custom Configuration**: Modify `k8s/rc-app/values.yaml` for customization

## Additional Resources

- **Scripts Documentation**: See `scripts/README.md` for detailed script information
- **Kubernetes Manifests**: Explore `k8s/manual/` for resource definitions
- **Helm Charts**: Check `k8s/rc-app/` for application configuration
- **Build Configuration**: Review `Makefile.toml` for available tasks

---

🎉 **Congratulations!** You now have a fully functional daksha-rc-core deployment with monitoring, ingress, and database
capabilities.
