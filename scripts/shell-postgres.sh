#!/bin/bash

# Script to connect to PostgreSQL database pod terminal with interactive shell
# Usage: ./shell-postgres.sh <helm-release-name>

set -e

# Check if release name is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <helm-release-name>"
    echo "Example: $0 dev"
    exit 1
fi

RELEASE_NAME="$1"
CHART_NAME="rc-app"

echo "Connecting to PostgreSQL pod terminal for release: $RELEASE_NAME"

# Check the status of the CNPG operator
echo "Checking CNPG operator status..."
kubectl wait --for=condition=Available deployment/cnpg-cloudnative-pg -n cnpg-system --timeout=120s

# Find the postgres pod with the specified label
echo "Finding PostgreSQL pod..."
POSTGRES_POD=$(kubectl get pods -l "cnpg.io/podRole=instance" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)

if [ -z "$POSTGRES_POD" ]; then
    echo "ERROR: No PostgreSQL pod found with label cnpg.io/podRole=instance"
    exit 1
fi

POSTGRES_NAMESPACE=$(kubectl get pods -l "cnpg.io/podRole=instance" -o jsonpath='{.items[0].metadata.namespace}' 2>/dev/null)

echo "Found PostgreSQL pod: $POSTGRES_POD in namespace: $POSTGRES_NAMESPACE"

# Check if pod is running
POD_STATUS=$(kubectl get pod "$POSTGRES_POD" -n "$POSTGRES_NAMESPACE" -o jsonpath='{.status.phase}')
if [ "$POD_STATUS" != "Running" ]; then
    echo "ERROR: Pod $POSTGRES_POD is not running (status: $POD_STATUS)"
    exit 1
fi

echo "Pod status: $POD_STATUS"
echo ""
echo "========================================="
echo "CONNECTING TO POSTGRESQL POD TERMINAL"
echo "========================================="
echo "Pod: $POSTGRES_POD"
echo "Namespace: $POSTGRES_NAMESPACE"
echo "Shell: Interactive bash session"
echo "========================================="
echo ""
echo "You are now connected to the PostgreSQL pod terminal."
echo "Available commands:"
echo "  - psql: Connect to PostgreSQL directly"
echo "  - pg_dump: Backup database"
echo "  - pg_restore: Restore database"
echo "  - Standard Linux commands (ls, cat, tail, etc.)"
echo ""
echo "Type 'exit' to leave the pod terminal."
echo "----------------------------------------"

# Connect to the pod with an interactive shell
kubectl exec -it "$POSTGRES_POD" -n "$POSTGRES_NAMESPACE" -- bash

echo ""
echo "Disconnected from PostgreSQL pod terminal."
