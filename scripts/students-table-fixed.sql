-- PostgreSQL script to create students table with JSONB data and generated columns
-- This script demonstrates:
-- 1. Creating a table with JSONB column for storing student schema data
-- 2. Generated column that extracts and formats indexFields from _osConfig
-- 3. Inserting sample data from Student_Schema_ref_fixed.json
-- 4. Using PostgreSQL functions that work in generated columns

-- Drop table if exists (for testing purposes)
DROP TABLE IF EXISTS students CASCADE;

-- Create a helper function for array to string conversion
-- This function can be used in generated columns unlike subqueries
CREATE OR REPLACE FUNCTION jsonb_array_to_string(arr jsonb, delimiter text DEFAULT ',')
RETURNS text
LANGUAGE sql
IMMUTABLE
AS $$
    SELECT string_agg(value #>> '{}', delimiter)
    FROM jsonb_array_elements(arr);
$$;

-- Create students table with JSONB data and generated columns
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
    -- Additional generated columns for demonstration
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
    -- Refactored: Replace system_fields_count with system_fields (comma-separated values)
    system_fields TEXT GENERATED ALWAYS AS (
        CASE
            WHEN jsonb_typeof(student_data->'_osConfig'->'systemFields') = 'array'
            THEN jsonb_array_to_string(student_data->'_osConfig'->'systemFields', ',')
            ELSE NULL
        END
    ) STORED,
    -- New generated column: attestationAttributes (comma-separated values)
    attestation_attributes TEXT GENERATED ALWAYS AS (
        CASE
            WHEN jsonb_typeof(student_data->'_osConfig'->'attestationAttributes') = 'array'
            THEN jsonb_array_to_string(student_data->'_osConfig'->'attestationAttributes', ',')
            ELSE NULL
        END
    ) STORED,
    -- New generated column: inviteRoles (comma-separated values)
    invite_roles TEXT GENERATED ALWAYS AS (
        CASE
            WHEN jsonb_typeof(student_data->'_osConfig'->'inviteRoles') = 'array'
            THEN jsonb_array_to_string(student_data->'_osConfig'->'inviteRoles', ',')
            ELSE NULL
        END
    ) STORED,
    -- New generated column: roles (comma-separated values)
    roles TEXT GENERATED ALWAYS AS (
        CASE
            WHEN jsonb_typeof(student_data->'_osConfig'->'roles') = 'array'
            THEN jsonb_array_to_string(student_data->'_osConfig'->'roles', ',')
            ELSE NULL
        END
    ) STORED,
    schema_title TEXT GENERATED ALWAYS AS (
        student_data->>'title'
    ) STORED,
    has_attestation_policies BOOLEAN GENERATED ALWAYS AS (
        jsonb_typeof(student_data->'_osConfig'->'attestationPolicies') = 'array'
        AND jsonb_array_length(student_data->'_osConfig'->'attestationPolicies') > 0
    ) STORED,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better performance
CREATE INDEX idx_students_student_data_gin ON students USING GIN (student_data);
CREATE INDEX idx_students_index_fields ON students (index_fields);
CREATE INDEX idx_students_schema_title ON students (schema_title);
CREATE INDEX idx_students_private_fields ON students (private_fields);
CREATE INDEX idx_students_system_fields ON students (system_fields);
CREATE INDEX idx_students_attestation_attributes ON students (attestation_attributes);
CREATE INDEX idx_students_invite_roles ON students (invite_roles);
CREATE INDEX idx_students_roles ON students (roles);

-- Create trigger for updating updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

CREATE TRIGGER update_students_updated_at
    BEFORE UPDATE ON students
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

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
          "$.educationDetails[?(@.osid == PROPERTY_ID)][instituteName, program, graduationYear, marks]",
          "$.identityDetails[fullName]"
        ],
        "type": "MANUAL",
        "attestorEntity": "Teacher",
        "attestorPlugin": "did:internal:Claim?entity=Teacher",
        "conditions": "(ATTESTOR#$.experience.[*].instituteOSID#.contains(REQUESTER#$.instituteOSID#) && ATTESTOR#$.experience[?(@.instituteOSID == REQUESTER#$.instituteOSID#)][_osState]#.contains(PUBLISHED))"
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

