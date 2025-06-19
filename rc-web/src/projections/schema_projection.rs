//! # Schema Projection Module
//!
//! This module provides functionality for converting JSON schemas into database projection structures.
//! It includes functions for flattening JSON schemas, generating CREATE TABLE statements, and
//! generating CREATE INDEX statements for PostgreSQL databases.
//!
//! ## Main Functions
//!
//! - `flatten_json_schema`: Converts JSON schema into flattened attributes with PostgreSQL types
//! - `generate_create_table_statement`: Creates complete CREATE TABLE DDL from JSON schema
//! - `generate_index_statements`: Creates CREATE INDEX statements based on _osConfig
//!
//! ## Usage Example
//!
//! ```rust
//! use serde_json::json;
//! use rc_web::projections::schema_projection::{
//!     flatten_json_schema, generate_create_table_statement, generate_index_statements
//! };
//!
//! let schema = json!({
//!     "title": "Student",
//!     "type": "object",
//!     "properties": {
//!         "name": { "type": "string" },
//!         "age": { "type": "integer" }
//!     },
//!     "_osConfig": {
//!         "indexFields": ["name"]
//!     }
//! });
//!
//! // Generate flattened attributes
//! let attributes = flatten_json_schema(&schema).unwrap();
//!
//! // Generate CREATE TABLE statement
//! let create_table = generate_create_table_statement(&schema).unwrap();
//!
//! // Generate CREATE INDEX statements
//! let indexes = generate_index_statements(&schema).unwrap();
//! ```
//!
//! ## Features
//!
//! - Maps JSON schema types to PostgreSQL column types
//! - Creates PostgreSQL JSON path expressions for generated columns
//! - Handles nested objects with underscore separation
//! - Supports depth control (max 3-4 levels)
//! - Converts all attribute names to lowercase
//! - Excludes intermediate object properties that have children
//!
//! ## PostgreSQL Type Mapping
//!
//! - `string` → `TEXT` (default), `TIMESTAMPTZ` (for date/date-time formats)
//! - `integer` → `INTEGER`
//! - `number` → `NUMERIC`
//! - `boolean` → `BOOLEAN`
//! - `array` → `JSONB`
//! - `object` → `JSONB` (at max depth only)

use serde_json::Value;

/// Represents a flattened attribute from a JSON schema with PostgreSQL column information
#[derive(Debug, Clone)]
pub struct FlattenedAttribute {
    /// The flattened attribute name (e.g., "user_profile_email")
    pub attribute_name: String,
    /// The PostgreSQL column type (e.g., "TEXT", "INTEGER", "TIMESTAMPTZ")
    pub column_type: String,
    /// The PostgreSQL JSON path expression for generated columns (e.g., "entity_data -> 'user' -> 'profile' ->> 'email'")
    pub generated_column_pattern: String,
}

/// Flattens a JSON schema and returns attribute names with database column types
/// and generated column patterns for PostgreSQL
///
/// # Arguments
/// * `schema` - The JSON schema to flatten
///
/// # Returns
/// * `Result<Vec<FlattenedAttribute>, String>` - Vector of flattened attributes or error message
///
/// # Example
/// ```rust
/// use serde_json::json;
/// use rc_web::projections::schema_projection::flatten_json_schema;
///
/// let schema = json!({
///     "type": "object",
///     "properties": {
///         "name": { "type": "string" },
///         "age": { "type": "integer" },
///         "profile": {
///             "type": "object",
///             "properties": {
///                 "bio": { "type": "string" }
///             }
///         }
///     }
/// });
///
/// let flattened = flatten_json_schema(&schema).unwrap();
/// // Results in attributes with column types and JSON path patterns
/// ```
pub fn flatten_json_schema(schema: &Value) -> Result<Vec<FlattenedAttribute>, String> {
    let mut attributes = Vec::new();

    // First, check if schema has definitions to determine max depth
    // If definitions exist, allow depth of 5, otherwise limit to 3
    let has_definitions = schema.get("definitions").is_some();
    let max_depth = if has_definitions { 5 } else { 3 };

    // Always start navigation from 'properties', not 'definitions'
    // Attributes under 'properties' will be flattened
    // Navigation to 'definitions' only happens when $ref is encountered
    if let Some(properties) = schema.get("properties") {
        process_properties(
            properties,
            "",
            &mut attributes,
            1,
            max_depth,
            schema, // Pass full schema for $ref resolution to definitions
        )?;
    }

    Ok(attributes)
}

