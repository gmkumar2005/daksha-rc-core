#!/bin/bash

# PostgreSQL Students Table Demo Script
# Usage: ./setup-students-demo.sh <prefix>
# Example: ./setup-students-demo.sh dev

set -e

# Check if prefix is provided
if [ $# -eq 0 ]; then
    echo "Error: No prefix provided"
    echo "Usage: $0 <prefix>"
    echo "Example: $0 dev"
    exit 1
fi

PREFIX="$1"
NAMESPACE="default"
SQL_FILE="$(dirname "$0")/students-table-fixed.sql"

# Construct service and secret names
SERVICE_NAME="${PREFIX}-rc-app-database-rw"
SECRET_NAME="${PREFIX}-rc-app-database-app"

echo "🎓 PostgreSQL Students Table Demo"
echo "=================================="
echo "Prefix: ${PREFIX}"
echo "Service: ${SERVICE_NAME}"
echo "Secret: ${SECRET_NAME}"
echo "Namespace: ${NAMESPACE}"
echo ""

# Check if SQL file exists
if [ ! -f "$SQL_FILE" ]; then
    echo "❌ Error: SQL file not found: $SQL_FILE"
    echo "Please make sure setup-students-table.sql exists in the scripts directory"
    exit 1
fi

# Extract database credentials from secret
echo "🔍 Extracting database credentials..."
USERNAME=$(kubectl get secret "${SECRET_NAME}" -n "${NAMESPACE}" -o jsonpath="{.data.username}" | base64 --decode)
PASSWORD=$(kubectl get secret "${SECRET_NAME}" -n "${NAMESPACE}" -o jsonpath="{.data.password}" | base64 --decode)
DBNAME=$(kubectl get secret "${SECRET_NAME}" -n "${NAMESPACE}" -o jsonpath="{.data.dbname}" | base64 --decode)

echo "✅ Database: ${DBNAME}"
echo "✅ Username: ${USERNAME}"
echo ""

# Check if port forwarding is already running
LOCAL_PORT="5432"
if lsof -Pi :${LOCAL_PORT} -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "⚠️  Port ${LOCAL_PORT} is already in use. Stopping existing port forwarding..."
    pkill -f "kubectl port-forward.*${LOCAL_PORT}:${LOCAL_PORT}" 2>/dev/null || true
    sleep 2
fi

# Start port forwarding in background
echo "🚀 Starting port forwarding on port ${LOCAL_PORT}..."
kubectl port-forward "svc/${SERVICE_NAME}" -n "${NAMESPACE}" ${LOCAL_PORT}:${LOCAL_PORT} >/dev/null 2>&1 &
PORT_FORWARD_PID=$!

# Function to cleanup port forwarding on exit
cleanup() {
    echo ""
    echo "🧹 Cleaning up port forwarding..."
    kill $PORT_FORWARD_PID 2>/dev/null || true
    exit 0
}

# Set trap to cleanup on script exit
trap cleanup EXIT INT TERM

# Wait for port forwarding to establish
echo "⏳ Waiting for port forwarding to establish..."
sleep 5

# Check if port forwarding is working
if ! lsof -Pi :${LOCAL_PORT} -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "❌ Error: Port forwarding failed to establish"
    exit 1
fi

echo "✅ Port forwarding established successfully!"
echo ""

# Execute the SQL script
echo "📝 Executing students table setup SQL..."
echo "======================================"
PGPASSWORD="${PASSWORD}" psql -h localhost -p ${LOCAL_PORT} -U "${USERNAME}" -d "${DBNAME}" -f "$SQL_FILE"

echo ""
echo "🎉 Students table demo completed successfully!"
echo ""
echo "📋 What was created:"
echo "  • students table with JSONB column for schema data"
echo "  • Generated column 'index_fields' from _osConfig.indexFields"
echo "  • Sample data inserted from Student_Schema_ref_fixed.json"
echo "  • Additional demo records with different configurations"
echo "  • Indexes for performance optimization"
echo ""
echo "🔍 Try these queries to explore the data:"
echo ""
echo "-- View all students with generated columns"
echo "SELECT id, index_fields, private_fields_count, unique_index_fields"
echo "FROM students ORDER BY id;"
echo ""
echo "-- Search by generated index_fields column"
echo "SELECT * FROM students WHERE index_fields LIKE '%studentName%';"
echo ""
echo "-- Extract specific JSONB fields"
echo "SELECT id, student_data->'_osConfig'->>'subjectJsonPath' as subject_path"
echo "FROM students WHERE student_data->'_osConfig' IS NOT NULL;"
echo ""
echo "💡 Connect to the database using:"
echo "PGPASSWORD='${PASSWORD}' psql -h localhost -p ${LOCAL_PORT} -U ${USERNAME} -d ${DBNAME}"
echo ""
echo "Press Ctrl+C to stop port forwarding and exit..."

# Keep the script running to maintain port forwarding
while true; do
    if ! kill -0 $PORT_FORWARD_PID 2>/dev/null; then
        echo "❌ Port forwarding process died. Exiting..."
        exit 1
    fi
    sleep 5
done
