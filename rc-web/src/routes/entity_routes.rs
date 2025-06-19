use crate::routes::{
    ErrorResponse, CLIENT_JOHN_EXAMPLE, CONSULTANT_SARAH_EXAMPLE, STUDENT_JOHN_EXAMPLE,
    TEACHER_SMITH_EXAMPLE,
};
use crate::{base_url, DError, DecisionMaker, SuccessResponse};
use crate::{API_PREFIX, COMMANDS, ENTITY, QUERY};
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use chrono::{DateTime, Utc};
use definitions_core::definitions_domain::DomainEvent;
use definitions_core::registry_domain::{CreateEntityCmd, EntityError};
use disintegrate::PersistedEvent;
use disintegrate_postgres::PgEventId;
use serde::Serialize;
#[allow(unused_imports)]
use serde_json::{json, Value};
use sqlx::{FromRow, PgPool};
use std::ops::Deref;
use utoipa::ToSchema;
use uuid::Uuid;

/// Helper function to check if error is "table does not exist"
fn is_table_not_found_error(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => db_err.code().map_or(false, |code| code == "42P01"),
        _ => false,
    }
}

/// Logs security events with consistent formatting
fn log_security_event(event_type: &str, details: &str, user_input: &str) {
    log::warn!(
        "[SECURITY] {}: {} | Input: '{}'",
        event_type,
        details,
        user_input
    );
}

/// Sanitizes column names to prevent SQL injection
///
/// This function performs comprehensive sanitization of column names by:
/// - Removing all non-alphanumeric characters except underscores
/// - Preventing SQL keywords and reserved words
/// - Limiting length to prevent buffer overflow attacks
/// - Converting to lowercase for consistency
///
/// # Parameters
/// * `column_name` - The raw column name from user input
///
/// # Returns
/// * `Some(String)` - Sanitized column name if valid
/// * `None` - If column name is invalid or potentially dangerous
fn sanitize_column_name(column_name: &str) -> Option<String> {
    // Remove all characters except alphanumeric and underscores
    let sanitized = column_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>();

    // Check if result is empty
    if sanitized.is_empty() {
        log_security_event(
            "COLUMN_SANITIZATION",
            "Empty column name after sanitization",
            column_name,
        );
        return None;
    }

    // Limit length to prevent buffer overflow
    if sanitized.len() > 64 {
        log_security_event(
            "COLUMN_LENGTH_ATTACK",
            &format!("Column name too long: {} chars", sanitized.len()),
            column_name,
        );
        return None;
    }

    // Convert to lowercase for consistency
    let normalized = sanitized.to_lowercase();

    // Check against SQL reserved words and dangerous patterns
    let sql_keywords = [
        "select",
        "insert",
        "update",
        "delete",
        "drop",
        "create",
        "alter",
        "truncate",
        "union",
        "join",
        "where",
        "order",
        "group",
        "having",
        "limit",
        "offset",
        "declare",
        "exec",
        "execute",
        "sp_",
        "xp_",
        "sys",
        "information_schema",
        "pg_",
        "mysql",
        "sqlite_",
        "master",
        "msdb",
        "tempdb",
        "model",
    ];

    if sql_keywords.contains(&normalized.as_str()) {
        log_security_event(
            "SQL_KEYWORD_BLOCKED",
            &format!("Attempted to use SQL keyword '{}' as column", normalized),
            column_name,
        );
        return None;
    }

    // Block patterns that start with dangerous prefixes
    if normalized.starts_with("pg_")
        || normalized.starts_with("sys")
        || normalized.starts_with("sp_")
        || normalized.starts_with("xp_")
    {
        log_security_event(
            "DANGEROUS_COLUMN_PREFIX",
            &format!("Blocked dangerous column prefix in '{}'", normalized),
            column_name,
        );
        return None;
    }

    Some(normalized)
}

