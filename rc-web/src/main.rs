use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{error, get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Context;
use chrono::Utc;
use definitions_core::definitions_domain::*;
use disintegrate::{NoSnapshot, PersistedEvent};
use disintegrate_postgres::{PgDecisionMaker, PgEventId, PgEventStore};
use log::debug;
use rc_web::models::ValidateDefRequest;
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::{postgres::PgConnectOptions, PgPool};
use std::env;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

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

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[post("/activate_def")]
async fn activate_def(
    decision_maker: Data<DecisionMaker>,
    web_cmd: web::Json<ValidateDefRequest>,
) -> Result<HttpResponse, DError> {
    let validate_def_cmd = ActivateDefinitionCmd {
        id: Uuid::from_str(web_cmd.def_id.as_str()).unwrap(),
        activated_at: Utc::now(),
        activated_by: "test_activated_by".to_string(),
    };

    let exec_results: Vec<PersistedEvent<PgEventId, DomainEvent>> =
        decision_maker.make(validate_def_cmd).await?;
    let validated_defid = exec_results
        .iter()
        .find_map(|ev| match ev.deref() {
            DomainEvent::DefActivated { id: def_id, .. } => Some(def_id),
            _ => None,
        })
        .unwrap();
    debug!(
        "Activation successful for Definition with ID: {}",
        validated_defid
    );

    let response_message = format!(
        "Activation successful for Definition with ID: {}",
        validated_defid
    );

    Ok(HttpResponse::Ok()
        .append_header(("Location", format!("/schema_def/{}", validated_defid)))
        .append_header(("message", response_message))
        .finish())
}

#[post("/validate_def")]
async fn validate_def(
    decision_maker: Data<DecisionMaker>,
    web_cmd: web::Json<ValidateDefRequest>,
) -> Result<HttpResponse, DError> {
    let validate_def_cmd = ValidateDefinitionCmd {
        id: Uuid::from_str(web_cmd.def_id.as_str()).unwrap(),
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
                id: def_id,
                ..
            } => Some((validation_result, def_id)),
            _ => None,
        })
        .unwrap();
    debug!(
        "Validation result of def Definition ID: {} is: {}",
        validated_defid, validation_result
    );

    let response_message = format!(
        "Validation result for Definition with ID {}: is {}.",
        validated_defid, validation_result
    );

    if validation_result != "Success" {
        return Err(DError::from(disintegrate::DecisionError::Domain(
            DefError::DefinitionNotValid,
        )));
    }

    Ok(HttpResponse::Ok()
        .append_header(("Location", format!("/schema_def/{}", validated_defid)))
        .append_header(("message", response_message))
        .finish())
}

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
            DomainEvent::DefCreated {
                title, id: def_id, ..
            } => Some((title, def_id)),
            _ => None,
        })
        .unwrap();
    debug!(
        "Created def with id: {} and title: {}",
        created_defid, created_title
    );
    let response_message = format!(
        "SchemaDef created with id: {} for title:  {}",
        created_defid, created_title
    );
    Ok(HttpResponse::Created()
        .append_header(("Location", format!("/schema_def/{}", created_defid)))
        .append_header(("message", response_message))
        .finish())
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] shared_pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let serde = disintegrate::serde::json::Json::<DomainEvent>::default();
    let event_store = PgEventStore::new(shared_pool.clone(), serde)
        .await
        .map_err(|e| shuttle_runtime::Error::from(anyhow::Error::new(e)))?;

    let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(Data::new(decision_maker.clone()))
            .service(echo)
            .service(create_def)
            .service(hello)
            .service(validate_def)
            .service(activate_def);
    };

    Ok(config.into())
}
