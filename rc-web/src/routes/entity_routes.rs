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
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_json::{json, Value};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use std::ops::Deref;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Helper function to check if error is "table does not exist"
fn is_table_not_found_error(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Database(db_err) => db_err.code().map_or(false, |code| code == "42P01"),
        _ => false,
    }
}

/// Whitelist of allowed column names for filtering
/// This prevents SQL injection and ensures only valid columns are queried
const ALLOWED_FILTER_COLUMNS: &[&str] = &[
    "id",
    "entity_data",
    "entity_type",
    "created_by",
    "created_at",
    "registry_def_id",
    "registry_def_version",
    "version",
];

/// Validates and sanitizes column name for filtering
fn validate_column_name(column_name: &str) -> Option<String> {
    // Remove any non-alphanumeric characters except underscores
    let sanitized = column_name.replace(|c: char| !c.is_alphanumeric() && c != '_', "");

    // Check if the sanitized column name is in the allowed list
    if ALLOWED_FILTER_COLUMNS.contains(&sanitized.as_str()) {
        Some(sanitized)
    } else {
        log::warn!("Attempted to filter by invalid column: {}", column_name);
        None
    }
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

/// Query parameters for filtering entities
///
/// Supports filtering by standard entity columns and custom attributes.
/// All filters are applied using AND conditions.
#[derive(Debug, Deserialize, IntoParams, Default)]
#[into_params(parameter_in = Query)]
pub struct EntityQuery {
    /// Filter by entity_type (exact match)
    #[param(example = "Student")]
    pub entity_type: Option<String>,
    /// Filter by created_by (exact match)
    #[param(example = "demo")]
    pub created_by: Option<String>,
    /// Filter by registry_def_id (exact match)
    #[param(example = "123e4567-e89b-12d3-a456-426614174000")]
    pub registry_def_id: Option<String>,
    /// Filter by registry_def_version (exact match)
    #[param(example = "1")]
    pub registry_def_version: Option<i32>,
    /// Additional filters as key-value pairs mapping to column names.
    /// Column names are sanitized to prevent SQL injection.
    /// Example: ?name=John&age=25&grade=A
    #[serde(flatten)]
    pub additional_filters: HashMap<String, String>,
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
///
/// **Standard Filters:**
/// - `entity_type` - Filter by entity type (exact match)
/// - `created_by` - Filter by creator (exact match)
/// - `registry_def_id` - Filter by registry definition ID (exact match)
/// - `registry_def_version` - Filter by definition version (exact match)
///
/// **Custom Filters:**
/// Any additional query parameters are treated as column filters.
/// Only whitelisted column names are allowed for security.
///
/// # Examples
/// - `/api/v1/entity/Student` - Get all students
/// - `/api/v1/entity/Student?created_by=demo` - Get students created by 'demo'
/// - `/api/v1/entity/Student?registry_def_version=1` - Get students using definition version 1
/// - `/api/v1/entity/Student?created_by=demo&registry_def_version=1` - Multiple standard filters
/// - `/api/v1/entity/Teacher?entity_type=Teacher&created_by=admin` - Filter teachers by creator
/// - `/api/v1/entity/Student?id=123e4567-e89b-12d3-a456-426614174000` - Get student by ID (alternative to ID endpoint)
///
/// # Security
/// - Entity types are validated against the definitions table
/// - Column names are validated against a whitelist to prevent SQL injection
/// - Invalid column names are logged and ignored
#[utoipa::path(
    get,
    path = "/api/v1/entity/{entity_type}",
    tags= [ENTITY, QUERY],
    summary = "Get entities by type with filtering",
    description = "Retrieves entities from the projection table for a given entity type with optional filtering capabilities.",
    params(
        ("entity_type" = String, Path, description = "The type of entity to retrieve (e.g., Student, Teacher, Client)", example = "Student"),
        EntityQuery
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
    query: web::Query<EntityQuery>,
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
    let mut bind_values: Vec<String> = vec![];

    // Add filters based on query parameters
    if let Some(entity_type_filter) = &query.entity_type {
        conditions.push(format!("entity_type = ${}", bind_values.len() + 1));
        bind_values.push(entity_type_filter.clone());
    }

    if let Some(created_by) = &query.created_by {
        conditions.push(format!("created_by = ${}", bind_values.len() + 1));
        bind_values.push(created_by.clone());
    }

    if let Some(registry_def_id) = &query.registry_def_id {
        conditions.push(format!("registry_def_id = ${}", bind_values.len() + 1));
        bind_values.push(registry_def_id.clone());
    }

    if let Some(registry_def_version) = &query.registry_def_version {
        conditions.push(format!("registry_def_version = ${}", bind_values.len() + 1));
        bind_values.push(registry_def_version.to_string());
    }

    // Add additional filters from the flattened HashMap
    for (key, value) in &query.additional_filters {
        // Validate and sanitize column name to prevent SQL injection
        if let Some(validated_column) = validate_column_name(key) {
            // Skip if this column was already processed in the standard filters
            if ![
                "entity_type",
                "created_by",
                "registry_def_id",
                "registry_def_version",
            ]
            .contains(&validated_column.as_str())
            {
                conditions.push(format!("{} = ${}", validated_column, bind_values.len() + 1));
                bind_values.push(value.clone());
            }
        }
    }

    // Add WHERE clause if there are conditions
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    // Add ORDER BY clause for consistent results
    sql.push_str(" ORDER BY created_at DESC, id ASC");

    // Build the query with parameter binding
    let mut query_builder = sqlx::query_as::<_, Entity>(&sql);
    for value in &bind_values {
        query_builder = query_builder.bind(value);
    }

    match query_builder.fetch_all(db_pool.get_ref()).await {
        Ok(entities) => Ok(HttpResponse::Ok().json(entities)),
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
