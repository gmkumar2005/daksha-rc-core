use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::{error, HttpResponse};
use definitions_core::definitions_domain::{DefError, DomainEvent};
use disintegrate::NoSnapshot;
use disintegrate_postgres::PgDecisionMaker;

pub mod config;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod services;

type DecisionMaker =
    PgDecisionMaker<DomainEvent, disintegrate::serde::json::Json<DomainEvent>, NoSnapshot>;

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct DError {
    #[from]
    source: disintegrate::DecisionError<DefError>,
}

impl error::ResponseError for DError {
    fn status_code(&self) -> StatusCode {
        match &self.source {
            disintegrate::DecisionError::Domain(domain_error) => match domain_error {
                // Add a match arm for `DefinitionAlreadyExists`
                DefError::DefinitionAlreadyExists(..) => StatusCode::CONFLICT, // 409
                _ => StatusCode::BAD_REQUEST,
            },

            disintegrate::DecisionError::EventStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
            disintegrate::DecisionError::StateStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }
}
