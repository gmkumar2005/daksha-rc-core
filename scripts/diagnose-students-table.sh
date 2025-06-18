#!/bin/bash

# Diagnostic script to check students table and database state
# Usage: ./diagnose-students-table.sh <prefix>
# Example: ./diagnose-students-table.sh dev

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

# Construct service and secret names
SERVICE_NAME="${PREFIX}-rc-app-database-rw"
SECRET_NAME="${PREFIX}-rc-app-database-app"

echo "🔍 PostgreSQL Database Diagnostic Tool"
echo "====================================="
echo "Prefix: ${PREFIX}"
echo "Service: ${SERVICE_NAME}"
echo "Secret: ${SECRET_NAME}"
echo "Namespace: ${NAMESPACE}"
echo ""

# Extract database credentials from secret
echo "🔍 Extracting database credentials..."
USERNAME=$(kubectl get secret "${SECRET_NAME}" -n "${NAMESPACE}" -o jsonpath="{.data.username}" | base64 --decode)
PASSWORD=$(kubectl get secret "${SECRET_NAME}" -n "${NAMESPACE}" -o jsonpath="{.data.password}" | base64 --decode)
DBNAME=$(kubectl get secret "${SECRET_NAME}" -n "${NAMESPACE}" -o jsonpath="{.data.dbname}" | base64 --decode)

echo "✅ Database: ${DBNAME}"
echo "✅ Username: ${USERNAME}"
echo ""

# Function to check if port is in use
check_port_in_use() {
    lsof -Pi :${LOCAL_PORT} -sTCP:LISTEN -t >/dev/null 2>&1
}

# Function to start port forwarding
start_port_forwarding() {
    echo "🚀 Starting port forwarding on port ${LOCAL_PORT}..."
    kubectl port-forward "svc/${SERVICE_NAME}" -n "${NAMESPACE}" ${LOCAL_PORT}:${LOCAL_PORT} >/dev/null 2>&1 &
    PORT_FORWARD_PID=$!
    echo "✅ Port forwarding started with PID: ${PORT_FORWARD_PID}"
}

# Function to cleanup port forwarding on exit
cleanup() {
    echo ""
    echo "🧹 Cleaning up port forwarding..."
    if [ ! -z "${PORT_FORWARD_PID}" ]; then
        kill ${PORT_FORWARD_PID} 2>/dev/null || true
        wait ${PORT_FORWARD_PID} 2>/dev/null || true
    fi
    # Kill any remaining port forwards for this service
    pkill -f "kubectl port-forward.*${SERVICE_NAME}" 2>/dev/null || true
    echo "✅ Cleanup complete"
    exit 0
}

# Set trap for cleanup
trap cleanup EXIT INT TERM

# Check if port forwarding is already running
LOCAL_PORT="5432"
if check_port_in_use; then
    echo "⚠️  Port ${LOCAL_PORT} is already in use. Stopping existing port forwarding..."
    pkill -f "kubectl port-forward.*${LOCAL_PORT}:${LOCAL_PORT}" 2>/dev/null || true
    pkill -f "kubectl port-forward.*${SERVICE_NAME}" 2>/dev/null || true
    sleep 3
fi

# Start initial port forwarding
start_port_forwarding

# Wait for port forwarding to establish
echo "⏳ Waiting for port forwarding to establish..."
sleep 5

# Verify port forwarding is working
if ! check_port_in_use; then
    echo "❌ Error: Port forwarding failed to establish"
    exit 1
fi

echo "✅ Port forwarding established successfully!"
echo ""

# Run comprehensive diagnostics
echo "🔍 Running Database Diagnostics..."
echo "================================="

# Create a temporary SQL file with diagnostic queries
DIAG_SQL=$(mktemp /tmp/diagnose_XXXXXX.sql)
cat > "$DIAG_SQL" << 'EOF'
-- Database Diagnostic Queries

-- 1. Check current database and user
SELECT
    '=== CONNECTION INFO ===' as section,
    current_database() as database_name,
    current_user as username,
    current_schema() as current_schema,
    session_user as session_user;