-- Insert additional sample data to demonstrate different configurations
INSERT INTO students (student_data) VALUES
('{
  "title": "Teacher Schema",
  "_osConfig": {
    "indexFields": ["teacherName", "employeeId", "subject"],
    "privateFields": ["$.personalDetails.ssn"],
    "uniqueIndexFields": ["employeeId"],
    "systemFields": ["_osCreatedAt", "_osUpdatedAt"],
    "attestationAttributes": ["qualification", "experience"],
    "inviteRoles": ["admin", "teacher"],
    "roles": ["teacher", "mentor"]
  }
}'::jsonb),
('{
  "title": "Course Schema",
  "_osConfig": {
    "indexFields": ["courseName"],
    "privateFields": [],
    "uniqueIndexFields": ["courseCode"],
    "systemFields": ["_osCreatedAt"],
    "attestationAttributes": ["accreditation"],
    "inviteRoles": ["anonymous"],
    "roles": ["public"]
  }
}'::jsonb),
('{
  "title": "Institution Schema",
  "_osConfig": {
    "indexFields": ["institutionName", "address", "contactNumber"],
    "privateFields": ["$.financialDetails"],
    "uniqueIndexFields": ["registrationNumber"],
    "systemFields": ["_osCreatedAt", "_osUpdatedAt", "_osCreatedBy"],
    "attestationAttributes": ["accreditation", "compliance", "audit"],
    "inviteRoles": ["admin", "staff", "auditor"],
    "roles": ["institution", "accredited", "verified"]
  }
}'::jsonb),
('{
  "title": "Complex Schema",
  "_osConfig": {
    "indexFields": ["name", "code", "category", "status"],
    "privateFields": ["$.sensitiveData.personal", "$.sensitiveData.financial"],
    "uniqueIndexFields": ["globalId", "externalRef"],
    "systemFields": ["_osCreatedAt", "_osUpdatedAt", "_osCreatedBy", "_osUpdatedBy"],
    "attestationAttributes": ["identity", "qualification", "background"],
    "inviteRoles": ["admin", "moderator", "reviewer"],
    "roles": ["user", "verified", "premium"],
    "attestationPolicies": [
      {"name": "verification1", "type": "AUTO"},
      {"name": "verification2", "type": "MANUAL"}
    ]
  }
}'::jsonb);

-- Insert edge cases for testing
INSERT INTO students (student_data) VALUES
('{"title": "Empty Config", "_osConfig": {}}'::jsonb),
('{"title": "No Config at all"}'::jsonb),
('{"title": "Null Fields", "_osConfig": {"indexFields": null, "systemFields": null, "attestationAttributes": null, "inviteRoles": null, "roles": null}}'::jsonb),
('{"title": "Empty Arrays", "_osConfig": {"indexFields": [], "systemFields": [], "attestationAttributes": [], "inviteRoles": [], "roles": []}}'::jsonb),
('{"title": "Single Field Arrays", "_osConfig": {"indexFields": ["singleField"], "systemFields": ["_osCreatedAt"], "attestationAttributes": ["singleAttr"], "inviteRoles": ["anonymous"], "roles": ["user"]}}'::jsonb);

-- Demonstrate the generated column functionality
SELECT
    '=== STUDENTS TABLE DEMONSTRATION ===' as demo_section;

-- Show table structure
SELECT
    'Table Structure Information:' as info;

\d students

SELECT
    'Sample Data with Generated Columns:' as info;

-- Query to show the generated column values
SELECT
    id,
    schema_title,
    index_fields,
    private_fields,
    unique_index_fields,
    system_fields,
    attestation_attributes,
    invite_roles,
    roles,
    has_attestation_policies,
    jsonb_pretty(student_data->'_osConfig'->'indexFields') as original_index_fields_json,
    created_at
FROM students
ORDER BY id;

-- Demonstrate querying by generated columns
SELECT
    'Filtering by Generated Columns:' as info;

SELECT
    id,
    schema_title,
    index_fields,
    system_fields,
    attestation_attributes,
    invite_roles,
    roles
