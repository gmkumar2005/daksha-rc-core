#![deny(clippy::unwrap_used, clippy::panic)]

use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{error, HttpResponse};
use definitions_core::definitions_domain::{DefError, DomainEvent};
use definitions_core::registry_domain::EntityError;
use disintegrate::{DecisionError, NoSnapshot};
use disintegrate_postgres::PgDecisionMaker;
use serde::Serialize;

pub mod config;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod projections;
pub mod routes;
pub mod services;
mod test;

type DecisionMaker =
    PgDecisionMaker<DomainEvent, disintegrate::serde::json::Json<DomainEvent>, NoSnapshot>;

#[derive(thiserror::Error, Debug)]
pub enum DError {
    #[error(transparent)]
    Def(#[from] DecisionError<DefError>),

    #[error(transparent)]
    Entity(#[from] DecisionError<EntityError>),
    // You may have other variants as needed
}

impl error::ResponseError for DError {
    fn status_code(&self) -> StatusCode {
        match &self {
            DError::Def(decision_error) => match decision_error {
                DecisionError::Domain(domain_error) => match domain_error {
                    DefError::DefinitionAlreadyExists(..) => StatusCode::CONFLICT,
                    _ => StatusCode::BAD_REQUEST,
                },

                DecisionError::EventStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
                DecisionError::StateStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            DError::Entity(entity_error) => match entity_error {
                DecisionError::Domain(entity_error) => match entity_error {
                    EntityError::EntityAlreadyExists(..) => StatusCode::CONFLICT,
                    _ => StatusCode::BAD_REQUEST,
                },
                DecisionError::EventStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
                DecisionError::StateStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
}

pub const DEFINITIONS: &str = "1. Definitions";
pub const COMMANDS: &str = "3. Commands";
pub const QUERY: &str = "4. Query";
pub const HEALTH: &str = "5. Health";
pub const ENTITY: &str = "2. Entity";
pub const API_PREFIX: &str = "/api/v1";

pub fn base_url() -> &'static str {
    let is_remote = std::env::var("SHUTTLE_PUBLIC_URL").is_ok();
    if is_remote {
        "https://daksha-ox98.shuttle.app"
    } else {
        "http://localhost:8000"
    }
}

#[derive(Serialize)]
pub struct ErrorMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    pub message: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    pub id: String,
    pub message: String,
}