-- 2. List all schemas
SELECT
    '=== ALL SCHEMAS ===' as section;
SELECT
    schema_name,
    schema_owner
FROM information_schema.schemata
ORDER BY schema_name;

-- 3. Check for students table in current schema
SELECT
    '=== STUDENTS TABLE IN CURRENT SCHEMA ===' as section;
SELECT
    table_schema,
    table_name,
    table_type
FROM information_schema.tables
WHERE table_name = 'students';

-- 4. Check for students table in all schemas
SELECT
    '=== STUDENTS TABLE IN ALL SCHEMAS ===' as section;
SELECT
    schemaname,
    tablename,
    tableowner,
    hasindexes,
    hasrules,
    hastriggers
FROM pg_tables
WHERE tablename = 'students';

-- 5. List all tables in current schema
SELECT
    '=== ALL TABLES IN CURRENT SCHEMA ===' as section;
SELECT
    table_name,
    table_type,
    table_schema
FROM information_schema.tables
WHERE table_schema = current_schema()
ORDER BY table_name;

-- 6. List all tables in public schema
SELECT
    '=== ALL TABLES IN PUBLIC SCHEMA ===' as section;
SELECT
    table_name,
    table_type
FROM information_schema.tables
WHERE table_schema = 'public'
ORDER BY table_name;

-- 7. Check table sizes if students table exists
SELECT
    '=== TABLE SIZES (if students exists) ===' as section;
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables
WHERE tablename = 'students';

-- 8. Show search_path
SELECT
    '=== SEARCH PATH ===' as section,
    current_setting('search_path') as search_path;

-- 9. Check for any students-related objects
SELECT
    '=== ANY STUDENTS-RELATED OBJECTS ===' as section;
SELECT
    n.nspname as schema_name,
    c.relname as object_name,
    c.relkind as object_type,
    CASE c.relkind
        WHEN 'r' THEN 'table'
        WHEN 'i' THEN 'index'
        WHEN 'S' THEN 'sequence'
        WHEN 'v' THEN 'view'
        WHEN 'm' THEN 'materialized view'
        WHEN 'c' THEN 'composite type'
        WHEN 't' THEN 'TOAST table'
        WHEN 'f' THEN 'foreign table'
    END as object_type_desc
FROM pg_class c
LEFT JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE c.relname LIKE '%student%'
ORDER BY n.nspname, c.relname;

-- 10. Recent activity (if we have permissions)
SELECT
    '=== RECENT DATABASE ACTIVITY ===' as section;
SELECT
    query,
    state,
    backend_start,
    query_start
FROM pg_stat_activity
WHERE state IS NOT NULL
  AND query NOT LIKE '%pg_stat_activity%'
ORDER BY query_start DESC
LIMIT 5;

-- 11. Check if we can create a test table
SELECT
    '=== TESTING TABLE CREATION PERMISSIONS ===' as section;
CREATE TEMPORARY TABLE test_permissions_check (id INTEGER);
SELECT 'SUCCESS: Can create tables' as permission_test;
DROP TABLE test_permissions_check;

-- 12. Show PostgreSQL version
SELECT
    '=== POSTGRESQL VERSION ===' as section,
    version() as postgresql_version;
EOF

# Execute diagnostic queries
echo "📊 Executing diagnostic queries..."
PGPASSWORD="${PASSWORD}" psql -h localhost -p ${LOCAL_PORT} -U "${USERNAME}" -d "${DBNAME}" -f "$DIAG_SQL"

# Clean up temporary file
rm -f "$DIAG_SQL"

