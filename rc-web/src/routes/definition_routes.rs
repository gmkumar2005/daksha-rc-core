use crate::models::ValidateDefRequest;
// use rc_web::{DError, DecisionMaker};
use crate::routes::{
    ErrorResponse, CLIENT_EXAMPLE, CONSULTANT_EXAMPLE, STUDENT_EXAMPLE, TEACHER_EXAMPLE,
};
use crate::{
    base_url, DError, DecisionMaker, SuccessResponse, API_PREFIX, COMMANDS, DEFINITIONS, QUERY,
};
use actix_web::web::{Data, Json, Query};
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use chrono::{DateTime, Utc};
use definitions_core::definitions_domain::{
    generate_id_from_title, read_title, ActivateDefinitionCmd, CreateDefinitionCmd, DefError,
    DefRecordStatus, DomainEvent, ValidateDefinitionCmd,
};
use disintegrate::PersistedEvent;
use disintegrate_postgres::PgEventId;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use std::ops::Deref;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DefinitionsResponse {
    /// Unique identifier
    pub id: String,
    /// Title or name
    pub title: String,
    /// The schema as a JSON string
    pub json_schema_string: String,
    /// Record status (e.g. Active, Inactive)
    pub record_status: DefRecordStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Who created the entry
    pub created_by: String,
    /// Who activated the entry (if any)
    pub activated_by: Option<String>,
}

pub fn routes() -> Scope {
    web::scope("")
        // .service(handlers::admin)
        .service(activate_def)
        .service(validate_def)
        .service(create_def)
        .service(get_definitions)
        .service(get_definitions_by_id)
}

/// Activate a definition
#[utoipa::path(
    post,
    path = "/api/v1/schema/activate_def",
    tags= [DEFINITIONS, COMMANDS],
    responses(
        (status = 200, description = "Activation successful", body = String)
    ),
     request_body(
        content_type = "application/json",
        examples(
            ("Teacher" = (value = json!({"id": "e757aa6e-d39a-2db7-6345-473ddd8aadb2"}), description = "Teacher in Education domain")),
            ("Student" = (value = json!({"id": "1bd23c91-3379-b65b-11cc-64984050e35c"}), description = "Student in Education domain")),
            ("Consultant" = (value = json!({"id": "8568a0b3-8900-e4d2-6e72-e3ad5792288e"}), description = "Consultant in Remote working domain")),
            ("Client" = (value = json!({"id": "fa0f1791-8ddc-5934-fc00-2aff27a84ddf"}), description = "Client in Remote working domain")),
        )
    ),
)]
#[post("/activate_def")]
async fn activate_def(
    decision_maker: Data<DecisionMaker>,
    web_cmd: web::Json<ValidateDefRequest>,
) -> Result<HttpResponse, DError> {
    let identifier = validate_id(&web_cmd)?;
    debug!("Activating def with id: {}", identifier);
    let activate_def_command = ActivateDefinitionCmd {
        id: identifier,
        activated_at: Utc::now(),
        activated_by: "test_activated_by".to_string(),
    };

    let _exec_results: Vec<PersistedEvent<PgEventId, DomainEvent>> =
        decision_maker.make(activate_def_command).await?;

    let response_message = format!(
        "Activation successful for Definition with ID: {}",
        web_cmd.id.as_str()
    );

    Ok(HttpResponse::Ok()
        .append_header((
            "Location",
            format!("{}{API_PREFIX}/schema/{}", base_url(), web_cmd.id.as_str()),
        ))
        .append_header(("message", response_message.clone()))
        .json(SuccessResponse {
            id: web_cmd.id.clone(),
            message: response_message,
        }))
}

fn validate_id(web_cmd: &Json<ValidateDefRequest>) -> Result<Uuid, DError> {
    Uuid::from_str(web_cmd.id.trim()).map_err(|e| {
        DError::from(disintegrate::DecisionError::Domain(DefError::InvalidUUID(
            e.to_string(),
            web_cmd.id.clone(),
        )))
    })
}