/// Sanitizes and formats SQL values based on their detected type
///
/// This function provides comprehensive value sanitization by:
/// - Detecting value types (numeric, boolean, UUID, string)
/// - Applying appropriate escaping and formatting for each type
/// - Preventing SQL injection through proper quoting and escaping
/// - Validating format for structured types (UUID, boolean)
///
/// # Parameters
/// * `value` - The raw value from user input
///
/// # Returns
/// * `Some(String)` - Properly formatted SQL value
/// * `None` - If value is invalid or potentially dangerous
fn sanitize_sql_value(value: &str) -> Option<String> {
    // Limit value length to prevent buffer overflow
    if value.len() > 1000 {
        log_security_event(
            "VALUE_LENGTH_ATTACK",
            &format!("Value too long: {} characters", value.len()),
            &value[..50.min(value.len())],
        );
        return None;
    }

    // Check for suspicious patterns that might indicate SQL injection attempts
    let dangerous_patterns = [
        "--",
        "/*",
        "*/",
        "@@",
        "char(",
        "cast(",
        "convert(",
        "exec(",
        "sp_",
        "xp_",
        "union",
        "select",
        "insert",
        "update",
        "delete",
        "drop",
        "create",
        "alter",
        "truncate",
        "script",
        "javascript",
        "vbscript",
        "onload",
        "onerror",
        "eval(",
    ];

    let value_lower = value.to_lowercase();
    for pattern in &dangerous_patterns {
        if value_lower.contains(pattern) {
            log_security_event(
                "SQL_INJECTION_ATTEMPT",
                &format!("Blocked dangerous pattern '{}'", pattern),
                value,
            );
            return None;
        }
    }

    // Try to parse as numeric (integer or float)
    if let Ok(num) = value.parse::<f64>() {
        // Additional validation for numeric values
        if num.is_finite() && num.abs() < 1e15 {
            return Some(value.to_string());
        } else {
            log_security_event(
                "INVALID_NUMERIC",
                "Numeric value out of safe range or not finite",
                value,
            );
            return None;
        }
    }

    // Try to parse as boolean
    match value.to_lowercase().as_str() {
        "true" => return Some("true".to_string()),
        "false" => return Some("false".to_string()),
        _ => {}
    }

    // Try to parse as UUID
    if uuid::Uuid::parse_str(value).is_ok() {
        // UUIDs are safe and don't need escaping beyond quoting
        return Some(format!("'{}'", value));
    }

    // Handle as string value with comprehensive escaping
    let escaped_value = value
        .replace('\\', "\\\\") // Escape backslashes first
        .replace('\'', "''") // Escape single quotes (SQL standard)
        .replace('"', "\"\"") // Escape double quotes
        .replace('\0', "") // Remove null bytes
        .replace('\r', "") // Remove carriage returns
        .replace('\n', " ") // Replace newlines with spaces
        .replace('\t', " "); // Replace tabs with spaces

    // Final check - ensure the escaped value isn't empty
    if escaped_value.trim().is_empty() {
        log_security_event(
            "VALUE_SANITIZATION",
            "Empty value after sanitization",
            value,
        );
        return None;
    }

    Some(format!("'{}'", escaped_value))
}

/// Validates that an entity type exists in the definitions table
///
/// This function performs a database query to check if the provided entity type
/// exists as a title in the definitions table. This validation ensures that only
/// valid, defined entity types can be used to query projection tables.
///
/// # Parameters
/// * `db_pool` - Database connection pool reference
/// * `entity_type` - The entity type string to validate (e.g., "Student", "Teacher")
///
/// # Returns
/// * `Ok(true)` - Entity type exists in definitions table
/// * `Ok(false)` - Entity type does not exist in definitions table
/// * `Err(sqlx::Error)` - Database query failed
///
/// # Examples
/// ```rust
/// let exists = validate_entity_type(&db_pool, "Student").await?;
/// if exists {
///     // Proceed with entity operations
/// } else {
///     // Return 404 error for invalid entity type
/// }
/// ```
///
/// # Security
/// Uses parameterized queries to prevent SQL injection attacks.
/// The entity type parameter is safely bound to the SQL query.
///
/// # Database Query
/// Executes: `SELECT COUNT(*) FROM definitions WHERE title = $1`
async fn validate_entity_type(db_pool: &PgPool, entity_type: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM definitions WHERE title = $1")
        .bind(entity_type)
        .fetch_one(db_pool)
        .await?;

    Ok(result > 0)
}

