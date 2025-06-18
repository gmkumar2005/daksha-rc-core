#!/bin/bash

# Script to connect to PostgreSQL database running in Kubernetes
# Usage: ./connect-postgres.sh <helm-release-name>

set -e

# Check if release name is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <helm-release-name>"
    echo "Example: $0 my-release"
    exit 1
fi

RELEASE_NAME="$1"
CHART_NAME="rc-app"
SECRET_NAME="${RELEASE_NAME}-${CHART_NAME}-database-app"
LOCAL_PORT="5432"

echo "Connecting to PostgreSQL for release: $RELEASE_NAME"
echo "Using secret: $SECRET_NAME"

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

# Extract credentials from the secret
echo "Extracting database credentials..."
DB_USERNAME=$(kubectl get secret "$SECRET_NAME" -o jsonpath='{.data.username}' | base64 -d)
DB_PASSWORD=$(kubectl get secret "$SECRET_NAME" -o jsonpath='{.data.password}' | base64 -d)
DB_NAME=$(kubectl get secret "$SECRET_NAME" -o jsonpath='{.data.dbname}' | base64 -d)

if [ -z "$DB_USERNAME" ] || [ -z "$DB_PASSWORD" ] || [ -z "$DB_NAME" ]; then
    echo "ERROR: Failed to extract credentials from secret $SECRET_NAME"
    echo "Please check if the secret exists and contains username, password, and dbname keys"
    exit 1
fi

echo "Database: $DB_NAME"
echo "Username: $DB_USERNAME"
echo ""
echo "Connection URLs:"
echo "----------------------------------------"
echo "Regular PostgreSQL URL:"
echo "postgresql://$DB_USERNAME:$DB_PASSWORD@localhost:$LOCAL_PORT/$DB_NAME"
echo ""
echo "JDBC URL:"
echo "jdbc:postgresql://localhost:$LOCAL_PORT/$DB_NAME?user=$DB_USERNAME&password=$DB_PASSWORD"
echo ""
echo "Connection Details:"
echo "Host: localhost"
echo "Port: $LOCAL_PORT"
echo "Database: $DB_NAME"
echo "Username: $DB_USERNAME"
echo "Password: [hidden - available in environment]"
echo "----------------------------------------"

# Start port forwarding in background
echo "Starting port forwarding on port $LOCAL_PORT..."
kubectl port-forward "$POSTGRES_POD" "$LOCAL_PORT:5432" -n "$POSTGRES_NAMESPACE" &
PORT_FORWARD_PID=$!

# Function to cleanup port forwarding
cleanup() {
    echo "Cleaning up port forwarding..."
    kill $PORT_FORWARD_PID 2>/dev/null || true
    wait $PORT_FORWARD_PID 2>/dev/null || true
}

# Set trap to cleanup on script exit
trap cleanup EXIT

# Wait a moment for port forwarding to establish
echo "Waiting for port forwarding to establish..."
sleep 3

# Check if psql is available
if ! command -v psql &> /dev/null; then
    echo "ERROR: psql is not installed or not available in PATH"
    echo "Please install PostgreSQL client tools"
    exit 1
fi

# Connect to the database
echo ""
echo "Connecting to PostgreSQL database..."
echo "You can now run SQL commands. Type \q to exit."
echo "----------------------------------------"

PGPASSWORD="$DB_PASSWORD" psql -h localhost -p "$LOCAL_PORT" -U "$DB_USERNAME" -d "$DB_NAME"

echo "Connection closed."