/// Validate a definition
#[utoipa::path(
    post,
    path = "/api/v1/schema/validate_def",
    tags= [DEFINITIONS, COMMANDS],
    responses(
        (status = 200, description = "Validation successful", body = String)
    )
)]
#[post("/validate_def")]
async fn validate_def(
    decision_maker: Data<DecisionMaker>,
    web_cmd: web::Json<ValidateDefRequest>,
) -> Result<HttpResponse, DError> {
    let identifier = validate_id(&web_cmd)?;
    debug!("Validating def with id: {}", identifier);
    let validate_def_cmd = ValidateDefinitionCmd {
        id: identifier,
        validated_at: Utc::now(),
        validated_by: "test_validated_by".to_string(),
    };

    let exec_results: Vec<PersistedEvent<PgEventId, DomainEvent>> =
        decision_maker.make(validate_def_cmd).await?;
    let (validation_result, validated_defid) = exec_results
        .iter()
        .find_map(|ev| match ev.deref() {
            DomainEvent::DefValidated {
                validation_result,
                id,
                ..
            } => Some((validation_result, id)),
            _ => None,
        })
        .ok_or_else(|| {
            DError::from(disintegrate::DecisionError::Domain(
                DefError::EventNotFound("DefValidated".to_string()),
            ))
        })?;
    debug!(
        "Validation result of def Definition ID: {} is: {}",
        validated_defid, validation_result
    );

    let response_message = format!(
        "Validation result for Definition with ID {}: is {}.",
        validated_defid, validation_result
    );
    debug!("{}", response_message.clone());
    if validation_result != "Success" {
        return Err(DError::from(disintegrate::DecisionError::Domain(
            DefError::DefinitionNotValid,
        )));
    }

    Ok(HttpResponse::Ok()
        .append_header((
            "Location",
            format!("{}{API_PREFIX}/schema/{}", base_url(), validated_defid),
        ))
        .append_header(("message", response_message.clone()))
        .json(SuccessResponse {
            id: web_cmd.id.clone(),
            message: response_message,
        }))
}

/// Create a definition
#[utoipa::path(
    post,
    path = "/api/v1/schema/create_def",
    tags= [DEFINITIONS, COMMANDS],
     request_body(
        content = String,
        content_type = "application/json",
        examples(
            ("Teacher" = (value = json!(serde_json::from_str::<Value>(TEACHER_EXAMPLE).unwrap()), description = "Teacher in Education domain")),
            ("Student" = (value = json!(serde_json::from_str::<Value>(STUDENT_EXAMPLE).unwrap()), description = "Student in Education domain")),
            ("Consultant" = (value = json!(serde_json::from_str::<Value>(CONSULTANT_EXAMPLE).unwrap()), description = "Consultant in Remote working domain")),
            ("Client" = (value = json!(serde_json::from_str::<Value>(CLIENT_EXAMPLE).unwrap()), description = "Client in Remote working domain")),
        )
    ),
    responses(
        (status = 200, description = "Definition created", body = String),
        (status = 400, description = "Invalid Schema", body = String),
        (status = 409, description = "Definition Already Exists", body = String),
    )
)]
#[post("/create_def")]
async fn create_def(
    decision_maker: Data<DecisionMaker>,
    web_cmd: String,
) -> Result<HttpResponse, DError> {
    let title =
        read_title(&web_cmd).map_err(|e| DError::from(disintegrate::DecisionError::Domain(e)))?;
    let create_def_cmd = CreateDefinitionCmd {
        id: generate_id_from_title(&title),
        title,
        definitions: vec!["test_def".to_string()],
        created_by: "test_created_by".to_string(),
        json_schema_string: web_cmd,
    };

    let exec_results: Vec<PersistedEvent<PgEventId, DomainEvent>> =
        decision_maker.make(create_def_cmd).await?;
    let (created_title, created_defid) = exec_results
        .iter()
        .find_map(|ev| match ev.deref() {
            DomainEvent::DefCreated { title, id, .. } => Some((title, id)),
            _ => None,
        })
        .ok_or_else(|| {
            DError::from(disintegrate::DecisionError::Domain(
                DefError::EventNotFound("DefCreated".to_string()),
            ))
        })?;

    let response_message = format!(
        "Definition created with Id: {} for Title:  {}",
        created_defid, created_title
    );
    debug!("{}", response_message.clone());
    Ok(HttpResponse::Created()
        .append_header((
            "Location",
            format!("{}{API_PREFIX}/schema/{}", base_url(), created_defid),
        ))
        .append_header(("message", response_message.clone()))
        .json(SuccessResponse {
            id: created_defid.to_string(),
            message: response_message,
        }))
}