/// Entity record from projection table
#[derive(Debug, Serialize, FromRow, ToSchema)]
struct Entity {
    /// Unique identifier for the entity
    id: Uuid,
    /// The entity data as JSON
    entity_data: serde_json::Value,
    /// Type of the entity (e.g., Student, Teacher)
    entity_type: String,
    /// Who created the entity
    created_by: String,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// ID of the registry definition used
    registry_def_id: Uuid,
    /// Version of the registry definition used
    registry_def_version: i32,
}

pub fn routes() -> Scope {
    web::scope("")
        // .service(handlers::admin)
        .service(create_entity)
        .service(get_entities)
        .service(get_entity_by_id)
        .service(hello)
}

/// Respond with "Hello world!"
#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

/// Create an entity for a given entity type
#[utoipa::path(
    post,
    path = "/api/v1/entity/{entity_type}",
    tags= [ENTITY, COMMANDS],
    request_body(
        content = String,
        content_type = "application/json",
        examples(
            ("Teacher_smith" = (value = json!(serde_json::from_str::<Value>(TEACHER_SMITH_EXAMPLE).expect("Failed to parse TEACHER_SMITH_EXAMPLE JSON")), description = "Teacher in Education domain")),
            ("Student_john" = (value = json!(serde_json::from_str::<Value>(STUDENT_JOHN_EXAMPLE).expect("Failed to parse STUDENT_JOHN_EXAMPLE JSON")), description = "Student in Education domain")),
            ("Consultant_sarah" = (value = json!(serde_json::from_str::<Value>(CONSULTANT_SARAH_EXAMPLE).expect("Failed to parse CONSULTANT_SARAH_EXAMPLE JSON")), description = "Consultant in Remote working domain")),
            ("Client_john" = (value = json!(serde_json::from_str::<Value>(CLIENT_JOHN_EXAMPLE).expect("Failed to parse CLIENT_JOHN_EXAMPLE JSON")), description = "Client in Remote working domain")),
        )
    ),
    params(
        ("entity_type" = String, Path, description = "Entity type", example = "Student")
    ),
    responses(
        (status = 200, description = "Entity created", body = String),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 409, description = "Entity Already Exists", body = String),
    )
)]
#[post("/{entity_type}")]
async fn create_entity(
    decision_maker: Data<DecisionMaker>,
    entity_type: web::Path<String>,
    web_cmd: web::Json<serde_json::Value>,
) -> Result<HttpResponse, DError> {
    let create_entity_cmd = CreateEntityCmd {
        id: Uuid::now_v7(),
        entity_body: web_cmd.to_string(),
        entity_type: entity_type.into_inner(),
        created_by: "demo".to_string(),
    };

    let exec_results: Vec<PersistedEvent<PgEventId, DomainEvent>> =
        decision_maker.make(create_entity_cmd).await?;

    let (id, registry_def_id, registry_def_version, entity_type) = exec_results
        .iter()
        .find_map(|ev| match ev.deref() {
            DomainEvent::EntityCreated {
                id,
                registry_def_id,
                registry_def_version,
                entity_type,
                ..
            } => Some((id, registry_def_id, registry_def_version, entity_type)),
            _ => None,
        })
        .ok_or_else(|| {
            DError::from(disintegrate::DecisionError::Domain(
                EntityError::EventNotFound("EntityCreated".to_string()),
            ))
        })?;

    let response_message = format!(
        "Entity created with ID: {}, definition used {} version {} for entity type {} ",
        id,
        registry_def_id,
        registry_def_version.get(),
        entity_type
    );

    Ok(HttpResponse::Ok()
        .append_header((
            "Location",
            format!("{}{API_PREFIX}/entity/{}", base_url(), id),
        ))
        .append_header(("message", response_message))
        .json(SuccessResponse {
            id: id.to_string(),
            message: format!("Entity created for Entity type: {}", entity_type),
        }))
}

