use rc_web::projections::schema_projection::{
    flatten_json_schema, generate_create_table_statement,
};
use serde_json::{json, Value};
use std::fs;

#[tokio::test]
async fn test_flatten_client_schema_from_file() {
    // Read the client_schema.json file
    let json_content = fs::read_to_string("tests/api-tests/client_schema.json")
        .expect("Failed to read client_schema.json");

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

    // Verify expected attributes are present
    let expected_attributes = vec![
        // Direct properties
        ("name", "TEXT", "entity_data ->> 'name'"),
        ("organization", "TEXT", "entity_data ->> 'organization'"),
        ("industry", "TEXT", "entity_data ->> 'industry'"),
        ("requirements", "TEXT", "entity_data ->> 'requirements'"),
        // Location (via $ref)
        (
            "location_city",
            "TEXT",
            "entity_data -> 'location' ->> 'city'",
        ),
        (
            "location_state",
            "TEXT",
            "entity_data -> 'location' ->> 'state'",
        ),
        (
            "location_country",
            "TEXT",
            "entity_data -> 'location' ->> 'country'",
        ),
        // Contact Information (via $ref)
        (
            "contactinformation_email",
            "TEXT",
            "entity_data -> 'contactinformation' ->> 'email'",
        ),
        (
            "contactinformation_phone",
            "TEXT",
            "entity_data -> 'contactinformation' ->> 'phone'",
        ),
        // Verified Credentials array items should be flattened (this is the key test)
        (
            "verifiedcredentials_credentialid",
            "TEXT",
            "entity_data -> 'verifiedcredentials' ->> 'credentialid'",
        ),
        (
            "verifiedcredentials_issuer",
            "TEXT",
            "entity_data -> 'verifiedcredentials' ->> 'issuer'",
        ),
        (
            "verifiedcredentials_issuedate",
            "TIMESTAMPTZ",
            "entity_data -> 'verifiedcredentials' ->> 'issuedate'",
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

    // Verify that verifiedCredentials is NOT treated as JSONB
    let jsonb_credentials = result
        .iter()
        .find(|attr| attr.attribute_name == "verifiedcredentials" && attr.column_type == "JSONB");

    assert!(
        jsonb_credentials.is_none(),
        "verifiedCredentials should not be treated as JSONB. Found: {:?}",
        jsonb_credentials
    );
}

#[tokio::test]
async fn test_client_schema_create_table_statement() {
    let json_content = fs::read_to_string("tests/api-tests/client_schema.json")
        .expect("Failed to read client_schema.json");

    let schema: Value = serde_json::from_str(&json_content).expect("Failed to parse JSON schema");

    let result = generate_create_table_statement(&schema).unwrap();

    println!("Generated CREATE TABLE statement:");
    println!("{}", result);

    // Verify table name
    assert!(result.contains("CREATE TABLE client_projection"));

    // Verify that array items are properly flattened as individual columns
    // Note: All primitive types are converted to TEXT in CREATE TABLE for PostgreSQL immutability
    assert!(result.contains("verifiedcredentials_credentialid TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("verifiedcredentials_issuer TEXT GENERATED ALWAYS AS"));
    assert!(result.contains("verifiedcredentials_issuedate TEXT GENERATED ALWAYS AS"));

    // Verify that verifiedCredentials is NOT present as a JSONB column
    assert!(
        !result.contains("verifiedcredentials JSONB GENERATED ALWAYS AS"),
        "verifiedCredentials should not be a JSONB column. CREATE TABLE statement: {}",
        result
    );
}

#[test]
fn test_client_schema_array_with_ref_flattening() {
    // Test specifically the array with $ref scenario
    let schema = json!({
        "title": "TestClient",
        "type": "object",
        "properties": {
            "credentials": {
                "type": "array",
                "items": {
                    "$ref": "#/definitions/Credential"
                }
            }
        },
        "definitions": {
            "Credential": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string"
                    },
                    "issuer": {
                        "type": "string"
                    },
                    "issueDate": {
                        "type": "string",
                        "format": "date"
                    }
                }
            }
        }
    });

    let result = flatten_json_schema(&schema).unwrap();

    // Should flatten array items, not treat as JSONB
    let expected_attributes = vec![
        "credentials_id",
        "credentials_issuer",
        "credentials_issuedate",
    ];

    for expected_name in expected_attributes {
        let found = result
            .iter()
            .find(|attr| attr.attribute_name == expected_name);

        assert!(
            found.is_some(),
            "Expected flattened attribute '{}' not found. Available: {:?}",
            expected_name,
            result.iter().map(|a| &a.attribute_name).collect::<Vec<_>>()
        );
    }

    // Should NOT have credentials as JSONB
    let jsonb_credentials = result
        .iter()
        .find(|attr| attr.attribute_name == "credentials" && attr.column_type == "JSONB");

    assert!(
        jsonb_credentials.is_none(),
        "Array with $ref should not be treated as JSONB"
    );
}

#[test]
fn test_client_schema_depth_with_definitions() {
    let json_content = fs::read_to_string("tests/api-tests/client_schema.json")
        .expect("Failed to read client_schema.json");

    let schema: Value = serde_json::from_str(&json_content).expect("Failed to parse JSON schema");

    let result = flatten_json_schema(&schema).unwrap();

    // Since definitions are present, should process up to 5 levels
    // The client schema has:
    // Level 1: root properties
    // Level 2: location/contactInformation via $ref
    // Level 3: verifiedCredentials array items via $ref

    // All expected attributes should be present (no depth limiting issues)
    assert!(
        result.len() >= 8,
        "Expected at least 8 flattened attributes, got {}. Attributes: {:?}",
        result.len(),
        result.iter().map(|a| &a.attribute_name).collect::<Vec<_>>()
    );
}
