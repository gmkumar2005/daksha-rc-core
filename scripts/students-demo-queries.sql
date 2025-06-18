-- PostgreSQL Students Table Demo - Manual Queries
-- Copy and paste these queries into your PostgreSQL session
-- Usage: ./scripts/connect-postgres.sh dev (then paste these queries)

-- =============================================================================
-- 1. CREATE STUDENTS TABLE WITH GENERATED COLUMNS
-- =============================================================================

-- Drop table if exists (for testing)
DROP TABLE IF EXISTS students CASCADE;

-- Create students table with JSONB data and generated columns
CREATE TABLE students (
    id SERIAL PRIMARY KEY,
    student_data JSONB NOT NULL,
    -- Generated column: Extract indexFields from _osConfig and convert array to comma-separated string
    index_fields TEXT GENERATED ALWAYS AS (
        CASE
            WHEN jsonb_typeof(student_data->'_osConfig'->'indexFields') = 'array'
            THEN replace(replace(replace(student_data->'_osConfig'->'indexFields'::text, '[', ''), ']', ''), '"', '')
            ELSE NULL
        END
    ) STORED,
    -- Additional generated columns for demonstration
    private_fields_count INTEGER GENERATED ALWAYS AS (
        CASE
            WHEN jsonb_typeof(student_data->'_osConfig'->'privateFields') = 'array'
            THEN jsonb_array_length(student_data->'_osConfig'->'privateFields')
            ELSE 0
        END
    ) STORED,
    unique_index_fields TEXT GENERATED ALWAYS AS (
        CASE
            WHEN jsonb_typeof(student_data->'_osConfig'->'uniqueIndexFields') = 'array'
            THEN replace(replace(replace(student_data->'_osConfig'->'uniqueIndexFields'::text, '[', ''), ']', ''), '"', '')
            ELSE NULL
        END
    ) STORED,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better performance
CREATE INDEX idx_students_student_data_gin ON students USING GIN (student_data);
CREATE INDEX idx_students_index_fields ON students (index_fields);

-- =============================================================================
-- 2. INSERT STUDENT SCHEMA DATA
-- =============================================================================

-- Insert the Student_Schema_ref_fixed.json data
INSERT INTO students (student_data) VALUES (
'{
  "$schema": "http://json-schema.org/draft-07/schema",
  "type": "object",
  "properties": {
    "Student": {
      "$ref": "#/definitions/Student"
    }
  },
  "required": ["Student"],
  "title": "Student",
  "definitions": {
    "Student": {
      "type": "object",
      "title": "The Student Schema",
      "required": [],
      "properties": {
        "identityDetails": {
          "type": "object",
          "title": "Identity Details",
          "description": "Identity Details",
          "required": [],
          "properties": {
            "fullName": {
              "type": "string",
              "title": "Full name"
            },
            "gender": {
              "type": "string",
              "enum": ["Male", "Female", "Other"],
              "title": "Gender"
            },
            "dob": {
              "type": "string",
              "format": "date",
              "title": "DOB"
            },
            "identityHolder": {
              "type": "object",
              "properties": {
                "type": {
                  "type": "string",
                  "title": "ID Type",
                  "enum": ["AADHAR", "PAN", "LICENSE", "OTHER"]
                },
                "value": {
                  "type": "string",
                  "title": "ID Value"
                }
              }
            }
          }
        },
        "contactDetails": {
          "type": "object",
          "title": "Contact Details",
          "description": "Contact Details",
          "required": [],
          "properties": {
            "email": {"type": "string", "title": "Email"},
            "mobile": {"type": "string", "title": "Mobile"},
            "address": {"type": "string", "title": "Address"}
          }
        }
      }
    }
  },
  "_osConfig": {
    "osComment": [
      "This section contains the OpenSABER specific configuration information",
      "privateFields: Optional; list of field names to be encrypted and stored in database",
      "signedFields: Optional; list of field names that must be pre-signed",
      "indexFields: Optional; list of field names used for creating index",
      "uniqueIndexFields: Optional; list of field names used for creating unique index",
      "systemFields: Optional; list of fields names used for system standard information"
    ],
    "privateFields": [
      "$.identityDetails.dob",
      "$.identityDetails.identityType",
      "$.identityDetails.identityValue"
    ],
    "internalFields": [
      "$.contactDetails.email",
      "$.contactDetails.mobile",
      "$.contactDetails.address"
    ],
    "signedFields": [],
    "indexFields": ["studentName"],
    "uniqueIndexFields": ["identityValue"],
    "systemFields": [
      "_osCreatedAt",
      "_osUpdatedAt",
      "_osCreatedBy",
      "_osUpdatedBy",
      "_osAttestedData",
      "_osClaimId",
      "_osState"
    ],
    "attestationAttributes": ["educationDetails", "nationalIdentifier"],
    "subjectJsonPath": "mobile",
    "ownershipAttributes": [
      {
        "email": "/contactDetails/email",
        "mobile": "/contactDetails/mobile",
        "userId": "/contactDetails/mobile"
      }
    ],
    "inviteRoles": ["anonymous"],
    "roles": ["anonymous"]
  }
}'::jsonb
);

-- Insert additional sample data with different indexFields configurations
INSERT INTO students (student_data) VALUES
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
    "privateFields": [],
    "uniqueIndexFields": ["courseCode"]
  }
}'::jsonb),
('{
  "title": "Institution Schema",
  "_osConfig": {
    "indexFields": ["institutionName", "address", "contactNumber"],
    "privateFields": ["$.financialDetails"],
    "uniqueIndexFields": ["registrationNumber"]
  }
}'::jsonb);