/// Get entities by entity type with optional filtering
///
/// This endpoint retrieves entities from the projection table for a given entity type.
/// You can filter results using query parameters that map to column names.
/// Multiple filters are combined using AND conditions.
///
/// # Filtering
/// Accepts arbitrary name=value pairs as query parameters. All filters are combined using AND conditions.
/// Values are automatically detected as numeric, boolean, UUID, or string types and handled appropriately.
/// Column names are sanitized by removing non-alphanumeric characters except underscores.
///
/// # Examples
/// - `/api/v1/entity/Student` - Get all students
/// - `/api/v1/entity/Student?created_by=demo` - Get students created by 'demo' (string filter)
/// - `/api/v1/entity/Student?registry_def_version=1` - Get students using definition version 1 (numeric filter)
/// - `/api/v1/entity/Student?age=20` - Filter by age (numeric filter)
/// - `/api/v1/entity/Student?grade_point=3.5` - Filter by GPA (decimal filter)
/// - `/api/v1/entity/Student?active=true` - Filter by active status (boolean filter)
/// - `/api/v1/entity/Student?registry_def_id=123e4567-e89b-12d3-a456-426614174000` - Filter by UUID
/// - `/api/v1/entity/Student?created_by=demo&registry_def_version=1&active=true` - Multiple filters with different types
/// - `/api/v1/entity/Student?name=John O'Connor` - String with special characters (automatically escaped)
///
/// # Security
/// - Entity types are validated against the definitions table
/// - Column names are comprehensively sanitized and validated against SQL keywords
/// - Values are sanitized based on type detection with multiple layers of protection
/// - SQL injection patterns are detected and blocked
/// - String values are properly escaped with multiple escape mechanisms
/// - Input length limits prevent buffer overflow attacks
#[utoipa::path(
    get,
    path = "/api/v1/entity/{entity_type}",
    tags= [ENTITY, QUERY],
    summary = "Get entities by type with filtering",
    description = "Retrieves entities from the projection table for a given entity type with optional filtering capabilities. Accepts arbitrary query parameters as filters (e.g., ?created_by=demo&name=John&age=20). All filters are combined using AND conditions.",
    params(
        ("entity_type" = String, Path, description = "The type of entity to retrieve (e.g., Student, Teacher, Client)", example = "Student")
    ),

    responses(
        (status = 200,
         description = "Successfully retrieved entities",
         body = Vec<Entity>,
         examples(
             ("empty_list" = (
                 summary = "No entities found",
                 description = "When no entities match the criteria",
                 value = json!([])
             )),
             ("student_list" = (
                 summary = "List of students",
                 description = "Example response with student entities",
                 value = json!([
                     {
                         "id": "123e4567-e89b-12d3-a456-426614174000",
                         "entity_data": {
                             "name": "John Doe",
                             "grade": "A",
                             "age": 20
                         },
                         "entity_type": "Student",
                         "created_by": "demo",
                         "created_at": "2024-01-15T10:30:00Z",
                         "registry_def_id": "def-123e4567-e89b-12d3-a456-426614174000",
                         "registry_def_version": 1
                     }
                 ])
             ))
         )
        ),
        (status = 404,
         description = "Entity type not found",
         body = ErrorResponse,
         examples(
             ("invalid_entity_type" = (
                 summary = "Entity type not defined",
                 description = "When the entity type doesn't exist in the definitions table",
                 value = json!({
                     "error": "INVALID_ENTITY_TYPE",
                     "error_description": "Entity type 'InvalidType' not found in definitions",
                     "message": "Entity type invalid"
                 })
             )),
             ("table_not_found" = (
                 summary = "Projection table does not exist",
                 description = "When the entity type's projection table hasn't been created",
                 value = json!({
                     "error": "TABLE_NOT_FOUND",
                     "error_description": "Projection table 'invalidtype_projection' does not exist",
                     "message": "Entity type 'InvalidType' not found. The projection table may not have been created yet."
                 })
             ))
         )
        ),
        (status = 500,
         description = "Internal server error",
         body = ErrorResponse,
         examples(
             ("database_error" = (
                 summary = "Database connection error",
                 description = "When there's a database connectivity issue",
                 value = json!({
                     "error": "DATABASE_ERROR",
                     "error_description": "Database error: connection timeout",
                     "message": "Failed to fetch entities"
                 })
             ))
         )
        ),
    )
)]
#[get("/{entity_type}")]
async fn get_entities(
    db_pool: Data<PgPool>,
    entity_type: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, DError> {
    let entity_type_str = entity_type.into_inner();

    // Validate that the entity type exists in definitions table
    match validate_entity_type(db_pool.get_ref(), &entity_type_str).await {
        Ok(exists) => {
            if !exists {
                return Ok(HttpResponse::NotFound().json(ErrorResponse {
                    error: Some("INVALID_ENTITY_TYPE".to_string()),
                    error_description: Some(format!(
                        "Entity type '{}' not found in definitions",
                        entity_type_str
                    )),
                    message: "Entity type invalid".to_string(),
                }));
            }
        }
        Err(e) => {
            log::error!("Failed to validate entity type: {}", e);
            return Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: Some("DATABASE_ERROR".to_string()),
                error_description: Some(format!("Failed to validate entity type: {}", e)),
                message: "Failed to validate entity type".to_string(),
            }));
        }
    }

    let table_name = format!("{}_projection", entity_type_str.to_lowercase());

    let mut sql = format!(
        "SELECT id, entity_data, entity_type, created_by, created_at, registry_def_id, registry_def_version FROM {}",
        table_name
    );

    let mut conditions = vec![];

    // Add filters from query parameters with comprehensive sanitization
    let mut blocked_filters = 0;
    let mut applied_filters = 0;

    for (key, value) in query.iter() {
        // Sanitize column name
        if let Some(sanitized_column) = sanitize_column_name(key) {
            // Sanitize value
            if let Some(sanitized_value) = sanitize_sql_value(value) {
                conditions.push(format!("{} = {}", sanitized_column, sanitized_value));
                applied_filters += 1;
                log::debug!("Applied filter: {} = {}", sanitized_column, sanitized_value);
            } else {
                blocked_filters += 1;
                log_security_event(
                    "VALUE_REJECTED",
                    &format!("Invalid value for column '{}'", key),
                    value,
                );
            }
        } else {
            blocked_filters += 1;
            log_security_event("COLUMN_REJECTED", "Invalid column name", key);
        }
    }

    // Log summary of filter processing
    if blocked_filters > 0 {
        log::warn!(
            "[SECURITY] Filter summary: {} applied, {} blocked",
            applied_filters,
            blocked_filters
        );
    }

    // Add WHERE clause if there are conditions
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    // Add ORDER BY clause for consistent results
    sql.push_str(" ORDER BY created_at DESC, id ASC");

    log::debug!("Executing SQL query: {}", sql);

    match sqlx::query_as::<_, Entity>(&sql)
        .fetch_all(db_pool.get_ref())
        .await
    {
        Ok(entities) => {
            log::info!(
                "Successfully retrieved {} entities of type '{}' with {} filters",
                entities.len(),
                entity_type_str,
                applied_filters
            );
            Ok(HttpResponse::Ok().json(entities))
        }
        Err(e) => {
            log::error!(
                "Database error for entity type '{}': {}",
                entity_type_str,
                e
            );
            if is_table_not_found_error(&e) {
                // Table doesn't exist - return 404
                Ok(HttpResponse::NotFound().json(ErrorResponse {
                    error: Some("TABLE_NOT_FOUND".to_string()),
                    error_description: Some(format!("Projection table '{}' does not exist", table_name)),
                    message: format!("Entity type '{}' not found. The projection table may not have been created yet.", entity_type_str),
                }))
            } else {
                // Log potential SQL injection attempt if query fails suspiciously
                if e.to_string().contains("syntax error") || e.to_string().contains("invalid") {
                    log_security_event(
                        "SUSPICIOUS_QUERY_FAILURE",
                        &format!("Query failed with syntax/validation error: {}", e),
                        &sql,
                    );
                }

                // Other database errors - return 500
                Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                    error: Some("DATABASE_ERROR".to_string()),
                    error_description: Some(format!("Database error: {}", e)),
                    message: "Failed to fetch entities".to_string(),
                }))
            }
        }
    }
}