/// Recursively processes properties from a JSON schema
///
/// # Arguments
/// * `properties` - The properties object from the JSON schema
/// * `prefix` - Current prefix for nested properties
/// * `attributes` - Mutable vector to collect flattened attributes
/// * `current_depth` - Current nesting depth
/// * `max_depth` - Maximum allowed depth
/// * `root_schema` - The root schema for resolving $ref references
fn process_properties(
    properties: &Value,
    prefix: &str,
    attributes: &mut Vec<FlattenedAttribute>,
    current_depth: usize,
    max_depth: usize,
    root_schema: &Value,
) -> Result<(), String> {
    if let Some(props_obj) = properties.as_object() {
        for (key, value) in props_obj {
            let lowercased_key = key.to_lowercase();
            let current_name = if prefix.is_empty() {
                lowercased_key.clone()
            } else {
                format!("{}_{}", prefix, lowercased_key)
            };

            // Priority 1: Check for $ref - Navigate to definitions only when $ref is encountered
            if let Some(ref_value) = value.get("$ref").and_then(|r| r.as_str()) {
                // Navigate to definitions to resolve the reference
                if let Some(resolved_value) = resolve_ref(ref_value, root_schema) {
                    // Continue flattening the resolved definition's properties
                    if let Some(nested_props) = resolved_value.get("properties") {
                        process_properties(
                            nested_props,
                            &current_name,
                            attributes,
                            current_depth + 1,
                            max_depth,
                            root_schema,
                        )?;
                    }
                }
            }
            // Priority 2: Process properties directly (no $ref) - flatten attributes under properties
            else if let Some(prop_type) = value.get("type").and_then(|t| t.as_str()) {
                match prop_type {
                    "object" => {
                        if current_depth < max_depth {
                            // Process nested object properties
                            if let Some(nested_props) = value.get("properties") {
                                process_properties(
                                    nested_props,
                                    &current_name,
                                    attributes,
                                    current_depth + 1,
                                    max_depth,
                                    root_schema,
                                )?;
                            } else {
                                // Object with no properties - treat as JSONB
                                let (column_type, _) = determine_column_info(value);
                                let pattern = create_generated_column_pattern(
                                    &current_name,
                                    prefix,
                                    &column_type,
                                );
                                attributes.push(FlattenedAttribute {
                                    attribute_name: current_name,
                                    column_type,
                                    generated_column_pattern: pattern,
                                });
                            }
                        } else {
                            // At max depth, treat object as JSONB
                            let (column_type, _) = determine_column_info(value);
                            let pattern = create_generated_column_pattern(
                                &current_name,
                                prefix,
                                &column_type,
                            );
                            attributes.push(FlattenedAttribute {
                                attribute_name: current_name,
                                column_type,
                                generated_column_pattern: pattern,
                            });
                        }
                    }
                    "array" => {
                        // Handle arrays - check if items have $ref to definitions
                        if let Some(items) = value.get("items") {
                            if let Some(ref_value) = items.get("$ref").and_then(|r| r.as_str()) {
                                // Array with $ref items - resolve and flatten the referenced object properties
                                if let Some(resolved_value) = resolve_ref(ref_value, root_schema) {
                                    if let Some(nested_props) = resolved_value.get("properties") {
                                        process_properties(
                                            nested_props,
                                            &current_name,
                                            attributes,
                                            current_depth + 1,
                                            max_depth,
                                            root_schema,
                                        )?;
                                    }
                                }
                            } else {
                                // Array without $ref items - treat as JSONB
                                let (column_type, _) = determine_column_info(value);
                                let pattern = create_generated_column_pattern(
                                    &current_name,
                                    prefix,
                                    &column_type,
                                );
                                attributes.push(FlattenedAttribute {
                                    attribute_name: current_name,
                                    column_type,
                                    generated_column_pattern: pattern,
                                });
                            }
                        } else {
                            // Array with no items definition - treat as JSONB
                            let (column_type, _) = determine_column_info(value);
                            let pattern = create_generated_column_pattern(
                                &current_name,
                                prefix,
                                &column_type,
                            );
                            attributes.push(FlattenedAttribute {
                                attribute_name: current_name,
                                column_type,
                                generated_column_pattern: pattern,
                            });
                        }
                    }
                    _ => {
                        // Leaf property - always include
                        let (column_type, _) = determine_column_info(value);
                        let pattern =
                            create_generated_column_pattern(&current_name, prefix, &column_type);
                        attributes.push(FlattenedAttribute {
                            attribute_name: current_name,
                            column_type,
                            generated_column_pattern: pattern,
                        });
                    }
                }
            } else {
                // No type specified, default to TEXT
                let column_type = "TEXT".to_string();
                let pattern = create_generated_column_pattern(&current_name, prefix, &column_type);
                attributes.push(FlattenedAttribute {
                    attribute_name: current_name,
                    column_type,
                    generated_column_pattern: pattern,
                });
            }
        }
    }
    Ok(())
}

