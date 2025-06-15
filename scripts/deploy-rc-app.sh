#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Deploying rc-app using Helm${NC}"
echo "=================================="

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

# Check if CNPG is installed
echo -e "${YELLOW}🔍 Checking if CNPG is installed...${NC}"
if ! kubectl get deployment cnpg-cloudnative-pg -n cnpg-system >/dev/null 2>&1; then
    echo -e "${RED}❌ CNPG is not installed. Please run 'cargo make install-cnpg' first.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ CNPG is installed and ready${NC}"

# Check if rc-app is already installed
echo -e "${YELLOW}🔍 Checking if rc-app is already installed...${NC}"
if helm list -n default | grep -q "^dev\s"; then
    echo -e "${GREEN}✅ rc-app (dev release) is already installed, skipping installation${NC}"
    
    # Verify deployment is ready
    echo -e "${YELLOW}⏳ Verifying rc-app deployment status...${NC}"
    if kubectl wait --for=condition=Available deployment/dev-rc-app -n default --timeout=30s >/dev/null 2>&1; then
        echo -e "${GREEN}✅ rc-app deployment is ready${NC}"
    else
        echo -e "${YELLOW}⚠️  rc-app deployment may not be fully ready yet${NC}"
    fi
    
    # Still check health
    echo -e "${YELLOW}🧪 Checking rc-app health...${NC}"
    if curl -k -s --max-time 10 https://rc.127.0.0.1.nip.io/healthz >/dev/null; then
        echo -e "${GREEN}✅ rc-app is healthy and responding${NC}"
        
        # Show status and exit
        echo -e "\n${BLUE}📊 rc-app Status:${NC}"
        kubectl get all -l app.kubernetes.io/instance=dev
        echo -e "\n${GREEN}✅ rc-app is ready for use!${NC}"
        exit 0
    else
        echo -e "${RED}❌ rc-app health check failed${NC}"
        exit 1
    fi
fi

# Install rc-app
echo -e "${YELLOW}🚀 Installing rc-app...${NC}"
helm install dev k8s/rc-app

# Wait for rc-app deployment to be available
echo -e "${YELLOW}⏳ Waiting for rc-app deployment to be ready...${NC}"
kubectl wait --for=condition=Available deployment/dev-rc-app -n default --timeout=120s

echo -e "${GREEN}✅ rc-app deployment is ready!${NC}"

# Show rc-app status
echo -e "\n${BLUE}📊 rc-app Status:${NC}"
kubectl get all -l app.kubernetes.io/instance=dev

# Health and readiness check
echo -e "\n${YELLOW}🧪 Checking rc-app health and readiness...${NC}"
echo -e "${YELLOW}  • Testing health endpoint...${NC}"

# Retry health check with backoff
MAX_RETRIES=5
RETRY_COUNT=0
HEALTH_CHECK_PASSED=false

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    if curl -k -s --max-time 10 https://rc.127.0.0.1.nip.io/healthz >/dev/null; then
        echo -e "${GREEN}✅ rc-app health check passed${NC}"
        HEALTH_CHECK_PASSED=true
        break
    else
        RETRY_COUNT=$((RETRY_COUNT + 1))
        if [ $RETRY_COUNT -lt $MAX_RETRIES ]; then
            echo -e "${YELLOW}⚠️  Health check failed, retrying in 10 seconds... (attempt $RETRY_COUNT/$MAX_RETRIES)${NC}"
            sleep 10
        fi
    fi
done

if [ "$HEALTH_CHECK_PASSED" = false ]; then
    echo -e "${RED}❌ rc-app health check failed after $MAX_RETRIES attempts${NC}"
    echo -e "${YELLOW}💡 Debugging information:${NC}"
    echo "  • Check pod logs: kubectl logs -l app.kubernetes.io/instance=dev"
    echo "  • Check pod status: kubectl get pods -l app.kubernetes.io/instance=dev"
    echo "  • Check service: kubectl get svc -l app.kubernetes.io/instance=dev"
    echo "  • Check ingress: kubectl get ingressroute -l app.kubernetes.io/instance=dev"
    exit 1
fi

# Additional readiness tests
echo -e "${YELLOW}  • Testing application response...${NC}"
RESPONSE=$(curl -k -s --max-time 10 https://rc.127.0.0.1.nip.io/healthz)
if [ -n "$RESPONSE" ]; then
    echo -e "${GREEN}✅ rc-app is responding with: ${RESPONSE}${NC}"
else
    echo -e "${YELLOW}⚠️  rc-app responded but with empty response${NC}"
fi

echo -e "\n${GREEN}✅ rc-app deployment and health checks complete!${NC}"
echo ""
echo -e "${BLUE}📋 Application Information:${NC}"
echo "• Application URL: https://rc.127.0.0.1.nip.io"
echo "• Health endpoint: https://rc.127.0.0.1.nip.io/healthz"
echo "• Helm release: dev"
echo "• Namespace: default"
echo ""
echo -e "${YELLOW}💡 Useful commands:${NC}"
echo "  • Check application status: kubectl get all -l app.kubernetes.io/instance=dev"
echo "  • View application logs: kubectl logs -l app.kubernetes.io/instance=dev"
echo "  • Test health endpoint: curl -k https://rc.127.0.0.1.nip.io/healthz"
echo "  • Uninstall application: helm uninstall dev"