use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{error, get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Context;
use definitions_core::definitions_domain::*;
use disintegrate::NoSnapshot;
use disintegrate_postgres::{PgDecisionMaker, PgEventStore};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgConnectOptions, PgPool};
use std::env;
use std::ops::Deref;
use utoipa::ToSchema;
use validator::Validate;

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
        match self.source {
            disintegrate::DecisionError::Domain(_) => StatusCode::BAD_REQUEST,
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

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateDefRequest {
    // #[validate(length(min = 3, message = "id is required and must be at least 3 characters"))]
    // pub id: String,
    #[validate(length(
        max = 4096,
        message = "schema is required and must be at less than 4096 characters"
    ))]
    pub schema: String,
}

pub fn get_valid_json_string() -> String {
    r###"
        {
            "title": "test_title",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###
    .to_string()
}

#[post("/create_def")]
async fn create_def(
    decision_maker: Data<DecisionMaker>,
    web_cmd: web::Json<CreateDefRequest>,
) -> Result<HttpResponse, DError> {
    // let generated_def_id = "1234";
    let generated_def_id = generate_id_from_title("test_title_1");
    let create_def_cmd = CreateDefinition {
        def_id: generated_def_id,
        def_title: "test_title_1".to_string(),
        definitions: vec!["test_def".to_string()],
        created_by: "test_created_by".to_string(),
        json_schema_string: web_cmd.schema.clone(),
    };

    let exec_results = decision_maker.make(create_def_cmd).await?;
    let title = exec_results
        .iter()
        .find_map(|ev| match ev.deref() {
            DomainEvent::DefCreated { title, .. } => Some(title),
            _ => None,
        })
        .unwrap();
    println!("title: {}", title);
    Ok(HttpResponse::Created()
        .append_header(("Location", format!("/schema_def/{}", generated_def_id)))
        .append_header(("message", "SchemaDef created"))
        .finish())
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let connect_options = database_url.parse::<PgConnectOptions>()?;
    let pool = PgPool::connect_with(connect_options)
        .await
        .context("Failed to connect to the database")?;
    let serde = disintegrate::serde::json::Json::<DomainEvent>::default();
    let event_store = PgEventStore::new(pool.clone(), serde).await?;
    let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
    Ok(HttpServer::new(move || {
        App::new()
            .app_data(Data::new(decision_maker.clone()))
            .service(echo)
            .service(create_def)
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?)
}
