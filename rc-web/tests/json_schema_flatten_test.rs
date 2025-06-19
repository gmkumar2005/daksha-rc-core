use rc_web::projections::schema_projection::{
    flatten_json_schema, generate_create_table_statement, generate_index_statements,
};
use serde_json::{json, Value};
use std::fs;

#[tokio::test]
async fn test_flatten_student_schema_from_file() {
    // Read the Student_Schema_ref_fixed.json file
    let json_content = fs::read_to_string("tests/api-tests/Student_Schema_ref_fixed.json")
        .expect("Failed to read Student_Schema_ref_fixed.json");

    let schema: Value = serde_json::from_str(&json_content).expect("Failed to parse JSON schema");

    // Flatten the schema
    let result = flatten_json_schema(&schema).unwrap();

    // Print all attributes for debugging
    println!("Flattened attributes:");
    for attr in &result {
        println!(
            "  {} -> {} ({})",
            attr.attribute_name, attr.column_type, attr.generated_column_pattern
        );
    }

    // Verify expected attributes are present - only leaf properties should be included
    let expected_attributes = vec![
        // Identity Details leaf properties (no intermediate objects)
        (
            "student_identitydetails_fullname",
            "TEXT",
            "entity_data -> 'student' -> 'identitydetails' ->> 'fullname'",
        ),
        (
            "student_identitydetails_gender",
            "TEXT",
            "entity_data -> 'student' -> 'identitydetails' ->> 'gender'",
        ),
        (
            "student_identitydetails_dob",
            "TIMESTAMPTZ",
            "entity_data -> 'student' -> 'identitydetails' ->> 'dob'",
        ),
        // Identity Holder leaf properties (4th level - should be included because definitions are present)
        (
            "student_identitydetails_identityholder_type",
            "TEXT",
            "entity_data -> 'student' -> 'identitydetails' -> 'identityholder' ->> 'type'",
        ),
        (
            "student_identitydetails_identityholder_value",
            "TEXT",
            "entity_data -> 'student' -> 'identitydetails' -> 'identityholder' ->> 'value'",
        ),
        // Contact Details leaf properties (no intermediate objects)
        (
            "student_contactdetails_email",
            "TEXT",
            "entity_data -> 'student' -> 'contactdetails' ->> 'email'",
        ),
        (
            "student_contactdetails_mobile",
            "TEXT",
            "entity_data -> 'student' -> 'contactdetails' ->> 'mobile'",
        ),
        (
            "student_contactdetails_address",
            "TEXT",
            "entity_data -> 'student' -> 'contactdetails' ->> 'address'",
        ),
    ];

    // Check that all expected attributes are present
    for (expected_name, expected_type, expected_pattern) in &expected_attributes {
        let found = result
            .iter()
            .find(|attr| attr.attribute_name == *expected_name);

        assert!(
            found.is_some(),
            "Expected attribute '{}' not found in result. Available attributes: {:?}",
            expected_name,
            result.iter().map(|a| &a.attribute_name).collect::<Vec<_>>()
        );

        let attr = found.unwrap();
        assert_eq!(
            attr.column_type, *expected_type,
            "Column type mismatch for '{}'. Expected '{}', got '{}'",
            expected_name, expected_type, attr.column_type
        );
        assert_eq!(
            attr.generated_column_pattern, *expected_pattern,
            "Generated pattern mismatch for '{}'. Expected '{}', got '{}'",
            expected_name, expected_pattern, attr.generated_column_pattern
        );
    }

    // Verify that we process 4 levels because definitions are present
    let fourth_level_attrs: Vec<_> = result
        .iter()
        .filter(|attr| attr.attribute_name.matches('_').count() == 3) // 4 levels = 3 underscores
        .collect();

    assert!(
        !fourth_level_attrs.is_empty(),
        "Expected 4th level attributes to be processed when definitions are present"
    );

    // Verify specific 4th level attributes
    assert!(
        result
            .iter()
            .any(|attr| attr.attribute_name == "student_identitydetails_identityholder_type"),
        "Should include 4th level attribute: student_identitydetails_identityholder_type"
    );
    assert!(
        result
            .iter()
            .any(|attr| attr.attribute_name == "student_identitydetails_identityholder_value"),
        "Should include 4th level attribute: student_identitydetails_identityholder_value"
    );

    // Verify that intermediate object properties are excluded
    assert!(
        !result
            .iter()
            .any(|attr| attr.attribute_name == "student_identitydetails"),
        "Intermediate object 'student_identitydetails' should not be included as it has children"
    );
    assert!(
        !result
            .iter()
            .any(|attr| attr.attribute_name == "student_contactdetails"),
        "Intermediate object 'student_contactdetails' should not be included as it has children"
    );
    assert!(
        !result
            .iter()
            .any(|attr| attr.attribute_name == "student_identitydetails_identityholder"),
        "Intermediate object 'student_identitydetails_identityholder' should not be included as it has children"
    );

    // Verify all attribute names are lowercase
    for attr in &result {
        assert_eq!(
            attr.attribute_name,
            attr.attribute_name.to_lowercase(),
            "Attribute name '{}' should be lowercase",
            attr.attribute_name
        );
    }

    // Test specific data types
    let dob_attr = result
        .iter()
        .find(|attr| attr.attribute_name == "student_identitydetails_dob");
    assert!(dob_attr.is_some(), "DOB attribute should be present");
    assert_eq!(
        dob_attr.unwrap().column_type,
        "TIMESTAMPTZ",
        "DOB should be TIMESTAMPTZ type due to date format"
    );

    let email_attr = result
        .iter()
        .find(|attr| attr.attribute_name == "student_contactdetails_email");
    assert!(email_attr.is_some(), "Email attribute should be present");
    assert_eq!(
        email_attr.unwrap().column_type,
        "TEXT",
        "Email should be TEXT type"
    );

    // Verify that only leaf properties are included
    for attr in &result {
        assert!(
            !attr.attribute_name.ends_with("details")
                || attr.attribute_name.matches('_').count() >= 2,
            "Object properties with children should not be included: {}",
            attr.attribute_name
        );
    }
}

