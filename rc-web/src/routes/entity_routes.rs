use crate::{DError, DecisionMaker};
use crate::{API_PREFIX, BASE_URL, COMMANDS, ENTITY};
use actix_web::web::Data;
use actix_web::{get, post, web, HttpResponse, Responder, Scope};
use definitions_core::definitions_domain::DomainEvent;
use definitions_core::registry_domain::{CreateEntityCmd, EntityError};
use disintegrate::PersistedEvent;
use disintegrate_postgres::PgEventId;
use std::ops::Deref;
use uuid::Uuid;
pub fn routes() -> Scope {
    web::scope("")
        // .service(handlers::admin)
        .service(create_entity)
        .service(hello)
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[utoipa::path(
    post,
    path = "/api/v1/entity/{entity_type}",
    tags= [ENTITY, COMMANDS],
    params(
        ("entity_id" = String, Path, description = "Entity type", example = "Student")
    ),
    responses(
        (status = 200, description = "Entity created", body = String),
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
        .append_header(("Location", format!("{BASE_URL}{API_PREFIX}/entity/{}", id)))
        .append_header(("message", response_message))
        .finish())
}