/// Determines the PostgreSQL column type and additional info from a JSON schema property
///
/// # Arguments
/// * `property` - The JSON schema property object
///
/// # Returns
/// * `(String, bool)` - Tuple of (column_type, is_nullable)
fn determine_column_info(property: &Value) -> (String, bool) {
    let property_type = property
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("string");

    let format = property.get("format").and_then(|f| f.as_str());

    let column_type = match (property_type, format) {
        ("string", Some("date")) => "TIMESTAMPTZ".to_string(),
        ("string", Some("date-time")) => "TIMESTAMPTZ".to_string(),
        ("string", _) => "TEXT".to_string(),
        ("integer", _) => "INTEGER".to_string(),
        ("number", _) => "NUMERIC".to_string(),
        ("boolean", _) => "BOOLEAN".to_string(),
        ("array", _) => "JSONB".to_string(),
        ("object", _) => "JSONB".to_string(),
        _ => "TEXT".to_string(), // Default fallback
    };

    (column_type, true) // Assuming nullable by default
}

/// Creates a PostgreSQL JSON path expression for generated columns
///
/// # Arguments
/// * `attribute_name` - The flattened attribute name
/// * `parent_path` - The parent path prefix
/// * `column_type` - The PostgreSQL column type for proper casting
///
/// # Returns
/// * `String` - PostgreSQL JSON path expression with proper type casting
fn create_generated_column_pattern(
    attribute_name: &str,
    parent_path: &str,
    column_type: &str,
) -> String {
    // Determine the final JSON operator based on column type
    let final_operator = if column_type == "JSONB" { "->" } else { "->>" };

    let base_pattern = if parent_path.is_empty() {
        format!("entity_data {} '{}'", final_operator, attribute_name)
    } else {
        // Split the attribute name to create nested JSON path
        let parts: Vec<&str> = attribute_name.split('_').collect();
        if parts.len() == 1 {
            format!("entity_data {} '{}'", final_operator, parts[0])
        } else if parts.len() == 2 {
            format!(
                "entity_data -> '{}' {} '{}'",
                parts[0], final_operator, parts[1]
            )
        } else {
            // For deeper nesting, create a more complex JSON path
            let mut path = "entity_data".to_string();
            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    path = format!("{} {} '{}'", path, final_operator, part);
                } else {
                    path = format!("{} -> '{}'", path, part);
                }
            }
            path
        }
    };

    // Avoid type casting in generated columns to ensure immutability
    // PostgreSQL generated columns must use immutable expressions
    // Type casting operations like ::INTEGER or ::TIMESTAMPTZ are not immutable
    base_pattern
}

/// Resolves a $ref reference to its corresponding definition
///
/// # Arguments
/// * `ref_path` - The reference path (e.g., "#/definitions/Student")
/// * `root_schema` - The root schema containing definitions
///
/// # Returns
/// * `Option<&Value>` - The resolved definition or None if not found
fn resolve_ref<'a>(ref_path: &str, root_schema: &'a Value) -> Option<&'a Value> {
    // Handle JSON Pointer format: "#/definitions/DefinitionName"
    if ref_path.starts_with("#/definitions/") {
        let def_name = ref_path.strip_prefix("#/definitions/")?;
        root_schema.get("definitions")?.get(def_name)
    } else {
        None
    }
}