/// Get entity by ID for a given entity type
///
/// This endpoint retrieves a specific entity by its ID from the projection table
/// for a given entity type. The entity ID must be a valid UUID.
///
/// # Examples
/// - `/api/v1/entity/Student/123e4567-e89b-12d3-a456-426614174000` - Get specific student
/// - `/api/v1/entity/Teacher/456e7890-e12b-34d5-a678-901234567890` - Get specific teacher
/// - `/api/v1/entity/Client/789abcdef-0123-4567-8901-23456789abcd` - Get specific client
///
/// # Error Scenarios
/// - Returns 404 if the entity type is not defined in the definitions table
/// - Returns 404 if the entity ID doesn't exist in the specified entity type table
/// - Returns 404 if the entity type's projection table doesn't exist
/// - Returns 500 for database connectivity or permission errors
#[utoipa::path(
    get,
    path = "/api/v1/entity/{entity_type}/{id}",
    tags= [ENTITY, QUERY],
    summary = "Get entity by ID",
    description = "Retrieves a specific entity by its ID from the projection table for a given entity type.",
    params(
        ("entity_type" = String, Path, description = "The type of entity (e.g., Student, Teacher, Client)", example = "Student"),
        ("id" = String, Path, description = "The unique identifier of the entity (UUID format)", example = "123e4567-e89b-12d3-a456-426614174000")
    ),
    responses(
        (status = 200,
         description = "Entity found successfully",
         body = Entity,
         examples(
             ("student_found" = (
                 summary = "Student entity retrieved",
                 description = "Example response when a student entity is found",
                 value = json!({
                     "id": "123e4567-e89b-12d3-a456-426614174000",
                     "entity_data": {
                         "name": "John Doe",
                         "grade": "A",
                         "age": 20,
                         "email": "john.doe@example.com"
                     },
                     "entity_type": "Student",
                     "created_by": "demo",
                     "created_at": "2024-01-15T10:30:00Z",
                     "registry_def_id": "def-123e4567-e89b-12d3-a456-426614174000",
                     "registry_def_version": 1
                 })
             )),
             ("teacher_found" = (
                 summary = "Teacher entity retrieved",
                 description = "Example response when a teacher entity is found",
                 value = json!({
                     "id": "456e7890-e12b-34d5-a678-901234567890",
                     "entity_data": {
                         "name": "Jane Smith",
                         "subject": "Mathematics",
                         "department": "Science"
                     },
                     "entity_type": "Teacher",
                     "created_by": "admin",
                     "created_at": "2024-01-14T09:15:00Z",
                     "registry_def_id": "def-456e7890-e12b-34d5-a678-901234567890",
                     "registry_def_version": 2
                 })
             ))
         )
        ),
        (status = 404,
         description = "Entity not found or entity type does not exist",
         body = ErrorResponse,
         examples(
             ("entity_not_found" = (
                 summary = "Entity ID not found",
                 description = "When the entity ID doesn't exist in the specified table",
                 value = json!({
                     "error": "NOT_FOUND",
                     "error_description": "Entity not found",
                     "message": "Entity with ID 123e4567-e89b-12d3-a456-426614174000 not found for type: Student"
                 })
             )),
             ("invalid_entity_type" = (
                 summary = "Entity type not defined",
                 description = "When the entity type doesn't exist in the definitions table",
                 value = json!({
                     "error": "INVALID_ENTITY_TYPE",
                     "error_description": "Entity type 'InvalidType' not found in definitions",
                     "message": "Entity type invalid"
                 })
             )),
             ("table_not_found" = (
                 summary = "Entity type table not found",
                 description = "When the entity type's projection table doesn't exist",
                 value = json!({
                     "error": "TABLE_NOT_FOUND",
                     "error_description": "Projection table 'invalidtype_projection' does not exist",
                     "message": "Entity type 'InvalidType' not found. The projection table may not have been created yet."
                 })
             ))
         )
        ),
        (status = 500,
         description = "Internal server error",
         body = ErrorResponse,
         examples(
             ("database_error" = (
                 summary = "Database connection error",
                 description = "When there's a database connectivity or permission issue",
                 value = json!({
                     "error": "DATABASE_ERROR",
                     "error_description": "Database error: connection refused",
                     "message": "Failed to fetch entity"
                 })
             ))
         )
        ),
    )
)]
#[get("/{entity_type}/{id}")]
async fn get_entity_by_id(
    db_pool: Data<PgPool>,
    path: web::Path<(String, Uuid)>,
) -> Result<HttpResponse, DError> {
    let (entity_type_str, entity_id) = path.into_inner();

    // Validate that the entity type exists in definitions table
    match validate_entity_type(db_pool.get_ref(), &entity_type_str).await {
        Ok(exists) => {
            if !exists {
                return Ok(HttpResponse::NotFound().json(ErrorResponse {
                    error: Some("INVALID_ENTITY_TYPE".to_string()),
                    error_description: Some(format!(
                        "Entity type '{}' not found in definitions",
                        entity_type_str
                    )),
                    message: "Entity type invalid".to_string(),
                }));
            }
        }
        Err(e) => {
            log::error!("Failed to validate entity type: {}", e);
            return Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                error: Some("DATABASE_ERROR".to_string()),
                error_description: Some(format!("Failed to validate entity type: {}", e)),
                message: "Failed to validate entity type".to_string(),
            }));
        }
    }

    let table_name = format!("{}_projection", entity_type_str.to_lowercase());

    let sql = format!(
        "SELECT id, entity_data, entity_type, created_by, created_at, registry_def_id, registry_def_version FROM {} WHERE id = $1",
        table_name
    );

    match sqlx::query_as::<_, Entity>(&sql)
        .bind(entity_id)
        .fetch_optional(db_pool.get_ref())
        .await
    {
        Ok(Some(entity)) => Ok(HttpResponse::Ok().json(entity)),
        Ok(None) => Ok(HttpResponse::NotFound().json(ErrorResponse {
            error: Some("NOT_FOUND".to_string()),
            error_description: Some("Entity not found".to_string()),
            message: format!(
                "Entity with ID {} not found for type: {}",
                entity_id, entity_type_str
            ),
        })),
        Err(e) => {
            log::error!("Database error: {}", e);
            if is_table_not_found_error(&e) {
                // Table doesn't exist - return 404
                Ok(HttpResponse::NotFound().json(ErrorResponse {
                    error: Some("TABLE_NOT_FOUND".to_string()),
                    error_description: Some(format!("Projection table '{}' does not exist", table_name)),
                    message: format!("Entity type '{}' not found. The projection table may not have been created yet.", entity_type_str),
                }))
            } else {
                // Other database errors - return 500
                Ok(HttpResponse::InternalServerError().json(ErrorResponse {
                    error: Some("DATABASE_ERROR".to_string()),
                    error_description: Some(format!("Database error: {}", e)),
                    message: "Failed to fetch entity".to_string(),
                }))
            }
        }
    }
}