FROM students
WHERE index_fields LIKE '%studentName%'
   OR index_fields LIKE '%teacherName%'
   OR roles LIKE '%anonymous%'
ORDER BY id;

-- Show how to extract specific fields from JSONB
SELECT
    'JSONB Field Extraction Examples:' as info;

SELECT
    id,
    schema_title,
    student_data->'_osConfig'->>'subjectJsonPath' as subject_json_path,
    student_data->'_osConfig'->'roles' as roles_array,
    student_data->'_osConfig'->'privateFields'->>0 as first_private_field,
    jsonb_array_length(COALESCE(student_data->'_osConfig'->'indexFields', '[]'::jsonb)) as index_fields_count,
    jsonb_array_length(COALESCE(student_data->'_osConfig'->'systemFields', '[]'::jsonb)) as system_fields_count,
    jsonb_array_length(COALESCE(student_data->'_osConfig'->'attestationAttributes', '[]'::jsonb)) as attestation_attributes_count
FROM students
WHERE student_data->'_osConfig' IS NOT NULL
ORDER BY id;

-- Advanced queries demonstrating JSONB operations
SELECT
    'Advanced JSONB Queries:' as info;

-- Find students with specific roles
SELECT
    id,
    schema_title,
    roles,
    invite_roles
FROM students
WHERE roles LIKE '%anonymous%'
   OR invite_roles LIKE '%admin%';

-- Find students with multiple system fields
SELECT
    id,
    schema_title,
    system_fields,
    attestation_attributes
FROM students
WHERE system_fields LIKE '%,%'
   AND attestation_attributes IS NOT NULL
ORDER BY LENGTH(system_fields) DESC;

-- Count occurrences of each role across all records
SELECT
    'Role Popularity Analysis:' as info;

SELECT
    role_value as role,
    COUNT(*) as occurrence_count
FROM students,
     jsonb_array_elements_text(student_data->'_osConfig'->'roles') as role_value
WHERE jsonb_typeof(student_data->'_osConfig'->'roles') = 'array'
GROUP BY role_value
ORDER BY occurrence_count DESC;

-- Count occurrences of each attestation attribute
SELECT
    'Attestation Attributes Analysis:' as info;

SELECT
    attr_value as attestation_attribute,
    COUNT(*) as occurrence_count
FROM students,
     jsonb_array_elements_text(student_data->'_osConfig'->'attestationAttributes') as attr_value
WHERE jsonb_typeof(student_data->'_osConfig'->'attestationAttributes') = 'array'
GROUP BY attr_value
ORDER BY occurrence_count DESC;

-- Show performance with GIN index
SELECT
    'Performance Test with GIN Index:' as info;

EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM students
WHERE student_data @> '{"_osConfig": {"roles": ["anonymous"]}}'::jsonb;

-- Summary information
SELECT
    'Summary Statistics:' as info;

SELECT
    COUNT(*) as total_records,
    COUNT(CASE WHEN index_fields IS NOT NULL AND index_fields != '' THEN 1 END) as records_with_index_fields,
    COUNT(CASE WHEN private_fields IS NOT NULL AND private_fields != '' THEN 1 END) as records_with_private_fields,
    COUNT(CASE WHEN system_fields IS NOT NULL AND system_fields != '' THEN 1 END) as records_with_system_fields,
    COUNT(CASE WHEN attestation_attributes IS NOT NULL AND attestation_attributes != '' THEN 1 END) as records_with_attestation_attributes,
    COUNT(CASE WHEN invite_roles IS NOT NULL AND invite_roles != '' THEN 1 END) as records_with_invite_roles,
    COUNT(CASE WHEN roles IS NOT NULL AND roles != '' THEN 1 END) as records_with_roles,
    COUNT(CASE WHEN has_attestation_policies THEN 1 END) as records_with_attestation_policies,
    STRING_AGG(DISTINCT roles, '; ' ORDER BY roles) as all_unique_roles
FROM students;

-- Show the helper function in action
SELECT
    'Helper Function Demonstration:' as info;