/// Generates a CREATE TABLE statement from a JSON schema using flattened attributes
///
/// # Arguments
/// * `schema` - The JSON schema to process
///
/// # Returns
/// * `Result<String, String>` - The CREATE TABLE statement or an error message
///
/// # Example
/// ```rust
/// use serde_json::json;
/// use rc_web::projections::schema_projection::generate_create_table_statement;
///
/// let schema = json!({
///     "title": "Student",
///     "type": "object",
///     "properties": {
///         "name": { "type": "string" },
///         "age": { "type": "integer" }
///     }
/// });
///
/// let create_table_sql = generate_create_table_statement(&schema).unwrap();
/// ```
pub fn generate_create_table_statement(schema: &Value) -> Result<String, String> {
    // Extract the title from the schema
    let title = schema
        .get("title")
        .and_then(|t| t.as_str())
        .ok_or("Schema must have a 'title' field")?;

    let table_name = format!("{}_projection", title.to_lowercase());

    // Get flattened attributes from the schema
    let flattened_attributes = flatten_json_schema(schema)?;

    // Start building the CREATE TABLE statement
    let mut create_table_sql = format!("CREATE TABLE {} (\n", table_name);

    // Add the entity_data column to store JSON data
    create_table_sql.push_str("    entity_data JSONB NOT NULL");

    // Add flattened columns as generated columns
    for attribute in flattened_attributes {
        create_table_sql.push_str(",\n");

        // Convert primitive types to TEXT for PostgreSQL immutability
        // Generated columns must use immutable expressions, type casting is not immutable
        let db_column_type = match attribute.column_type.as_str() {
            "INTEGER" | "NUMERIC" | "BOOLEAN" | "DATE" | "TIMESTAMPTZ" => "TEXT",
            _ => &attribute.column_type, // Keep TEXT and JSONB as-is
        };

        create_table_sql.push_str(&format!(
            "    {} {} GENERATED ALWAYS AS ({}) STORED",
            attribute.attribute_name, db_column_type, attribute.generated_column_pattern
        ));
    }

    // Close the CREATE TABLE statement
    create_table_sql.push_str("\n);");

    Ok(create_table_sql)
}

/// Generates CREATE INDEX statements from a JSON schema using flattened attributes and _osConfig
///
/// # Arguments
/// * `schema` - The JSON schema to process
///
/// # Returns
/// * `Result<Vec<String>, String>` - Vector of CREATE INDEX statements or an error message
///
/// # Example
/// ```rust
/// use serde_json::json;
/// use rc_web::projections::schema_projection::generate_index_statements;
///
/// let schema = json!({
///     "title": "Student",
///     "type": "object",
///     "properties": {
///         "name": { "type": "string" }
///     },
///     "_osConfig": {
///         "indexFields": ["name"],
///         "uniqueIndexFields": ["email"]
///     }
/// });
///
/// let index_statements = generate_index_statements(&schema).unwrap();
/// ```
pub fn generate_index_statements(schema: &Value) -> Result<Vec<String>, String> {
    // Extract the title from the schema
    let title = schema
        .get("title")
        .and_then(|t| t.as_str())
        .ok_or("Schema must have a 'title' field")?;

    let table_name = format!("{}_projection", title.to_lowercase());

    // Get flattened attributes from the schema to map field names to column names
    let flattened_attributes = flatten_json_schema(schema)?;

    let mut index_statements = Vec::new();

    // Check if _osConfig exists
    if let Some(os_config) = schema.get("_osConfig") {
        // Process regular index fields
        if let Some(index_fields) = os_config.get("indexFields") {
            if let Some(index_array) = index_fields.as_array() {
                for field in index_array {
                    if let Some(field_name) = field.as_str() {
                        let matching_columns =
                            find_matching_columns(&flattened_attributes, field_name);
                        for column_name in matching_columns {
                            let index_name = format!("idx_{}_{}", table_name, column_name);
                            let index_statement = format!(
                                "CREATE INDEX {} ON {} ({});",
                                index_name, table_name, column_name
                            );
                            index_statements.push(index_statement);
                        }
                    }
                }
            }
        }

        // Process unique index fields
        if let Some(unique_index_fields) = os_config.get("uniqueIndexFields") {
            if let Some(unique_index_array) = unique_index_fields.as_array() {
                for field in unique_index_array {
                    if let Some(field_name) = field.as_str() {
                        let matching_columns =
                            find_matching_columns(&flattened_attributes, field_name);
                        for column_name in matching_columns {
                            let index_name = format!("uidx_{}_{}", table_name, column_name);
                            let index_statement = format!(
                                "CREATE UNIQUE INDEX {} ON {} ({});",
                                index_name, table_name, column_name
                            );
                            index_statements.push(index_statement);
                        }
                    }
                }
            }
        }
    }

    // Always add an index on entity_data for JSON queries
    let json_index_name = format!("idx_{}_entity_data_gin", table_name);
    let json_index_statement = format!(
        "CREATE INDEX {} ON {} USING GIN (entity_data);",
        json_index_name, table_name
    );
    index_statements.push(json_index_statement);

    Ok(index_statements)
}