#[derive(Debug, Serialize, FromRow)]
struct Definition {
    id: Uuid,
    title: String,
    json_schema_string: serde_json::Value,
    record_status: String,
    created_at: chrono::DateTime<Utc>,
    created_by: String,
    activated_by: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams, Default)]
#[into_params(parameter_in = Query)]
pub struct DefinitionQuery {
    #[param(example = "Student")]
    pub title: Option<String>,
    #[param(example = "Active")]
    pub record_status: Option<String>,
}

/// Show all definitions
#[utoipa::path(
    get,
    path = "/api/v1/schema",
    tags= [DEFINITIONS, QUERY],
    params(
        DefinitionQuery
    ),
    responses(
        (status = 200, body = DefinitionsResponse),
        (status = 404, description = "Definition not found", body = ErrorResponse)
    )
)]
#[get("")]
async fn get_definitions(db_pool: Data<PgPool>, query: Query<DefinitionQuery>) -> impl Responder {
    let mut sql = String::from(
        r#"
        SELECT id, title, json_schema_string, record_status, created_at, created_by, activated_by
        FROM definition
        "#,
    );

    let mut conditions = vec![];
    let mut params: Vec<(usize, &str)> = vec![];

    if let Some(title) = &query.title {
        conditions.push(format!("title = ${}", params.len() + 1));
        params.push((params.len() + 1, title));
    }

    if let Some(status) = &query.record_status {
        conditions.push(format!("record_status = ${}", params.len() + 1));
        params.push((params.len() + 1, status));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(" ORDER BY created_at DESC");

    // Dynamically build query
    let mut query_builder = sqlx::query_as::<_, Definition>(&sql);
    for (_idx, val) in params {
        query_builder = query_builder.bind(val);
    }

    match query_builder.fetch_all(db_pool.get_ref()).await {
        Ok(definitions) => HttpResponse::Ok().json(definitions),
        Err(e) => {
            debug!("Database query failed: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: Some("Failed to fetch definitions".into()),
                error_description: Some(format!("Error {} while fetching", e)),
                message: format!("Error {} while fetching", e),
            })
        }
    }
}

/// Show definitions by ID
#[utoipa::path(
    get,
    path = "/api/v1/schema/{id}",
    tags= [DEFINITIONS, QUERY],
    params(
        ("id" = Uuid, Path, description = "ID of the definition to fetch",example = "1bd23c91-3379-b65b-11cc-64984050e35c")
    ),
    responses(
     (status = 200, body = DefinitionsResponse),
     (status = 404, description = "Definition not found", body = ErrorResponse)
    )
)]
#[get("/{id}")]
async fn get_definitions_by_id(db_pool: Data<PgPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    debug!("querying id: {}", id);
    match sqlx::query_as::<_, Definition>(
        r#"
        SELECT id, title, json_schema_string, record_status, created_at, created_by, activated_by
        FROM definition
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db_pool.get_ref())
    .await
    {
        Ok(Some(definition)) => HttpResponse::Ok().json(definition),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: Some("definition_not_found".into()),
            error_description: Some(format!("Definition not found for id {}", id)),
            message: format!("Definition not found for id {}", id),
        }),
        Err(e) => {
            error!("Database query failed: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: Some("Failed to fetch definition or invalid id".into()),
                error_description: Some(format!("Error {} while fetching id {}", e, id)),
                message: format!("Error {} while fetching id {}", e, id),
            })
        }
    }
}
