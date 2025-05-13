use crate::models::ValidateDefRequest;
// use rc_web::{DError, DecisionMaker};
use crate::{DError, DecisionMaker};
use actix_web::web::Data;
use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use definitions_core::definitions_domain::{
    generate_id_from_title, read_title, ActivateDefinitionCmd, CreateDefinitionCmd, DefError,
    DomainEvent, ValidateDefinitionCmd,
};
use disintegrate::PersistedEvent;
use disintegrate_postgres::PgEventId;
use log::debug;
use std::ops::Deref;
use std::str::FromStr;
use uuid::Uuid;

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

    let _exec_results: Vec<PersistedEvent<PgEventId, DomainEvent>> =
        decision_maker.make(validate_def_cmd).await?;

    let response_message = format!(
        "Activation successful for Definition with ID: {}",
        web_cmd.def_id.as_str()
    );

    Ok(HttpResponse::Ok()
        .append_header((
            "Location",
            format!("/schema_def/{}", web_cmd.def_id.as_str()),
        ))
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
