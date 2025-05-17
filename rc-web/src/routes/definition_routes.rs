use crate::models::ValidateDefRequest;
// use rc_web::{DError, DecisionMaker};
use crate::{DError, DecisionMaker};
use actix_web::web::{Data, Query};
use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::Utc;
use definitions_core::definitions_domain::{
    generate_id_from_title, read_title, ActivateDefinitionCmd, CreateDefinitionCmd, DefError,
    DomainEvent, ValidateDefinitionCmd,
};
use disintegrate::PersistedEvent;
use disintegrate_postgres::PgEventId;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
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

#[derive(Debug, Deserialize)]
pub struct DefinitionQuery {
    title: Option<String>,
    record_status: Option<String>,
}

#[get("/definitions")]
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
            eprintln!("Database query failed: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch definitions")
        }
    }
}

#[get("/definitions/{id}")]
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
        Ok(None) => HttpResponse::NotFound().body(format!("No definition found for id {}", id)),
        Err(e) => {
            error!("Database query failed: {}", e);
            HttpResponse::NotFound().body("Failed to fetch definition")
        }
    }
}
