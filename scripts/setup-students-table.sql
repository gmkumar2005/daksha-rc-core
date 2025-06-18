-- PostgreSQL script to create students table with JSONB data and generated columns
-- This script demonstrates:
-- 1. Creating a table with JSONB column for storing student schema data
-- 2. Generated column that extracts and formats indexFields from _osConfig
-- 3. Inserting sample data from Student_Schema_ref_fixed.json
-- 4. Querying to demonstrate the generated column functionality

-- Drop table if exists (for testing purposes)
DROP TABLE IF EXISTS students CASCADE;

-- Create students table with JSONB data and generated columns
CREATE TABLE students (
    id SERIAL PRIMARY KEY,
    student_data JSONB NOT NULL,
    -- Generated column: Extract indexFields from _osConfig and convert array to comma-separated string
    index_fields TEXT GENERATED ALWAYS AS (
        CASE
            WHEN jsonb_typeof(student_data->'_osConfig'->'indexFields') = 'array'
            THEN replace(replace(student_data->'_osConfig'->'indexFields'::text, '[', ''), ']', '')::text
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
            THEN replace(replace(student_data->'_osConfig'->'uniqueIndexFields'::text, '[', ''), ']', '')::text
            ELSE NULL
        END
    ) STORED,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better performance
CREATE INDEX idx_students_student_data_gin ON students USING GIN (student_data);
CREATE INDEX idx_students_index_fields ON students (index_fields);

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
  "required": [
    "Student"
  ],
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
              "enum": [
                "Male",
                "Female",
                "Other"
              ],
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
                  "$comment": "Nationality",
                  "title": "ID Type",
                  "enum": [
                    "AADHAR",
                    "PAN",
                    "LICENSE",
                    "OTHER"
                  ]
                },
                "value": {
                  "type": "string",
                  "$comment": "Nationality",
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
            "email": {
              "type": "string",
              "title": "Email"
            },
            "mobile": {
              "type": "string",
              "title": "Mobile"
            },
            "address": {
              "type": "string",
              "title": "Address"
            }
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
      "indexFields: Optional; list of field names used for creating index. Enclose within braces to indicate it is a composite index. In this definition, (serialNum, studentCode) is a composite index and studentName is a single column index.",
      "uniqueIndexFields: Optional; list of field names used for creating unique index. Field names must be different from index field name",
      "systemFields: Optional; list of fields names used for system standard information like created, updated timestamps and userid"
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
    "indexFields": [
      "studentName"
    ],
    "uniqueIndexFields": [
      "identityValue"
    ],
    "systemFields": [
      "_osCreatedAt",
      "_osUpdatedAt",
      "_osCreatedBy",
      "_osUpdatedBy",
      "_osAttestedData",
      "_osClaimId",
      "_osState"
    ],
    "attestationAttributes": [
      "educationDetails",
      "nationalIdentifier"
    ],
    "attestationPolicies": [
      {
        "name": "attestationEducationDetails",
        "properties": [
          "educationDetails/[]"
        ],
        "paths": [
          "$.educationDetails[?(@.osid == ''PROPERTY_ID'')][''instituteName'', ''program'', ''graduationYear'', ''marks'']",
          "$.identityDetails[''fullName'']"
        ],
        "type": "MANUAL",
        "attestorEntity": "Teacher",
        "attestorPlugin": "did:internal:Claim?entity=Teacher",
        "conditions": "(ATTESTOR#$.experience.[*].instituteOSID#.contains(REQUESTER#$.instituteOSID#) && ATTESTOR#$.experience[?(@.instituteOSID == REQUESTER#$.instituteOSID#)][''_osState'']#.contains(''PUBLISHED''))"
      }
    ],
    "autoAttestationPolicies": [
      {
        "parentProperty": "identityDetails",
        "property": "identityHolder",
        "nodeRef": "$.identityDetails.identityHolder",
        "valuePath": "$.identityDetails.identityHolder.value",
        "typePath": "$.identityDetails.identityHolder.type"
      }
    ],
    "subjectJsonPath": "mobile",
    "ownershipAttributes": [
      {
        "email": "/contactDetails/email",
        "mobile": "/contactDetails/mobile",
        "userId": "/contactDetails/mobile"
      }
    ],
    "inviteRoles": [
      "anonymous"
    ],
    "roles": [
      "anonymous"
    ]
  }
}'::jsonb
);

