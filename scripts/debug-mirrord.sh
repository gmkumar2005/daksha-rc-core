#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Starting debug session with mirrord${NC}"
echo "=========================================="

# Check if kubectl is available
if ! command -v kubectl >/dev/null 2>&1; then
    echo -e "${RED}❌ kubectl is not installed${NC}"
    exit 1
fi

# Check if mirrord is available
if ! command -v mirrord >/dev/null 2>&1; then
    echo -e "${RED}❌ mirrord is not installed${NC}"
    echo -e "${YELLOW}💡 Install mirrord: https://mirrord.dev/docs/overview/quick-start/${NC}"
    exit 1
fi

# Check if rc-app deployment exists
echo -e "${YELLOW}🔍 Checking for rc-app deployment...${NC}"
if ! kubectl get deployment dev-rc-app -n default >/dev/null 2>&1; then
    echo -e "${RED}❌ rc-app deployment not found. Please run 'cargo make deploy-rc-app' first.${NC}"
    exit 1
fi

# Check if deployment has exactly one replica
echo -e "${YELLOW}🔍 Verifying deployment has single pod...${NC}"
REPLICA_COUNT=$(kubectl get deployment dev-rc-app -n default -o jsonpath='{.spec.replicas}')
if [ "$REPLICA_COUNT" != "1" ]; then
    echo -e "${RED}❌ Deployment has $REPLICA_COUNT replicas, expected 1${NC}"
    echo -e "${YELLOW}💡 Scale to 1 replica: kubectl scale deployment dev-rc-app --replicas=1${NC}"
    exit 1
fi

# Wait for deployment to be ready
echo -e "${YELLOW}⏳ Waiting for deployment to be ready...${NC}"
kubectl wait --for=condition=Available deployment/dev-rc-app -n default --timeout=60s

# Get the pod name
echo -e "${YELLOW}🔍 Getting pod name...${NC}"
POD_NAME=$(kubectl get pods -l app.kubernetes.io/instance=dev -n default -o jsonpath='{.items[0].metadata.name}')

if [ -z "$POD_NAME" ]; then
    echo -e "${RED}❌ No pod found for rc-app${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Found pod: $POD_NAME${NC}"

# Verify pod is running
POD_STATUS=$(kubectl get pod "$POD_NAME" -n default -o jsonpath='{.status.phase}')
if [ "$POD_STATUS" != "Running" ]; then
    echo -e "${RED}❌ Pod is not running (status: $POD_STATUS)${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Pod is running${NC}"

# Change to rc-web directory
cd rc-web

echo -e "${BLUE}🚀 Starting debug session...${NC}"
echo -e "${YELLOW}Command: RUST_LOG=rc_web=debug mirrord exec --target pod/$POD_NAME cargo run${NC}"
echo ""

# Execute mirrord with the dynamic pod name
RUST_LOG=rc_web=debug mirrord exec --target pod/$POD_NAME cargo run