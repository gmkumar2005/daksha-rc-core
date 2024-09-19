// // mod command_extractor;
// use std::collections::HashMap;
// use std::sync::{Arc, Mutex};
// use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
// use actix_web::dev::Payload;
// use actix_web::web::Data;
// use async_trait::async_trait;
// use cqrs_es::{CqrsFramework, EventEnvelope, Query};
// use cqrs_es::mem_store::MemStore;
// use log::debug;
// use serde::{Deserialize, Serialize};
// use validator::Validate;
// use definitions_manager_lib::schema_def::SchemaDef;
// use definitions_manager_lib::schema_def_commands::SchemaDefCommand;
// use definitions_manager_lib::schema_def_services::{SchemaDefServices, SchemaDefServicesApi, SchemaValidationError};
//
// #[get("/")]
// async fn hello(data: web::Data<AppState>) -> String {
//     let app_name = &data.app_name; // <- get app_name
//     format!("Hello, {app_name}!") // <- response with app_name
// }
//
// #[post("/create")]
// async fn create(data: web::Data<AppState>) -> impl Responder {
//     let app_name = &data.app_name;
//     let message = format!("Creating resource for {}", app_name);
//     HttpResponse::Created().body(message)
// }
//
// #[derive(Debug, Serialize, Deserialize, Validate)]
// pub struct CreateDefRequest {
//     #[validate(length(
//         min = 3,
//         message = "id is required and must be at least 3 characters"
//     ))]
//    pub id: String,
//     #[validate(length(
//         max = 4096,
//         message = "schema is required and must be at less than 4096 characters"
//     ))]
//     pub schema: String,
// }
//
//
//
// #[post("/create_def")]
// async fn create_def(data: web::Json<CreateDefRequest>,req: HttpRequest) -> impl Responder {
//     let command = SchemaDefCommand::CreateDef {
//         id: data.id.clone(),
//         schema: data.schema.clone(),
//     };
//     debug!("CreateDef command received: {:#?}", command);
//     if let Some(user_agent) = req.headers().get("User-Agent") {
//         if let Ok(value) = user_agent.to_str() {
//             debug!("User-Agent {}", value);
//         }
//     }
//
//
// // Handle the command (e.g., send it to a command handler)
//     // log the id found in command
//
//     HttpResponse::Ok().body("CreateDef command received")
// }
//
// pub struct AppState {
//     pub app_name: String,
//     pub cqrs: Arc<CqrsFramework<SchemaDef, MemStore<SchemaDef>>>,
// }
//
//
// pub struct SimpleLoggingQuery {}
// #[async_trait]
// impl Query<SchemaDef> for SimpleLoggingQuery {
//     async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<SchemaDef>]) {
//         for event in events {
//             println!("aggregate_id-event.seq: {}-{} -- {:#?} -- metadata {:#?}", aggregate_id, event.sequence, &event.payload, &event.metadata);
//         }
//     }
// }
//
// pub struct MockSchemaDefServices {
//     pub get_user_id: Mutex<Result<(), SchemaValidationError>>,
// }
//
// impl Default for MockSchemaDefServices {
//     fn default() -> Self {
//         Self {
//             get_user_id: Mutex::new(Ok(())),
//         }
//     }
// }
// impl MockSchemaDefServices {
//     pub fn new() -> Self {
//         Self::default()
//     }
//     fn create_def(&self, _id: &str, _schema: &str) -> Result<(), SchemaValidationError> {
//         self.get_user_id.lock().unwrap().clone()
//     }
// }
// #[async_trait]
// impl SchemaDefServicesApi for MockSchemaDefServices {
//     async fn get_user_id(&self, user_id: &str) -> Result<(), SchemaValidationError> {
//         self.get_user_id.lock().unwrap().clone()
//     }
// }
//
// pub async fn get_new_app_state() -> Data<AppState> {
//     let event_store = MemStore::<SchemaDef>::default();
//     let mut metadata = HashMap::new();
//     metadata.insert("time".to_string(), chrono::Utc::now().to_rfc3339());
//     let query = SimpleLoggingQuery {};
//     let services = SchemaDefServices::new(Box::new(MockSchemaDefServices::default()));
//     let cqrs = CqrsFramework::new(event_store, vec![Box::new(query)], services);
//     web::Data::new(AppState {
//         app_name: String::from("Actix web"),
//         cqrs: Arc::new(cqrs),
//     })
// }