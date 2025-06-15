#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Deploying demo applications (httpbin and whoami)${NC}"
echo "=================================================="

# Check if kubectl is available
if ! command -v kubectl >/dev/null 2>&1; then
    echo -e "${RED}❌ kubectl is not installed. Please run 'cargo make install-kubectl' first.${NC}"
    exit 1
fi

# Check if Kind cluster is running
echo -e "${YELLOW}🔍 Checking Kind cluster status...${NC}"
if ! kubectl cluster-info >/dev/null 2>&1; then
    echo -e "${RED}❌ Kubernetes cluster is not accessible. Please run 'cargo make setup-kind-cluster' first.${NC}"
    exit 1
fi

# Check if we're connected to the right cluster
CURRENT_CONTEXT=$(kubectl config current-context)
if [[ "$CURRENT_CONTEXT" != "kind-kind" ]]; then
    echo -e "${YELLOW}⚠️  Current context is '$CURRENT_CONTEXT', not 'kind-kind'${NC}"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo -e "${GREEN}✅ Kubernetes cluster is accessible${NC}"
echo "Current context: $CURRENT_CONTEXT"

# Deploy whoami application
echo -e "\n${YELLOW}🤖 Deploying whoami application...${NC}"
if [ -f "k8s/manual/whoami.yaml" ]; then
    kubectl apply -f k8s/manual/whoami.yaml
    echo -e "${GREEN}✅ whoami application deployed${NC}"
else
    echo -e "${RED}❌ whoami.yaml not found at k8s/manual/whoami.yaml${NC}"
    exit 1
fi

# Deploy httpbin application
echo -e "\n${YELLOW}🌐 Deploying httpbin application...${NC}"
if [ -f "k8s/manual/httpbin.yaml" ]; then
    kubectl apply -f k8s/manual/httpbin.yaml
    echo -e "${GREEN}✅ httpbin application deployed${NC}"
else
    echo -e "${RED}❌ httpbin.yaml not found at k8s/manual/httpbin.yaml${NC}"
    exit 1
fi

# Copy TLS certificate to application namespaces
echo -e "\n${YELLOW}🔐 Ensuring TLS certificates in application namespaces...${NC}"
for namespace in whoami httpbin; do
    if kubectl get namespace "$namespace" >/dev/null 2>&1; then
        if ! kubectl get secret wildcard-tls -n "$namespace" >/dev/null 2>&1; then
            echo -e "${YELLOW}  • Copying TLS certificate to $namespace namespace...${NC}"
            kubectl get secret wildcard-tls -n default -o yaml | \
                sed "s/namespace: default/namespace: $namespace/" | \
                kubectl apply -f - >/dev/null 2>&1 || true
        else
            echo -e "${GREEN}  • TLS certificate already exists in $namespace namespace${NC}"
        fi
    fi
done

# Wait for deployments to be ready
echo -e "\n${YELLOW}⏳ Waiting for deployments to be ready...${NC}"
echo -e "${YELLOW}  • Waiting for whoami deployment...${NC}"
kubectl wait --for=condition=available --timeout=60s deployment/whoami -n whoami

echo -e "${YELLOW}  • Waiting for httpbin deployment...${NC}"
kubectl wait --for=condition=available --timeout=60s deployment/httpbin -n httpbin

echo -e "${GREEN}✅ All deployments are ready!${NC}"

# Show deployment status
echo -e "\n${BLUE}📊 Deployment Status:${NC}"
echo -e "${YELLOW}whoami namespace:${NC}"
kubectl get all -n whoami

echo -e "\n${YELLOW}httpbin namespace:${NC}"
kubectl get all -n httpbin

# Test the applications
echo -e "\n${BLUE}🧪 Testing applications...${NC}"
echo -e "${YELLOW}Testing httpbin...${NC}"
if curl -k -s --max-time 10 https://httpbin.127.0.0.1.nip.io/get >/dev/null; then
    echo -e "${GREEN}✅ httpbin is responding${NC}"
else
    echo -e "${YELLOW}⚠️  httpbin may not be ready yet (this is normal)${NC}"
fi

echo -e "${YELLOW}Testing whoami...${NC}"
if curl -k -s --max-time 10 https://whoami.127.0.0.1.nip.io/ >/dev/null; then
    echo -e "${GREEN}✅ whoami is responding${NC}"
else
    echo -e "${YELLOW}⚠️  whoami may not be ready yet (this is normal)${NC}"
fi

echo -e "\n${GREEN}✅ Demo applications deployment complete!${NC}"
echo ""
echo -e "${BLUE}📋 Application URLs:${NC}"
echo "• httpbin: https://httpbin.127.0.0.1.nip.io"
echo "• whoami:  https://whoami.127.0.0.1.nip.io"
echo "• Traefik Dashboard: https://dashboard.127.0.0.1.nip.io"
echo ""
echo -e "${YELLOW}💡 Test commands:${NC}"
echo "  curl -k https://httpbin.127.0.0.1.nip.io/get"
echo "  curl -k https://whoami.127.0.0.1.nip.io/"
echo ""
echo -e "${YELLOW}🔍 Useful kubectl commands:${NC}"
echo "  kubectl get all -n whoami"
echo "  kubectl get all -n httpbin"
echo "  kubectl get all -A"