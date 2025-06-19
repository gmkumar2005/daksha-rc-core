use crate::routes::{
    ErrorResponse, CLIENT_JOHN_EXAMPLE, CONSULTANT_SARAH_EXAMPLE, STUDENT_JOHN_EXAMPLE,
    TEACHER_SMITH_EXAMPLE,
};
use crate::{base_url, DError, DecisionMaker, SuccessResponse};
use crate::{API_PREFIX, COMMANDS, ENTITY, QUERY};
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse, Responder, Scope};

use definitions_core::definitions_domain::DomainEvent;
use definitions_core::registry_domain::{CreateEntityCmd, EntityError};
use disintegrate::PersistedEvent;
use disintegrate_postgres::PgEventId;
use serde::Serialize;
use serde_json::Value;
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

/// Get entities by entity type
#[utoipa::path(
    get,
    path = "/api/v1/entity/{entity_type}",
    tags= [ENTITY, QUERY],
    params(
        ("entity_type" = String, Path, description = "Entity type", example = "Student")
    ),
    responses(
        (status = 200, description = "List of entities", body = Vec<Entity>),
        (status = 404, description = "Entity type not found - projection table does not exist", body = ErrorResponse),
        (status = 500, description = "Internal server error - database error", body = ErrorResponse),
    )
)]
#[get("/{entity_type}")]
async fn get_entities(
    db_pool: Data<PgPool>,
    entity_type: web::Path<String>,
) -> Result<HttpResponse, DError> {
    let entity_type_str = entity_type.into_inner();
    let table_name = format!("{}_projection", entity_type_str.to_lowercase());

    let sql = format!(
        "SELECT id, entity_data, entity_type, created_by, registry_def_id, registry_def_version FROM {}",
        table_name
    );

    match sqlx::query_as::<_, Entity>(&sql)
        .fetch_all(db_pool.get_ref())
        .await
    {
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
#[utoipa::path(
    get,
    path = "/api/v1/entity/{entity_type}/{id}",
    tags= [ENTITY, QUERY],
    params(
        ("entity_type" = String, Path, description = "Entity type", example = "Student"),
        ("id" = String, Path, description = "Entity ID", example = "123e4567-e89b-12d3-a456-426614174000")
    ),
    responses(
        (status = 200, description = "Entity found", body = Entity),
        (status = 404, description = "Entity not found or entity type does not exist", body = ErrorResponse),
        (status = 500, description = "Internal server error - database error", body = ErrorResponse),
    )
)]
#[get("/{entity_type}/{id}")]
async fn get_entity_by_id(
    db_pool: Data<PgPool>,
    path: web::Path<(String, Uuid)>,
) -> Result<HttpResponse, DError> {
    let (entity_type_str, entity_id) = path.into_inner();
    let table_name = format!("{}_projection", entity_type_str.to_lowercase());

    let sql = format!(
        "SELECT id, entity_data, entity_type, created_by, registry_def_id, registry_def_version FROM {} WHERE id = $1",
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