SELECT
    id,
    schema_title,
    student_data->'_osConfig'->'systemFields' as original_system_fields_json,
    jsonb_array_to_string(student_data->'_osConfig'->'systemFields', ',') as comma_separated,
    jsonb_array_to_string(student_data->'_osConfig'->'systemFields', ' | ') as pipe_separated,
    student_data->'_osConfig'->'attestationAttributes' as original_attestation_attributes_json,
    jsonb_array_to_string(student_data->'_osConfig'->'attestationAttributes', ',') as attestation_attrs_comma_separated,
    student_data->'_osConfig'->'roles' as original_roles_json,
    jsonb_array_to_string(student_data->'_osConfig'->'roles', ',') as roles_comma_separated
FROM students
WHERE (jsonb_typeof(student_data->'_osConfig'->'systemFields') = 'array'
       AND jsonb_array_length(student_data->'_osConfig'->'systemFields') > 0)
   OR (jsonb_typeof(student_data->'_osConfig'->'attestationAttributes') = 'array'
       AND jsonb_array_length(student_data->'_osConfig'->'attestationAttributes') > 0)
   OR (jsonb_typeof(student_data->'_osConfig'->'roles') = 'array'
       AND jsonb_array_length(student_data->'_osConfig'->'roles') > 0)
ORDER BY id;

-- Edge cases handling demonstration
SELECT
    'Edge Cases Handling:' as info;

SELECT
    id,
    schema_title,
    index_fields,
    system_fields,
    attestation_attributes,
    invite_roles,
    roles,
    CASE
        WHEN roles IS NULL THEN 'NULL'
        WHEN roles = '' THEN 'EMPTY'
        WHEN roles NOT LIKE '%,%' THEN 'SINGLE_ROLE'
        ELSE 'MULTIPLE_ROLES'
    END as roles_classification,
    CASE
        WHEN system_fields IS NULL THEN 'NULL'
        WHEN system_fields = '' THEN 'EMPTY'
        WHEN system_fields NOT LIKE '%,%' THEN 'SINGLE_FIELD'
        ELSE 'MULTIPLE_FIELDS'
    END as system_fields_classification
FROM students
ORDER BY id;

-- Test the update trigger
UPDATE students
SET student_data = student_data || '{"lastModified": "test"}'::jsonb
WHERE id = 1;

SELECT
    'Update Trigger Test:' as info;

SELECT
    id,
    schema_title,
    created_at,
    updated_at,
    (updated_at > created_at) as was_updated
FROM students
WHERE id = 1;

SELECT
    '=== DEMONSTRATION COMPLETE ===' as final_section;

-- Useful queries for development
SELECT
    'Useful Development Queries:' as info;

-- Query to find all schemas with their capabilities
CREATE VIEW student_schema_summary AS
SELECT
    id,
    schema_title,
    index_fields,
    private_fields,
    unique_index_fields,
    system_fields,
    attestation_attributes,
    invite_roles,
    roles,
    has_attestation_policies,
    CASE
        WHEN private_fields IS NOT NULL AND private_fields != '' THEN 'Has Privacy Controls'
        ELSE 'No Privacy Controls'
    END as privacy_status,
    CASE
        WHEN has_attestation_policies THEN 'Supports Attestation'
        ELSE 'No Attestation'
    END as attestation_status,
    CASE
        WHEN roles IS NOT NULL AND roles != '' THEN 'Has Role-Based Access'
        ELSE 'No Role Controls'
    END as role_status,
    created_at
FROM students
WHERE schema_title IS NOT NULL;

SELECT * FROM student_schema_summary ORDER BY id;

-- Role-based filtering examples
SELECT
    'Role-based Filtering Examples:' as info;

-- Find all schemas accessible to anonymous users
SELECT
    id,
    schema_title,
    roles,
    invite_roles
FROM students
WHERE roles LIKE '%anonymous%'
   OR invite_roles LIKE '%anonymous%';

-- Find all schemas with admin access
SELECT
    id,
    schema_title,
    roles,
    invite_roles
FROM students
WHERE roles LIKE '%admin%'
   OR invite_roles LIKE '%admin%';

-- Find schemas with attestation capabilities
SELECT
    id,
    schema_title,
    attestation_attributes,
    has_attestation_policies
FROM students
WHERE attestation_attributes IS NOT NULL
  AND attestation_attributes != ''
  AND has_attestation_policies = true;
