use crate::app::{SimpleApplicationState};
use actix_web::web::Data;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use cqrs_es::AggregateError;
use definitions_manager_lib::schema_def_commands::SchemaDefCommand;
use definitions_manager_lib::schema_def_events::SchemaDefError;
use log::debug;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[get("/")]
async fn hello(app_state: Data<SimpleApplicationState>) -> String {
    let app_name = &app_state.app_name; // <- get app_name
    format!("Hello, {app_name}!") // <- response with app_name
}


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateDefRequest {
    #[validate(length(
        min = 3,
        message = "id is required and must be at least 3 characters"
    ))]
    pub id: String,
    #[validate(length(
        max = 4096,
        message = "schema is required and must be at less than 4096 characters"
    ))]
    pub schema: String,
}


#[post("/create_def")]
async fn create_def(data: web::Json<CreateDefRequest>, req: HttpRequest, app_state: Data<SimpleApplicationState>) -> impl Responder {
    let command = SchemaDefCommand::CreateDef {
        id: data.id.clone(),
        schema: data.schema.clone(),
    };

    debug!("CreateDef command received: {}", command);
    if let Some(user_agent) = req.headers().get("User-Agent") {
        if let Ok(value) = user_agent.to_str() {
            debug!("User-Agent {}", value);
        }
    }


    let response = match app_state.cqrs.execute(&data.id, command).await {
        Ok(_) => HttpResponse::Created()
            .append_header(("Location", format!("/schema_def/{}", data.id)))
            .append_header(("message", "SchemaDef created"))
            .finish(),
        Err(AggregateError::UserError(SchemaDefError::ExistsError { .. })) => {
            HttpResponse::Created()
                .append_header(("Location", format!("/schema_def/{}", data.id)))
                .append_header(("message", "SchemaDef already exists"))
                .finish()
        }
        Err(AggregateError::UserError(schema_def_error)) => {
            log::error!("User error: {}", schema_def_error);
            HttpResponse::BadRequest().json(schema_def_error)
        }
        Err(e) => {
            log::error!("UnexpectedError: {}", e);
            // print type of e
            HttpResponse::InternalServerError().body("An unexpected error occurred while processing your request")},
    };
    response
}