-- =============================================================================
-- 3. DEMONSTRATE GENERATED COLUMN VALUES
-- =============================================================================

-- Show all records with generated columns
SELECT
    '=== ALL RECORDS WITH GENERATED COLUMNS ===' as section;

SELECT
    id,
    index_fields,
    private_fields,
    system_fields_count,
    unique_index_fields,
    student_data->>'title' as schema_title,
    created_at
FROM students
ORDER BY id;

-- =============================================================================
-- 4. DEMONSTRATE ORIGINAL vs GENERATED VALUES
-- =============================================================================

SELECT
    '=== ORIGINAL JSON vs GENERATED STRING ===' as section;

SELECT
    id,
    student_data->>'title' as title,
    jsonb_pretty(student_data->'_osConfig'->'indexFields') as original_json_array,
    index_fields as generated_comma_separated,
    jsonb_array_length(student_data->'_osConfig'->'indexFields') as array_length
FROM students
WHERE student_data->'_osConfig'->'indexFields' IS NOT NULL
ORDER BY id;

-- =============================================================================
-- 5. SEARCH AND FILTER BY GENERATED COLUMNS
-- =============================================================================

SELECT
    '=== FILTERING BY GENERATED COLUMNS ===' as section;

-- Find records with 'studentName' in indexFields
SELECT
    id,
    index_fields,
    student_data->>'title' as title
FROM students
WHERE index_fields LIKE '%studentName%';

-- Find records with multiple index fields
SELECT
    id,
    index_fields,
    student_data->>'title' as title
FROM students
WHERE index_fields LIKE '%,%';  -- Contains comma (multiple fields)

-- =============================================================================
-- 6. JSONB OPERATIONS AND QUERIES
-- =============================================================================

SELECT
    '=== ADVANCED JSONB OPERATIONS ===' as section;

-- Extract specific fields from _osConfig
SELECT
    id,
    student_data->'_osConfig'->>'subjectJsonPath' as subject_json_path,
    student_data->'_osConfig'->'roles' as roles_array,
    student_data->'_osConfig'->'privateFields'->>0 as first_private_field,
    jsonb_array_length(COALESCE(student_data->'_osConfig'->'indexFields', '[]'::jsonb)) as index_fields_count
FROM students
ORDER BY id;

-- Find records using JSONB containment operator
SELECT
    id,
    index_fields,
    student_data->>'title' as title
FROM students
WHERE student_data->'_osConfig'->'indexFields' @> '["studentName"]'::jsonb;

-- =============================================================================
-- 7. AGGREGATE ANALYSIS
-- =============================================================================

SELECT
    '=== AGGREGATE ANALYSIS ===' as section;

-- Count occurrences of each index field across all records
SELECT
    index_field_value as index_field,
    COUNT(*) as occurrence_count
FROM students,
     jsonb_array_elements_text(student_data->'_osConfig'->'indexFields') as index_field_value
WHERE student_data->'_osConfig'->'indexFields' IS NOT NULL
GROUP BY index_field_value
ORDER BY occurrence_count DESC;

-- Summary statistics
SELECT
    COUNT(*) as total_records,
    COUNT(CASE WHEN index_fields IS NOT NULL THEN 1 END) as records_with_index_fields,
    ROUND(AVG(private_fields_count), 2) as avg_private_fields_count,
    COUNT(DISTINCT index_fields) as unique_index_field_combinations
FROM students;

-- =============================================================================
-- 8. EDGE CASES TESTING
-- =============================================================================

SELECT
    '=== TESTING EDGE CASES ===' as section;

-- Insert edge cases
INSERT INTO students (student_data) VALUES
('{"title": "Empty Config", "_osConfig": {}}'::jsonb),
('{"title": "No Config"}'::jsonb),
('{"title": "Null Index Fields", "_osConfig": {"indexFields": null}}'::jsonb),
('{"title": "Empty Array", "_osConfig": {"indexFields": []}}'::jsonb);

-- Show how generated column handles edge cases
SELECT
    id,
    student_data->>'title' as title,
    index_fields,
    CASE
        WHEN index_fields IS NULL THEN 'NULL'
        WHEN index_fields = '' THEN 'EMPTY'
        ELSE 'HAS_VALUE: ' || index_fields
    END as index_fields_status,
    student_data->'_osConfig'->'indexFields' as original_json
FROM students
WHERE id > (SELECT MAX(id) - 4 FROM students)
ORDER BY id;

-- =============================================================================
-- 9. PERFORMANCE DEMONSTRATION
-- =============================================================================

SELECT
    '=== PERFORMANCE WITH GIN INDEX ===' as section;

-- Show query plan for JSONB search
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT * FROM students
WHERE student_data @> '{"_osConfig": {"indexFields": ["studentName"]}}'::jsonb;

-- =============================================================================
-- 10. FINAL SUMMARY
-- =============================================================================

SELECT
    '=== FINAL SUMMARY ===' as section;

SELECT
    'Total Records: ' || COUNT(*) as summary
FROM students
UNION ALL
SELECT
    'Records with Index Fields: ' || COUNT(CASE WHEN index_fields IS NOT NULL AND index_fields != '' THEN 1 END)
FROM students
UNION ALL
SELECT
    'Unique Index Field Combinations: ' || COUNT(DISTINCT index_fields)
FROM students
WHERE index_fields IS NOT NULL AND index_fields != '';

-- Show final state of the table
\d students

SELECT
    '=== DEMO COMPLETE ===' as section;