#[tokio::test]
async fn test_generated_column_patterns_for_postgres() {
    let json_content = fs::read_to_string("tests/api-tests/Student_Schema_ref_fixed.json")
        .expect("Failed to read Student_Schema_ref_fixed.json");

    let schema: Value = serde_json::from_str(&json_content).expect("Failed to parse JSON schema");

    let result = flatten_json_schema(&schema).unwrap();

    // Test that generated column patterns are valid PostgreSQL JSON path expressions
    for attr in &result {
        let pattern = &attr.generated_column_pattern;

        // Should start with entity_data
        assert!(
            pattern.contains("entity_data"),
            "Pattern '{}' should contain 'entity_data'",
            pattern
        );

        // Should contain appropriate JSON operators
        assert!(
            pattern.contains("->>") || pattern.contains("->"),
            "Pattern '{}' should contain JSON operators (->> or ->)",
            pattern
        );

        // Verify the pattern structure matches the expected depth
        let underscore_count = attr.attribute_name.matches('_').count();
        let double_arrow_count = pattern.matches("->>").count();
        // Count single arrows that are not part of double arrows
        let temp_pattern = pattern.replace("->>", " XX ");
        let single_arrow_count = temp_pattern.matches("->").count();

        match underscore_count {
            0 => {
                // Single level: entity_data ->> 'field'
                assert!(
                    double_arrow_count == 1 && single_arrow_count == 0,
                    "Single level pattern '{}' should have one ->> and no ->",
                    pattern
                );
            }
            1 => {
                // Two levels: entity_data -> 'level1' ->> 'level2'
                assert!(
                    double_arrow_count == 1 && single_arrow_count == 1,
                    "Two level pattern '{}' should have one -> and one ->>",
                    pattern
                );
            }
            2 => {
                // Three levels: entity_data -> 'level1' -> 'level2' ->> 'level3'
                assert!(
                    double_arrow_count == 1 && single_arrow_count == 2,
                    "Three level pattern '{}' should have two -> and one ->>",
                    pattern
                );
            }
            3 => {
                // Four levels: entity_data -> 'level1' -> 'level2' -> 'level3' ->> 'level4'
                assert!(
                    double_arrow_count == 1 && single_arrow_count == 3,
                    "Four level pattern '{}' should have three -> and one ->>",
                    pattern
                );
            }
            _ => {
                panic!(
                    "Unexpected nesting depth for attribute: {}",
                    attr.attribute_name
                );
            }
        }
    }
}

