use crate::routes::{
    ErrorResponse, CLIENT_JOHN_EXAMPLE, CONSULTANT_SARAH_EXAMPLE, STUDENT_JOHN_EXAMPLE,
    TEACHER_SMITH_EXAMPLE,
};
use crate::{base_url, DError, DecisionMaker, SuccessResponse};
use crate::{API_PREFIX, COMMANDS, ENTITY};
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use definitions_core::definitions_domain::DomainEvent;
use definitions_core::registry_domain::{CreateEntityCmd, EntityError};
use disintegrate::PersistedEvent;
use disintegrate_postgres::PgEventId;
use serde_json::Value;
use std::ops::Deref;
use uuid::Uuid;

pub fn routes() -> Scope {
    web::scope("")
        // .service(handlers::admin)
        .service(create_entity)
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