-- Insert additional sample data to demonstrate different indexFields configurations
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

-- Demonstrate the generated column functionality
SELECT
    '=== STUDENTS TABLE DEMONSTRATION ===' as demo_section;

SELECT
    'Table Structure:' as info;

-- Show table structure
\d students

SELECT
    'Sample Data with Generated Columns:' as info;

-- Query to show the generated column values
SELECT
    id,
    index_fields,
    private_fields_count,
    unique_index_fields,
    jsonb_pretty(student_data->'_osConfig'->'indexFields') as original_index_fields_json,
    created_at
FROM students
ORDER BY id;

-- Demonstrate querying by generated column
SELECT
    'Filtering by Generated Column:' as info;

SELECT
    id,
    index_fields,
    student_data->>'title' as schema_title
FROM students
WHERE index_fields LIKE '%studentName%'
ORDER BY id;

-- Show how to extract specific fields from JSONB
SELECT
    'JSONB Field Extraction Examples:' as info;

SELECT
    id,
    student_data->'_osConfig'->>'subjectJsonPath' as subject_json_path,
    student_data->'_osConfig'->'roles' as roles_array,
    student_data->'_osConfig'->'privateFields'->>0 as first_private_field,
    jsonb_array_length(student_data->'_osConfig'->'indexFields') as index_fields_count
FROM students
WHERE student_data->'_osConfig'->'indexFields' IS NOT NULL
ORDER BY id;

-- Advanced queries demonstrating JSONB operations
SELECT
    'Advanced JSONB Queries:' as info;

-- Find students with specific indexFields
SELECT
    id,
    index_fields,
    student_data->>'title' as title
FROM students
WHERE student_data->'_osConfig'->'indexFields' @> '["studentName"]'::jsonb;

-- Count occurrences of each index field across all records
SELECT
    index_field_value as index_field,
    COUNT(*) as occurrence_count
FROM students,
     jsonb_array_elements_text(student_data->'_osConfig'->'indexFields') as index_field_value
WHERE student_data->'_osConfig'->'indexFields' IS NOT NULL
GROUP BY index_field_value
ORDER BY occurrence_count DESC;

-- Show performance with GIN index
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM students
WHERE student_data @> '{"_osConfig": {"indexFields": ["studentName"]}}'::jsonb;

-- Summary information
SELECT
    'Summary Information:' as info;

SELECT
    COUNT(*) as total_records,
    COUNT(CASE WHEN index_fields IS NOT NULL THEN 1 END) as records_with_index_fields,
    AVG(private_fields_count) as avg_private_fields_count,
    STRING_AGG(DISTINCT index_fields, '; ') as all_unique_index_field_combinations
FROM students;

-- Show how the generated column handles edge cases
INSERT INTO students (student_data) VALUES
('{"title": "Empty Config", "_osConfig": {}}'::jsonb),
('{"title": "No Config"}'::jsonb),
('{"title": "Null Index Fields", "_osConfig": {"indexFields": null}}'::jsonb);

SELECT
    'Edge Cases Handling:' as info;

SELECT
    id,
    student_data->>'title' as title,
    index_fields,
    CASE
        WHEN index_fields IS NULL THEN 'NULL'
        WHEN index_fields = '' THEN 'EMPTY'
        ELSE 'HAS_VALUE'
    END as index_fields_status
FROM students
WHERE id > (SELECT MAX(id) - 3 FROM students)
ORDER BY id;