echo ""
echo "🔧 TROUBLESHOOTING STEPS:"
echo "========================"
echo ""
echo "If students table is not found, try these steps:"
echo ""
echo "1. 📝 Re-run the setup script with verbose output:"
echo "   PGPASSWORD='${PASSWORD}' psql -h localhost -p ${LOCAL_PORT} -U ${USERNAME} -d ${DBNAME} -c '\\dt'"
echo ""
echo "2. 🔍 Check if table exists in a different schema:"
echo "   PGPASSWORD='${PASSWORD}' psql -h localhost -p ${LOCAL_PORT} -U ${USERNAME} -d ${DBNAME} -c \"SELECT schemaname, tablename FROM pg_tables WHERE tablename='students';\""
echo ""
echo "3. 📋 Create the table manually (simple version):"
echo "   PGPASSWORD='${PASSWORD}' psql -h localhost -p ${LOCAL_PORT} -U ${USERNAME} -d ${DBNAME} << 'EOSQL'"
echo "CREATE TABLE students ("
echo "    id SERIAL PRIMARY KEY,"
echo "    student_data JSONB NOT NULL,"
echo "    index_fields TEXT GENERATED ALWAYS AS ("
echo "        array_to_string("
echo "            ARRAY(SELECT jsonb_array_elements_text(student_data->'_osConfig'->'indexFields')),"
echo "            ','"
echo "        )"
echo "    ) STORED"
echo ");"
echo "EOSQL"
echo ""
echo "4. 🧪 Test with a simple insert:"
echo "   PGPASSWORD='${PASSWORD}' psql -h localhost -p ${LOCAL_PORT} -U ${USERNAME} -d ${DBNAME} -c \"INSERT INTO students (student_data) VALUES ('{\\\"_osConfig\\\": {\\\"indexFields\\\": [\\\"test1\\\", \\\"test2\\\"]}}');\""
echo ""
echo "5. ✅ Verify the table and data:"
echo "   PGPASSWORD='${PASSWORD}' psql -h localhost -p ${LOCAL_PORT} -U ${USERNAME} -d ${DBNAME} -c 'SELECT * FROM students;'"
echo ""
echo "💡 Common Issues:"
echo "  • Table created in different schema (check search_path)"
echo "  • Transaction rollback due to SQL errors"
echo "  • Permission issues"
echo "  • Connection to different database"
echo ""
echo "🔗 Connect manually to investigate:"
echo "PGPASSWORD='${PASSWORD}' psql -h localhost -p ${LOCAL_PORT} -U ${USERNAME} -d ${DBNAME}"
echo ""
echo "🔄 Starting persistent monitoring for diagnostics..."
echo "Press Ctrl+C to stop port forwarding and exit..."
echo ""

# Persistent monitoring with auto-restart
RESTART_COUNT=0
MAX_RESTARTS=10
RESTART_DELAY=3

while true; do
    # Check if port forwarding process is still running
    if ! kill -0 ${PORT_FORWARD_PID} 2>/dev/null; then
        RESTART_COUNT=$((RESTART_COUNT + 1))
        echo "⚠️  Port forwarding process died (restart #${RESTART_COUNT}) - $(date '+%H:%M:%S')"

        if [ ${RESTART_COUNT} -gt ${MAX_RESTARTS} ]; then
            echo "❌ Maximum restart attempts (${MAX_RESTARTS}) exceeded. Exiting..."
            exit 1
        fi

        echo "🔄 Restarting port forwarding in ${RESTART_DELAY} seconds..."
        sleep ${RESTART_DELAY}

        # Kill any remaining processes
        pkill -f "kubectl port-forward.*${SERVICE_NAME}" 2>/dev/null || true
        sleep 2

        # Restart port forwarding
        start_port_forwarding

        # Wait for it to establish
        sleep 3

        if check_port_in_use; then
            echo "✅ Port forwarding restarted successfully"
            # Reset restart count on successful restart
            RESTART_COUNT=0
        else
            echo "❌ Failed to restart port forwarding"
        fi
    else
        # Port forwarding is healthy, show status occasionally
        if [ $(($(date +%s) % 60)) -eq 0 ]; then
            echo "💚 Port forwarding healthy (PID: ${PORT_FORWARD_PID}) - $(date '+%H:%M:%S')"
        fi
    fi

    sleep 2
done
