use std::collections::HashMap;
use std::sync::{Arc};
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use actix_web::web::Data;
use async_trait::async_trait;
use cqrs_es::{AggregateError, CqrsFramework, EventEnvelope, Query};
use cqrs_es::mem_store::MemStore;
use log::debug;
use postgres_es::{default_postgress_pool, PostgresCqrs,PostgresViewRepository};
use serde::{Deserialize, Serialize};
use validator::Validate;
use definitions_manager_lib::schema_def::SchemaDef;
use definitions_manager_lib::schema_def_commands::SchemaDefCommand;
use definitions_manager_lib::schema_def_events::SchemaDefError;
use definitions_manager_lib::schema_def_queries::SchemaDefView;
use definitions_manager_lib::schema_def_services::{SchemaDefServices};
use crate::db::connection::{cqrs_framework, MockSchemaDefServices};

#[get("/")]
async fn hello(app_state: Data<ApplicationState>) -> String {
    let app_name = &app_state.app_name; // <- get app_name
    format!("Hello, {app_name}!") // <- response with app_name
}

#[post("/create")]
async fn create(app_state: Data<ApplicationState>) -> impl Responder {
    let app_name = &app_state.app_name;
    let message = format!("Creating resource for {}", app_name);
    HttpResponse::Created().body(message)
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
async fn create_def(data: web::Json<CreateDefRequest>, req: HttpRequest, app_state: Data<ApplicationState>) -> impl Responder {
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
            HttpResponse::BadRequest().json(schema_def_error)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Execution failed: {}", e)),
    };
    response
}


#[derive(Clone)]
pub struct ApplicationState {
    pub app_name: String,
    pub cqrs: Arc<PostgresCqrs<SchemaDef>>,
    pub schema_def_query: Arc<PostgresViewRepository<SchemaDefView, SchemaDef>>,
}
//
pub struct SimpleLoggingQuery {}
#[async_trait]
impl Query<SchemaDef> for SimpleLoggingQuery {
    async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<SchemaDef>]) {
        for event in events {
            println!("aggregate_id-event.seq: {}-{} -- {:#?} -- metadata {:#?}", aggregate_id, event.sequence, &event.payload, &event.metadata);
        }
    }
}

pub async fn application_state_factory(connection_string: &str) -> Data<ApplicationState> {
    let pool =
        default_postgress_pool(
            connection_string)
            .await;
    let (cqrs, schema_def_query) =
        cqrs_framework(pool);
    Data::new(ApplicationState {
        app_name: String::from("Actix web"),
        cqrs,
        schema_def_query,
    })
}


#[derive(Clone)]
pub struct TestApplicationState {
    pub app_name: String,
    pub cqrs: Arc<CqrsFramework<SchemaDef, MemStore<SchemaDef>>>,
    // pub schema_def_query: Arc<PostgresViewRepository<SchemaDefView, SchemaDef>>,
}
pub async fn in_mem_application_state_factory() -> Data<TestApplicationState> {
    let event_store = MemStore::<SchemaDef>::default();
    let mut metadata = HashMap::new();
    metadata.insert("time".to_string(), chrono::Utc::now().to_rfc3339());
    let services = SchemaDefServices::new(Box::new(MockSchemaDefServices::default()));
    let query = SimpleLoggingQuery {};
    let cqrs_in_mem = CqrsFramework::new(event_store, vec![Box::new(query)], services);
    let cqrs = Arc::new(cqrs_in_mem);
    Data::new(TestApplicationState {
        app_name: String::from("Actix web"),
        cqrs,
        // schema_def_query,
    })
}
