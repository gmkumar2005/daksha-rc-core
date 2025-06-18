#!/bin/bash

# Simple PostgreSQL Students Table Creation Script
# Usage: ./create-students-simple.sh <prefix>
# Example: ./create-students-simple.sh dev

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

echo "🎓 Simple Students Table Creation"
echo "================================="
echo "Prefix: ${PREFIX}"
echo "Service: ${SERVICE_NAME}"
echo "Secret: ${SECRET_NAME}"
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

# Create the students table with generated column
echo "📝 Creating students table..."
PGPASSWORD="${PASSWORD}" psql -h localhost -p ${LOCAL_PORT} -U "${USERNAME}" -d "${DBNAME}" << 'EOSQL'

-- Drop table if exists
DROP TABLE IF EXISTS students CASCADE;

-- Create a helper function for array to string conversion
CREATE OR REPLACE FUNCTION jsonb_array_to_string(arr jsonb, delimiter text DEFAULT ',')
RETURNS text
LANGUAGE sql
IMMUTABLE
AS $$
    SELECT string_agg(value #>> '{}', delimiter)
    FROM jsonb_array_elements(arr);
$$;

-- Create students table with JSONB data and generated column
CREATE TABLE students (
    id SERIAL PRIMARY KEY,
    student_data JSONB NOT NULL,
    -- Generated column: Extract indexFields from _osConfig and convert array to comma-separated string
    index_fields TEXT GENERATED ALWAYS AS (
        CASE
            WHEN jsonb_typeof(student_data->'_osConfig'->'indexFields') = 'array'
            THEN jsonb_array_to_string(student_data->'_osConfig'->'indexFields', ',')
            ELSE NULL
        END
    ) STORED,
    private_fields TEXT GENERATED ALWAYS AS (
        CASE
            WHEN jsonb_typeof(student_data->'_osConfig'->'privateFields') = 'array'
            THEN jsonb_array_to_string(student_data->'_osConfig'->'privateFields', ',')
            ELSE NULL
        END
    ) STORED,
    unique_index_fields TEXT GENERATED ALWAYS AS (
        CASE
            WHEN jsonb_typeof(student_data->'_osConfig'->'uniqueIndexFields') = 'array'
            THEN jsonb_array_to_string(student_data->'_osConfig'->'uniqueIndexFields', ',')
            ELSE NULL
        END
    ) STORED,
    schema_title TEXT GENERATED ALWAYS AS (
        student_data->>'title'
    ) STORED,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_students_student_data_gin ON students USING GIN (student_data);
CREATE INDEX idx_students_index_fields ON students (index_fields);
CREATE INDEX idx_students_private_fields ON students (private_fields);

-- Insert sample data
INSERT INTO students (student_data) VALUES
('{
  "title": "Student Schema",
  "_osConfig": {
    "indexFields": ["studentName"],
    "privateFields": ["$.identityDetails.dob", "$.identityDetails.identityType"],
    "uniqueIndexFields": ["identityValue"]
  }
}'::jsonb),
('{
  "title": "Teacher Schema",
  "_osConfig": {
    "indexFields": ["teacherName", "employeeId", "subject"],
    "privateFields": ["$.personalDetails.ssn"],
    "uniqueIndexFields": ["employeeId"]
  }
}'::jsonb),
('{
  "title": "Course Schema",
  "_osConfig": {
    "indexFields": ["courseName"],
    "uniqueIndexFields": ["courseCode"]
  }
}'::jsonb),
('{
  "title": "Complex Schema",
  "_osConfig": {
    "indexFields": ["name", "code", "category", "status"],
    "privateFields": ["$.sensitiveData.personal", "$.sensitiveData.financial"],
    "uniqueIndexFields": ["globalId", "externalRef"]
  }
}'::jsonb);

-- Verify the table was created and show results
\dt students

SELECT
    'Table created successfully!' as status,
    COUNT(*) as total_records
FROM students;

SELECT
    id,
    schema_title,
    index_fields,
    private_fields,
    unique_index_fields,
    jsonb_pretty(student_data->'_osConfig'->'indexFields') as original_json_array
FROM students
ORDER BY id;

-- Show the helper function with different delimiters
SELECT
    'Helper Function with Different Delimiters:' as demo;

SELECT
    id,
    schema_title,
    jsonb_array_to_string(student_data->'_osConfig'->'indexFields', ',') as comma_separated,
    jsonb_array_to_string(student_data->'_osConfig'->'indexFields', ' | ') as pipe_separated,
    jsonb_array_to_string(student_data->'_osConfig'->'indexFields', ';') as semicolon_separated
FROM students
WHERE jsonb_typeof(student_data->'_osConfig'->'indexFields') = 'array'
ORDER BY id;

EOSQL

echo ""
echo "🎉 Students table created successfully!"
echo ""
echo "📋 What was created:"
echo "  • students table with JSONB column"
echo "  • Generated column 'index_fields' from _osConfig.indexFields"
echo "  • 3 sample records with different indexFields configurations"
echo "  • GIN index for JSONB performance"
echo ""
echo "🔍 Test queries you can run:"
echo ""
echo "-- View all data with generated columns"
echo "SELECT id, schema_title, index_fields, private_fields, unique_index_fields FROM students;"
echo ""
echo "-- Filter by generated column"
echo "SELECT * FROM students WHERE index_fields LIKE '%studentName%';"
echo ""
echo "-- Show original JSON vs generated string"
echo "SELECT jsonb_pretty(student_data->'_osConfig'->'indexFields') as json, index_fields as string FROM students;"
echo ""
echo "-- Test the helper function with different delimiters"
echo "SELECT schema_title, jsonb_array_to_string(student_data->'_osConfig'->'indexFields', ' | ') FROM students;"
echo ""
echo "-- Search by multiple criteria"
echo "SELECT * FROM students WHERE private_fields LIKE '%,%' AND index_fields LIKE '%,%';"
echo ""
echo "💡 Connect to database:"
echo "PGPASSWORD='${PASSWORD}' psql -h localhost -p ${LOCAL_PORT} -U ${USERNAME} -d ${DBNAME}"
echo ""
echo "🔄 Starting persistent monitoring..."
echo "✅ Port forwarding is now persistent and will auto-restart if needed"
echo "✅ Safe for external database tools and JDBC connections"
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