/// Helper function to find matching column names based on field patterns
///
/// # Arguments
/// * `flattened_attributes` - The flattened attributes from the schema
/// * `field_pattern` - The field pattern to match (supports partial matching)
///
/// # Returns
/// * `Vec<String>` - Vector of matching column names
fn find_matching_columns(
    flattened_attributes: &[FlattenedAttribute],
    field_pattern: &str,
) -> Vec<String> {
    let pattern_lower = field_pattern.to_lowercase();
    let mut matching_columns = Vec::new();

    for attribute in flattened_attributes {
        let attribute_lower = attribute.attribute_name.to_lowercase();

        // Direct match
        if attribute_lower == pattern_lower {
            matching_columns.push(attribute.attribute_name.clone());
        }
        // Pattern matches the end of the attribute name (for nested fields)
        else if attribute_lower.ends_with(&format!("_{}", pattern_lower)) {
            matching_columns.push(attribute.attribute_name.clone());
        }
        // Pattern matches any part of the attribute name
        else if attribute_lower.contains(&pattern_lower) {
            matching_columns.push(attribute.attribute_name.clone());
        }
    }

    matching_columns
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_flatten_simple_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            }
        });

        let result = flatten_json_schema(&schema).unwrap();
        assert_eq!(result.len(), 2);

        let name_attr = result
            .iter()
            .find(|attr| attr.attribute_name == "name")
            .unwrap();
        assert_eq!(name_attr.column_type, "TEXT");
        assert_eq!(name_attr.generated_column_pattern, "entity_data ->> 'name'");

        let age_attr = result
            .iter()
            .find(|attr| attr.attribute_name == "age")
            .unwrap();
        assert_eq!(age_attr.column_type, "INTEGER");
        assert_eq!(age_attr.generated_column_pattern, "entity_data ->> 'age'");
    }

    #[test]
    fn test_generate_create_table_statement_simple() {
        let schema = json!({
            "title": "User",
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            }
        });

        let result = generate_create_table_statement(&schema).unwrap();

        assert!(result.contains("CREATE TABLE user_projection"));
        assert!(result.contains("entity_data JSONB NOT NULL"));
        assert!(result.contains("name TEXT GENERATED ALWAYS AS"));
        assert!(result.contains("age TEXT GENERATED ALWAYS AS"));
        assert!(result.ends_with(");"));
    }

    #[test]
    fn test_generate_index_statements_simple() {
        let schema = json!({
            "title": "User",
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "email": { "type": "string" }
            },
            "_osConfig": {
                "indexFields": ["name"],
                "uniqueIndexFields": ["email"]
            }
        });

        let result = generate_index_statements(&schema).unwrap();

        assert!(result.len() >= 3); // name, email, gin index
        assert!(result
            .iter()
            .any(|stmt| stmt.contains("CREATE INDEX idx_user_projection_name")));
        assert!(result
            .iter()
            .any(|stmt| stmt.contains("CREATE UNIQUE INDEX uidx_user_projection_email")));
        assert!(result
            .iter()
            .any(|stmt| stmt.contains("CREATE INDEX idx_user_projection_entity_data_gin")));
    }
}