#[tokio::test]
async fn test_column_type_mapping() {
    let json_content = fs::read_to_string("tests/api-tests/Student_Schema_ref_fixed.json")
        .expect("Failed to read Student_Schema_ref_fixed.json");

    let schema: Value = serde_json::from_str(&json_content).expect("Failed to parse JSON schema");

    let result = flatten_json_schema(&schema).unwrap();

    // Test specific type mappings - only leaf properties
    let type_tests = vec![
        ("student_identitydetails_fullname", "TEXT"), // string type
        ("student_identitydetails_gender", "TEXT"),   // string with enum
        ("student_identitydetails_dob", "TIMESTAMPTZ"), // string with date format
        ("student_contactdetails_email", "TEXT"),     // string type
        ("student_identitydetails_identityholder_type", "TEXT"), // nested string
        ("student_identitydetails_identityholder_value", "TEXT"), // nested string
    ];

    for (attr_name, expected_type) in type_tests {
        let attr = result
            .iter()
            .find(|a| a.attribute_name == attr_name)
            .expect(&format!("Attribute '{}' should be present", attr_name));

        assert_eq!(
            attr.column_type, expected_type,
            "Type mapping incorrect for '{}'. Expected '{}', got '{}'",
            attr_name, expected_type, attr.column_type
        );
    }
}

#[tokio::test]
async fn test_flatten_student_schema() {
    let schema = json!({
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
        }
    });

    let result = flatten_json_schema(&schema).unwrap();

    // Verify expected attributes - only leaf properties should be included
    let expected_attributes = vec![
        // Identity Details leaf properties (no intermediate objects)
        (
            "student_identitydetails_fullname",
            "TEXT",
            "entity_data -> 'student' -> 'identitydetails' ->> 'fullname'",
        ),
        (
            "student_identitydetails_gender",
            "TEXT",
            "entity_data -> 'student' -> 'identitydetails' ->> 'gender'",
        ),
        (
            "student_identitydetails_dob",
            "TIMESTAMPTZ",
            "entity_data -> 'student' -> 'identitydetails' ->> 'dob'",
        ),
        // Identity Holder leaf properties (4th level - should be included because definitions are present)
        (
            "student_identitydetails_identityholder_type",
            "TEXT",
            "entity_data -> 'student' -> 'identitydetails' -> 'identityholder' ->> 'type'",
        ),
        (
            "student_identitydetails_identityholder_value",
            "TEXT",
            "entity_data -> 'student' -> 'identitydetails' -> 'identityholder' ->> 'value'",
        ),
        // Contact Details leaf properties (no intermediate objects)
        (
            "student_contactdetails_email",
            "TEXT",
            "entity_data -> 'student' -> 'contactdetails' ->> 'email'",
        ),
        (
            "student_contactdetails_mobile",
            "TEXT",
            "entity_data -> 'student' -> 'contactdetails' ->> 'mobile'",
        ),
        (
            "student_contactdetails_address",
            "TEXT",
            "entity_data -> 'student' -> 'contactdetails' ->> 'address'",
        ),
    ];

    // Check that all expected attributes are present
    for (expected_name, expected_type, expected_pattern) in &expected_attributes {
        let found = result
            .iter()
            .find(|attr| attr.attribute_name == *expected_name);
        assert!(
            found.is_some(),
            "Expected attribute '{}' not found",
            expected_name
        );

        let attr = found.unwrap();
        assert_eq!(
            attr.column_type, *expected_type,
            "Column type mismatch for '{}'",
            expected_name
        );
        assert_eq!(
            attr.generated_column_pattern, *expected_pattern,
            "Generated pattern mismatch for '{}'",
            expected_name
        );
    }

    // Verify that we have the expected number of attributes (including nested ones)
    assert!(
        result.len() >= expected_attributes.len(),
        "Expected at least {} attributes, got {}",
        expected_attributes.len(),
        result.len()
    );
}

#[test]
fn test_flatten_simple_schema_without_definitions() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string"
            },
            "age": {
                "type": "integer"
            },
            "profile": {
                "type": "object",
                "properties": {
                    "bio": {
                        "type": "string"
                    }
                }
            }
        }
    });

    let result = flatten_json_schema(&schema).unwrap();

    let expected_attributes = vec![
        ("name", "TEXT", "entity_data ->> 'name'"),
        ("age", "INTEGER", "entity_data ->> 'age'"),
        // profile object excluded because it has children
        ("profile_bio", "TEXT", "entity_data -> 'profile' ->> 'bio'"),
    ];

    assert_eq!(result.len(), expected_attributes.len());

    for (expected_name, expected_type, expected_pattern) in expected_attributes {
        let found = result
            .iter()
            .find(|attr| attr.attribute_name == expected_name);
        assert!(
            found.is_some(),
            "Expected attribute '{}' not found",
            expected_name
        );

        let attr = found.unwrap();
        assert_eq!(attr.column_type, expected_type);
        assert_eq!(attr.generated_column_pattern, expected_pattern);
    }
}

