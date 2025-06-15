#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🗄️  Installing CloudNativePG (CNPG) CRDs${NC}"
echo "=========================================="

# Check if kubectl is available
if ! command -v kubectl >/dev/null 2>&1; then
    echo -e "${RED}❌ kubectl is not installed. Please run 'cargo make install-kubectl' first.${NC}"
    exit 1
fi

# Check if helm is installed
if ! command -v helm >/dev/null 2>&1; then
    echo -e "${RED}❌ Helm is not installed. Please install helm first.${NC}"
    echo "Visit: https://helm.sh/docs/intro/install/"
    exit 1
fi

# Check if Kubernetes cluster is accessible
echo -e "${YELLOW}🔍 Checking Kubernetes cluster status...${NC}"
if ! kubectl cluster-info >/dev/null 2>&1; then
    echo -e "${RED}❌ Kubernetes cluster is not accessible. Please run 'cargo make setup-kind-cluster' first.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Kubernetes cluster is accessible${NC}"
CURRENT_CONTEXT=$(kubectl config current-context)
echo "Current context: $CURRENT_CONTEXT"

# Add CNPG Helm repository
echo -e "${YELLOW}📦 Adding CNPG Helm repository...${NC}"
helm repo add cnpg https://cloudnative-pg.github.io/charts
helm repo update

# Check if CNPG is already installed
echo -e "${YELLOW}🔍 Checking if CNPG is already installed...${NC}"
if helm list -n cnpg-system | grep -q cnpg; then
    echo -e "${GREEN}✅ CNPG is already installed, skipping installation${NC}"
    
    # Verify CNPG deployment is ready
    echo -e "${YELLOW}⏳ Verifying CNPG deployment status...${NC}"
    if kubectl wait --for=condition=Available deployment/cnpg-cloudnative-pg -n cnpg-system --timeout=30s >/dev/null 2>&1; then
        echo -e "${GREEN}✅ CNPG deployment is ready${NC}"
    else
        echo -e "${YELLOW}⚠️  CNPG deployment may not be fully ready yet${NC}"
    fi
    
    # Show status and exit
    echo -e "\n${BLUE}📊 CNPG Status:${NC}"
    kubectl get all -n cnpg-system
    echo -e "\n${GREEN}✅ CNPG is ready for use!${NC}"
    exit 0
fi

# Install CNPG
echo -e "${YELLOW}🚀 Installing CNPG...${NC}"
helm upgrade --install cnpg \
  --namespace cnpg-system \
  --create-namespace \
  cnpg/cloudnative-pg

# Wait for CNPG deployment to be available
echo -e "${YELLOW}⏳ Waiting for CNPG deployment to be ready...${NC}"
kubectl wait --for=condition=Available deployment/cnpg-cloudnative-pg -n cnpg-system --timeout=120s

echo -e "${GREEN}✅ CNPG installation complete!${NC}"

# Show CNPG status
echo -e "\n${BLUE}📊 CNPG Status:${NC}"
kubectl get all -n cnpg-system

# Show CNPG version
echo -e "\n${BLUE}📋 CNPG Information:${NC}"
if kubectl get deployment cnpg-cloudnative-pg -n cnpg-system >/dev/null 2>&1; then
    CNPG_VERSION=$(kubectl get deployment cnpg-cloudnative-pg -n cnpg-system -o jsonpath='{.spec.template.spec.containers[0].image}' | cut -d':' -f2)
    echo "CNPG Version: $CNPG_VERSION"
fi

echo -e "\n${GREEN}✅ CloudNativePG (CNPG) is ready!${NC}"
echo ""
echo -e "${YELLOW}💡 Next steps:${NC}"
echo "  • Deploy rc-app: cargo make deploy-rc-app"
echo "  • Check CNPG status: kubectl get all -n cnpg-system"
echo "  • Create PostgreSQL clusters using CNPG CRDs"