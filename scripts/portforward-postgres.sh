#!/bin/bash

# Script to maintain persistent port forwarding to PostgreSQL database in Kubernetes
# Usage: ./portforward-postgres.sh <helm-release-name> [local-port]

set -e

# Default values
DEFAULT_LOCAL_PORT="5432"
RETRY_INTERVAL=10
MAX_RETRIES=5

# Check if release name is provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 <helm-release-name> [local-port]"
    echo "Example: $0 my-release 5432"
    echo "Default local port: $DEFAULT_LOCAL_PORT"
    exit 1
fi

RELEASE_NAME="$1"
LOCAL_PORT="${2:-$DEFAULT_LOCAL_PORT}"
CHART_NAME="rc-app"
SECRET_NAME="${RELEASE_NAME}-${CHART_NAME}-database-app"
PORT_FORWARD_PID=""

echo "Setting up persistent port forwarding for release: $RELEASE_NAME"
echo "Local port: $LOCAL_PORT"
echo "Using secret: $SECRET_NAME"

# Function to cleanup port forwarding
cleanup() {
    echo "Cleaning up port forwarding..."
    if [ ! -z "$PORT_FORWARD_PID" ] && kill -0 $PORT_FORWARD_PID 2>/dev/null; then
        kill $PORT_FORWARD_PID 2>/dev/null || true
        wait $PORT_FORWARD_PID 2>/dev/null || true
    fi
    echo "Port forwarding stopped."
}

# Set trap to cleanup on script exit
trap cleanup EXIT INT TERM

# Function to check if port is available
is_port_available() {
    ! lsof -i :$LOCAL_PORT >/dev/null 2>&1
}

# Function to check if port forwarding is working
is_port_forward_working() {
    if [ -z "$PORT_FORWARD_PID" ] || ! kill -0 $PORT_FORWARD_PID 2>/dev/null; then
        return 1
    fi

    # Check if the port is actually listening
    if ! lsof -i :$LOCAL_PORT >/dev/null 2>&1; then
        return 1
    fi

    return 0
}

# Function to get database connection info
get_db_info() {
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
}

# Function to start port forwarding
start_port_forward() {
    local retry_count=0

    while [ $retry_count -lt $MAX_RETRIES ]; do
        echo "Starting port forwarding (attempt $((retry_count + 1))/$MAX_RETRIES)..."

        # Check if port is available
        if ! is_port_available; then
            echo "WARNING: Port $LOCAL_PORT is already in use. Trying to continue..."
        fi

        # Start port forwarding in background
        kubectl port-forward "$POSTGRES_POD" "$LOCAL_PORT:5432" -n "$POSTGRES_NAMESPACE" > /dev/null 2>&1 &
        PORT_FORWARD_PID=$!

        # Wait a moment for port forwarding to establish
        sleep 5

        # Check if port forwarding is working
        if is_port_forward_working; then
            echo "Port forwarding established successfully (PID: $PORT_FORWARD_PID)"
            return 0
        else
            echo "Port forwarding failed to establish"
            if [ ! -z "$PORT_FORWARD_PID" ]; then
                kill $PORT_FORWARD_PID 2>/dev/null || true
                wait $PORT_FORWARD_PID 2>/dev/null || true
                PORT_FORWARD_PID=""
            fi
            retry_count=$((retry_count + 1))
            if [ $retry_count -lt $MAX_RETRIES ]; then
                echo "Retrying in $RETRY_INTERVAL seconds..."
                sleep $RETRY_INTERVAL
            fi
        fi
    done

    echo "ERROR: Failed to establish port forwarding after $MAX_RETRIES attempts"
    return 1
}

# Function to print connection information
print_connection_info() {
    echo ""
    echo "========================================="
    echo "PERSISTENT PORT FORWARDING ACTIVE"
    echo "========================================="
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
    echo "Password: [available in connection URLs above]"
    echo "----------------------------------------"
    echo "Port forwarding PID: $PORT_FORWARD_PID"
    echo "========================================="
    echo ""
    echo "Press Ctrl+C to stop port forwarding"
    echo ""
}

# Main execution
get_db_info

if ! start_port_forward; then
    exit 1
fi

print_connection_info

# Monitor and maintain the port forwarding
echo "Monitoring port forwarding... (checking every ${RETRY_INTERVAL}s)"
while true; do
    sleep $RETRY_INTERVAL

    if ! is_port_forward_working; then
        echo "$(date): Port forwarding lost, attempting to recover..."

        # Clean up the dead process
        if [ ! -z "$PORT_FORWARD_PID" ]; then
            kill $PORT_FORWARD_PID 2>/dev/null || true
            wait $PORT_FORWARD_PID 2>/dev/null || true
            PORT_FORWARD_PID=""
        fi

        # Try to restart port forwarding
        if start_port_forward; then
            echo "$(date): Port forwarding recovered successfully"
            echo "New PID: $PORT_FORWARD_PID"
        else
            echo "$(date): Failed to recover port forwarding, exiting..."
            exit 1
        fi
    else
        echo "$(date): Port forwarding healthy (PID: $PORT_FORWARD_PID)"
    fi
done