#[test]
fn test_depth_limiting() {
    let schema = json!({
        "type": "object",
        "properties": {
            "level1": {
                "type": "object",
                "properties": {
                    "level2": {
                        "type": "object",
                        "properties": {
                            "level3": {
                                "type": "object",
                                "properties": {
                                    "level4": {
                                        "type": "string"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let result = flatten_json_schema(&schema).unwrap();

    // Should only process up to 3 levels (no definitions present)
    let level4_attr = result
        .iter()
        .find(|attr| attr.attribute_name.contains("level4"));
    assert!(
        level4_attr.is_none(),
        "Level 4 should not be processed without definitions"
    );

    // Level 3 object should be present because its children (level4) won't be processed due to depth limit
    let level3_attr = result
        .iter()
        .find(|attr| attr.attribute_name == "level1_level2_level3");
    assert!(
        level3_attr.is_some(),
        "Level 3 object should be included because its children exceed max depth"
    );

    // Level 4 should not be processed due to depth limit
    assert!(
        result
            .iter()
            .all(|attr| !attr.attribute_name.contains("level4")),
        "Level 4 should not be processed due to depth limit"
    );
}

#[tokio::test]
async fn test_generate_create_table_statement_from_student_schema() {
    // Read the Student_Schema_ref_fixed.json file
    let json_content = fs::read_to_string("tests/api-tests/Student_Schema_ref_fixed.json")
        .expect("Failed to read Student_Schema_ref_fixed.json");

    let schema: Value = serde_json::from_str(&json_content).expect("Failed to parse JSON schema");

    let result = generate_create_table_statement(&schema)
        .expect("Failed to generate CREATE TABLE statement");

    // Check that the table name is correct
    assert!(result.contains("CREATE TABLE student_projection"));

    // Check for new required columns
    assert!(result.contains("id UUID PRIMARY KEY"));
    assert!(result.contains("entity_type TEXT NOT NULL"));
    assert!(result.contains("created_by TEXT NOT NULL"));
    assert!(result.contains("created_at TIMESTAMPTZ NOT NULL"));
    assert!(result.contains("registry_def_id UUID NOT NULL"));
    assert!(result.contains("registry_def_version INTEGER NOT NULL"));
    assert!(result.contains("version INTEGER NOT NULL"));

    // Check that entity_data column exists
    assert!(result.contains("entity_data JSONB NOT NULL"));

    // Check that some expected generated columns exist
    assert!(result.contains("student_identitydetails_fullname TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("student_identitydetails_gender TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("student_identitydetails_dob TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("student_contactdetails_email TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("student_contactdetails_mobile TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("student_contactdetails_address TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("student_identitydetails_identityholder_type TEXT GENERATED ALWAYS AS"));
    assert!(
        result.contains("student_identitydetails_identityholder_value TEXT GENERATED ALWAYS AS")
    );

    // Check that the statement ends properly
    assert!(result.ends_with(");"));

    println!("Generated CREATE TABLE statement:\n{}", result);

    // Check that generated column patterns are correct
    assert!(result.contains("entity_data -> 'student' -> 'identitydetails' ->> 'fullname'"));
    assert!(result.contains("entity_data -> 'student' -> 'contactdetails' ->> 'email'"));
    assert!(result
        .contains("entity_data -> 'student' -> 'identitydetails' -> 'identityholder' ->> 'type'"));
}

#[test]
fn test_generate_create_table_statement_simple_schema() {
    let schema = json!({
        "title": "User",
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer" },
            "email": { "type": "string", "format": "email" },
            "active": { "type": "boolean" },
            "profile": {
                "type": "object",
                "properties": {
                    "bio": { "type": "string" },
                    "website": { "type": "string" }
                }
            }
        }
    });

    let result = generate_create_table_statement(&schema)
        .expect("Failed to generate CREATE TABLE statement");

    // Check table name
    assert!(result.contains("CREATE TABLE user_projection"));

    // Check for new required columns
    assert!(result.contains("id UUID PRIMARY KEY"));
    assert!(result.contains("entity_type TEXT NOT NULL"));
    assert!(result.contains("created_by TEXT NOT NULL"));
    assert!(result.contains("created_at TIMESTAMPTZ NOT NULL"));
    assert!(result.contains("registry_def_id UUID NOT NULL"));
    assert!(result.contains("registry_def_version INTEGER NOT NULL"));
    assert!(result.contains("version INTEGER NOT NULL"));

    // Check entity_data column
    assert!(result.contains("entity_data JSONB NOT NULL"));

    // Check generated columns (all primitive types converted to TEXT for immutability)
    assert!(result.contains("name TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("age TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("email TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("active TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("profile_bio TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("profile_website TEXT GENERATED ALWAYS AS"));

    // Check JSON patterns
    assert!(result.contains("entity_data ->> 'name'"));
    assert!(result.contains("entity_data ->> 'age'"));
    assert!(result.contains("entity_data -> 'profile' ->> 'bio'"));

    println!("Generated CREATE TABLE statement:\n{}", result);
}

#[test]
fn test_generate_create_table_statement_missing_title() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        }
    });

    let result = generate_create_table_statement(&schema);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Schema must have a 'title' field");
}

#[test]
fn test_generate_create_table_statement_with_definitions() {
    let schema = json!({
        "title": "Product",
        "type": "object",
        "properties": {
            "Product": {
                "$ref": "#/definitions/Product"
            }
        },
        "definitions": {
            "Product": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "price": { "type": "number" },
                    "category": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "code": { "type": "string" }
                        }
                    }
                }
            }
        }
    });

    let result = generate_create_table_statement(&schema)
        .expect("Failed to generate CREATE TABLE statement");

    // Check table name
    assert!(result.contains("CREATE TABLE product_projection"));

    // Check for new required columns
    assert!(result.contains("id UUID PRIMARY KEY"));
    assert!(result.contains("entity_type TEXT NOT NULL"));
    assert!(result.contains("created_by TEXT NOT NULL"));
    assert!(result.contains("created_at TIMESTAMPTZ NOT NULL"));
    assert!(result.contains("registry_def_id UUID NOT NULL"));
    assert!(result.contains("registry_def_version INTEGER NOT NULL"));
    assert!(result.contains("version INTEGER NOT NULL"));

    // Check that it processes definitions (all primitive types converted to TEXT for immutability)
    assert!(result.contains("product_name TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("product_price TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("product_category_name TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("product_category_code TEXT GENERATED ALWAYS AS"));

    println!("Generated CREATE TABLE statement:\n{}", result);
}

#[test]
fn test_generate_create_table_statement_various_types() {
    let schema = json!({
        "title": "TestEntity",
        "type": "object",
        "properties": {
            "text_field": { "type": "string" },
            "int_field": { "type": "integer" },
            "num_field": { "type": "number" },
            "bool_field": { "type": "boolean" },
            "date_field": { "type": "string", "format": "date" },
            "datetime_field": { "type": "string", "format": "date-time" },
            "array_field": { "type": "array", "items": { "type": "string" } },
            "object_field": { "type": "object" }
        }
    });

    let result = generate_create_table_statement(&schema)
        .expect("Failed to generate CREATE TABLE statement");

    // Check table name
    assert!(result.contains("CREATE TABLE testentity_projection"));

    // Check for new required columns
    assert!(result.contains("id UUID PRIMARY KEY"));
    assert!(result.contains("entity_type TEXT NOT NULL"));
    assert!(result.contains("created_by TEXT NOT NULL"));
    assert!(result.contains("created_at TIMESTAMPTZ NOT NULL"));
    assert!(result.contains("registry_def_id UUID NOT NULL"));
    assert!(result.contains("registry_def_version INTEGER NOT NULL"));
    assert!(result.contains("version INTEGER NOT NULL"));

    // Check different column types (all primitive types are TEXT for immutability)
    assert!(result.contains("text_field TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("int_field TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("num_field TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("bool_field TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("date_field TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("datetime_field TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("array_field JSONB GENERATED ALWAYS AS"));
    assert!(result.contains("object_field JSONB GENERATED ALWAYS AS"));

    println!("Generated CREATE TABLE statement:\n{}", result);
}

#[tokio::test]
async fn test_generate_index_statements_from_student_schema() {
    // Read the Student_Schema_ref_fixed.json file
    let json_content = fs::read_to_string("tests/api-tests/Student_Schema_ref_fixed.json")
        .expect("Failed to read Student_Schema_ref_fixed.json");

    let schema: Value = serde_json::from_str(&json_content).expect("Failed to parse JSON schema");

    let result = generate_index_statements(&schema).expect("Failed to generate index statements");

    // Check that we have at least some index statements
    assert!(
        !result.is_empty(),
        "Should generate at least one index statement"
    );

    // Check that the GIN index on entity_data is always created
    assert!(
        result.iter().any(|stmt| stmt.contains("CREATE INDEX idx_student_projection_entity_data_gin ON student_projection USING GIN (entity_data);")),
        "Should create GIN index on entity_data"
    );

    // Print all generated index statements
    println!("Generated index statements:");
    for (i, stmt) in result.iter().enumerate() {
        println!("{}. {}", i + 1, stmt);
    }

    // Check that index statements are well-formed
    for stmt in &result {
        assert!(
            stmt.starts_with("CREATE"),
            "Index statement should start with CREATE"
        );
        assert!(
            stmt.contains("INDEX"),
            "Index statement should contain INDEX"
        );
        assert!(
            stmt.contains("ON student_projection"),
            "Index statement should reference correct table"
        );
        assert!(
            stmt.ends_with(";"),
            "Index statement should end with semicolon"
        );
    }
}

#[test]
fn test_generate_index_statements_with_os_config() {
    let schema = json!({
        "title": "User",
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "email": { "type": "string" },
            "age": { "type": "integer" },
            "profile": {
                "type": "object",
                "properties": {
                    "bio": { "type": "string" },
                    "website": { "type": "string" }
                }
            }
        },
        "_osConfig": {
            "indexFields": ["name", "age"],
            "uniqueIndexFields": ["email"]
        }
    });

    let result = generate_index_statements(&schema).expect("Failed to generate index statements");

    // Should have at least 4 indexes: name, age, email (unique), and entity_data (GIN)
    assert!(
        result.len() >= 4,
        "Should generate at least 4 index statements"
    );

    // Check for regular indexes
    assert!(
        result.iter().any(|stmt| stmt
            .contains("CREATE INDEX idx_user_projection_name ON user_projection (name);")),
        "Should create index on name"
    );
    assert!(
        result
            .iter()
            .any(|stmt| stmt
                .contains("CREATE INDEX idx_user_projection_age ON user_projection (age);")),
        "Should create index on age"
    );

    // Check for unique index
    assert!(
        result.iter().any(|stmt| stmt.contains(
            "CREATE UNIQUE INDEX uidx_user_projection_email ON user_projection (email);"
        )),
        "Should create unique index on email"
    );

    // Check for GIN index on entity_data
    assert!(
        result.iter().any(|stmt| stmt.contains("CREATE INDEX idx_user_projection_entity_data_gin ON user_projection USING GIN (entity_data);")),
        "Should create GIN index on entity_data"
    );

    println!("Generated index statements:");
    for (i, stmt) in result.iter().enumerate() {
        println!("{}. {}", i + 1, stmt);
    }
}

#[test]
fn test_generate_index_statements_without_os_config() {
    let schema = json!({
        "title": "Product",
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "price": { "type": "number" }
        }
    });

    let result = generate_index_statements(&schema).expect("Failed to generate index statements");

    // Should have only the GIN index on entity_data
    assert_eq!(result.len(), 1, "Should generate only one index statement");

    // Check for GIN index on entity_data
    assert!(
        result.iter().any(|stmt| stmt.contains("CREATE INDEX idx_product_projection_entity_data_gin ON product_projection USING GIN (entity_data);")),
        "Should create GIN index on entity_data"
    );

    println!("Generated index statements:");
    for (i, stmt) in result.iter().enumerate() {
        println!("{}. {}", i + 1, stmt);
    }
}

#[test]
fn test_generate_index_statements_with_nested_fields() {
    let schema = json!({
        "title": "Employee",
        "type": "object",
        "properties": {
            "personalInfo": {
                "type": "object",
                "properties": {
                    "firstName": { "type": "string" },
                    "lastName": { "type": "string" },
                    "email": { "type": "string" }
                }
            },
            "workInfo": {
                "type": "object",
                "properties": {
                    "department": { "type": "string" },
                    "salary": { "type": "number" }
                }
            }
        },
        "_osConfig": {
            "indexFields": ["firstName", "department"],
            "uniqueIndexFields": ["email"]
        }
    });

    let result = generate_index_statements(&schema).expect("Failed to generate index statements");

    // Should have at least 4 indexes: firstName, department, email (unique), and entity_data (GIN)
    assert!(
        result.len() >= 4,
        "Should generate at least 4 index statements"
    );

    // Check that nested field indexes are created
    assert!(
        result
            .iter()
            .any(|stmt| stmt.contains("personalinfo_firstname") && stmt.contains("CREATE INDEX")),
        "Should create index on nested firstName field"
    );
    assert!(
        result
            .iter()
            .any(|stmt| stmt.contains("workinfo_department") && stmt.contains("CREATE INDEX")),
        "Should create index on nested department field"
    );
    assert!(
        result.iter().any(|stmt| stmt.contains("personalinfo_email") && stmt.contains("CREATE UNIQUE INDEX")),
        "Should create unique index on nested email field"
    );

    println!("Generated index statements:");
    for (i, stmt) in result.iter().enumerate() {
        println!("{}. {}", i + 1, stmt);
    }
}

#[test]
fn test_generate_index_statements_missing_title() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        },
        "_osConfig": {
            "indexFields": ["name"]
        }
    });

    let result = generate_index_statements(&schema);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Schema must have a 'title' field");
}

#[test]
fn test_generate_index_statements_with_definitions() {
    let schema = json!({
        "title": "Order",
        "type": "object",
        "properties": {
            "Order": {
                "$ref": "#/definitions/Order"
            }
        },
        "definitions": {
            "Order": {
                "type": "object",
                "properties": {
                    "orderId": { "type": "string" },
                    "customerName": { "type": "string" },
                    "amount": { "type": "number" },
                    "status": { "type": "string" }
                }
            }
        },
        "_osConfig": {
            "indexFields": ["orderId", "customerName"],
            "uniqueIndexFields": ["orderId"]
        }
    });

    let result = generate_index_statements(&schema).expect("Failed to generate index statements");

    // Should have at least 4 indexes: orderId, customerName, orderId (unique), and entity_data (GIN)
    assert!(
        result.len() >= 4,
        "Should generate at least 4 index statements"
    );

    // Check for regular indexes
    assert!(
        result
            .iter()
            .any(|stmt| stmt.contains("order_orderid") && stmt.contains("CREATE INDEX")),
        "Should create index on orderId"
    );
    assert!(
        result
            .iter()
            .any(|stmt| stmt.contains("order_customername") && stmt.contains("CREATE INDEX")),
        "Should create index on customerName"
    );

    // Check for unique index
    assert!(
        result
            .iter()
            .any(|stmt| stmt.contains("order_orderid") && stmt.contains("CREATE UNIQUE INDEX")),
        "Should create unique index on orderId"
    );

    // Check for GIN index on entity_data
    assert!(
        result
            .iter()
            .any(|stmt| stmt.contains("CREATE INDEX idx_order_projection_entity_data_gin")),
        "Should create GIN index on entity_data"
    );

    println!("Generated index statements:");
    for (i, stmt) in result.iter().enumerate() {
        println!("{}. {}", i + 1, stmt);
    }
}

#[test]
fn test_generate_index_statements_empty_os_config() {
    let schema = json!({
        "title": "SimpleEntity",
        "type": "object",
        "properties": {
            "field1": { "type": "string" },
            "field2": { "type": "integer" }
        },
        "_osConfig": {
            "indexFields": [],
            "uniqueIndexFields": []
        }
    });

    let result = generate_index_statements(&schema).expect("Failed to generate index statements");

    // Should have only the GIN index on entity_data
    assert_eq!(result.len(), 1, "Should generate only one index statement");

    // Check for GIN index on entity_data
    assert!(
        result
            .iter()
            .any(|stmt| stmt.contains("CREATE INDEX idx_simpleentity_projection_entity_data_gin")),
        "Should create GIN index on entity_data"
    );

    println!("Generated index statements:");
    for (i, stmt) in result.iter().enumerate() {
        println!("{}. {}", i + 1, stmt);
    }
}
