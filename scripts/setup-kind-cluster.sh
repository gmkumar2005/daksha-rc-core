#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Setting up Kind cluster with Traefik${NC}"
echo "=============================================="

# Check if kind is installed
if ! command -v kind >/dev/null 2>&1; then
    echo -e "${RED}❌ Kind is not installed. Please install kind first.${NC}"
    echo "Visit: https://kind.sigs.k8s.io/docs/user/quick-start/#installation"
    exit 1
fi

# Check if helm is installed
if ! command -v helm >/dev/null 2>&1; then
    echo -e "${RED}❌ Helm is not installed. Please install helm first.${NC}"
    echo "Visit: https://helm.sh/docs/intro/install/"
    exit 1
fi

# Check if kubectl is installed
if ! command -v kubectl >/dev/null 2>&1; then
    echo -e "${RED}❌ kubectl is not installed. Please run 'cargo make install-kubectl' first.${NC}"
    exit 1
fi

echo -e "${YELLOW}🔍 Checking if Kind cluster already exists...${NC}"
if kind get clusters | grep -q "^kind$"; then
    echo -e "${YELLOW}⚠️  Kind cluster 'kind' already exists${NC}"
    read -p "Do you want to delete and recreate it? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}🗑️  Deleting existing Kind cluster...${NC}"
        kind delete cluster
    else
        echo -e "${GREEN}✅ Using existing Kind cluster${NC}"
    fi
else
    echo -e "${GREEN}✅ No existing Kind cluster found${NC}"
fi

# Create Kind cluster if it doesn't exist
if ! kind get clusters | grep -q "^kind$"; then
    echo -e "${YELLOW}🏗️  Creating Kind cluster...${NC}"
    if [ -f "k8s/manual/kind-config.yaml" ]; then
        KIND_EXPERIMENTAL_PROVIDER=podman kind create cluster --config k8s/manual/kind-config.yaml
    else
        echo -e "${RED}❌ kind-config.yaml not found at k8s/manual/kind-config.yaml${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Kind cluster created successfully${NC}"
fi

# Set kubectl context
echo -e "${YELLOW}🔧 Setting kubectl context...${NC}"
kubectl config use-context kind-kind

# Add Traefik Helm repository
echo -e "${YELLOW}📦 Adding Traefik Helm repository...${NC}"
helm repo add traefik https://helm.traefik.io/traefik
helm repo update

# Install Traefik CRDs
echo -e "${YELLOW}🔧 Installing Traefik CRDs...${NC}"
helm install traefik-crds traefik/traefik-crds --namespace traefik-system --create-namespace

# Install Traefik
echo -e "${YELLOW}🚀 Installing Traefik...${NC}"
if [ -f "k8s/manual/traefik-values.yaml" ]; then
    helm upgrade --install traefik traefik/traefik -f k8s/manual/traefik-values.yaml --namespace traefik-system --create-namespace
else
    echo -e "${RED}❌ traefik-values.yaml not found at k8s/manual/traefik-values.yaml${NC}"
    exit 1
fi

# Wait for Traefik to be ready
echo -e "${YELLOW}⏳ Waiting for Traefik to be ready...${NC}"
kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=traefik -n traefik-system --timeout=60s

# Create wildcard TLS certificate
echo -e "${YELLOW}🔐 Creating wildcard TLS certificate...${NC}"
# Create a temporary directory for certificates
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

# Generate self-signed certificate for *.127.0.0.1.nip.io
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout tls.key \
  -out tls.crt \
  -subj "/CN=*.127.0.0.1.nip.io" \
  -addext "subjectAltName=DNS:*.127.0.0.1.nip.io,DNS:127.0.0.1.nip.io" 2>/dev/null

# Create TLS secret in default namespace
kubectl create secret tls wildcard-tls \
  --cert=tls.crt \
  --key=tls.key \
  --dry-run=client -o yaml | kubectl apply -f -

# Copy TLS secret to traefik-system namespace
kubectl get secret wildcard-tls -o yaml | \
  sed 's/namespace: default/namespace: traefik-system/' | \
  kubectl apply -f -

# Cleanup
cd - >/dev/null
rm -rf "$TMP_DIR"

echo -e "${GREEN}✅ TLS certificate created and copied to namespaces${NC}"

# Apply Traefik dashboard IngressRoute
echo -e "${YELLOW}🖥️  Setting up Traefik dashboard...${NC}"
if [ -f "k8s/manual/traefik-dashboard-ingressroute.yaml" ]; then
    kubectl apply -f k8s/manual/traefik-dashboard-ingressroute.yaml
else
    echo -e "${YELLOW}⚠️  traefik-dashboard-ingressroute.yaml not found, skipping dashboard setup${NC}"
fi

echo -e "${GREEN}✅ Kind cluster with Traefik setup complete!${NC}"
echo ""
echo -e "${BLUE}📋 Cluster Information:${NC}"
echo "Cluster: kind"
echo "Context: kind-kind"
echo "Traefik Namespace: traefik-system"
echo "Traefik Dashboard: https://dashboard.127.0.0.1.nip.io"
echo ""
echo -e "${YELLOW}💡 Next steps:${NC}"
echo "  • Deploy demo apps: cargo make deploy-demo-apps"
echo "  • Check cluster status: kubectl get all -A"
echo "  • Check Traefik status: kubectl get all -n traefik-system"
echo "  • Access Traefik dashboard: https://dashboard.127.0.0.1.nip.io"