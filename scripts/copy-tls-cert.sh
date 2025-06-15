#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to copy TLS certificate to a namespace
copy_tls_to_namespace() {
    local namespace=$1
    
    if [ -z "$namespace" ]; then
        echo -e "${RED}❌ Namespace not provided${NC}"
        return 1
    fi
    
    # Check if wildcard-tls secret exists in default namespace
    if ! kubectl get secret wildcard-tls -n default >/dev/null 2>&1; then
        echo -e "${RED}❌ wildcard-tls secret not found in default namespace${NC}"
        return 1
    fi
    
    # Check if namespace exists
    if ! kubectl get namespace "$namespace" >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Namespace '$namespace' does not exist${NC}"
        return 1
    fi
    
    # Check if secret already exists in target namespace
    if kubectl get secret wildcard-tls -n "$namespace" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ TLS certificate already exists in namespace '$namespace'${NC}"
        return 0
    fi
    
    # Copy the secret to the target namespace
    echo -e "${YELLOW}🔐 Copying TLS certificate to namespace '$namespace'...${NC}"
    kubectl get secret wildcard-tls -n default -o yaml | \
        sed "s/namespace: default/namespace: $namespace/" | \
        kubectl apply -f - >/dev/null
    
    echo -e "${GREEN}✅ TLS certificate copied to namespace '$namespace'${NC}"
}

# Main script logic
if [ $# -eq 0 ]; then
    echo -e "${YELLOW}Usage: $0 <namespace1> [namespace2] [namespace3] ...${NC}"
    echo -e "${YELLOW}Example: $0 whoami httpbin traefik-system${NC}"
    exit 1
fi

# Process each namespace argument
for namespace in "$@"; do
    copy_tls_to_namespace "$namespace"
done

echo -e "${GREEN}✅ TLS certificate copy operation complete${NC}"