use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use validator::Validate;
use definitions_manager_lib::schema_def_commands::SchemaDefCommand;

#[get("/")]
async fn hello() -> impl Responder {
    "Hello, Actix web!"
}

#[post("/create")]
async fn create() -> impl Responder {
    HttpResponse::Created().body("Resource created successfully!")
}

#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
struct CreateDefRequest {
    #[validate(length(
        min = 3,
        message = "id is required and must be at least 3 characters"
    ))]
    id: String,
    #[validate(length(
        max = 4096,
        message = "schema is required and must be at less than 4096 characters"
    ))]
    schema: String,
}


async fn create_def(data: web::Json<CreateDefRequest>) -> impl Responder {
    let command = SchemaDefCommand::CreateDef {
        id: data.id.clone(),
        schema: data.schema.clone(),
    };
    // Handle the command (e.g., send it to a command handler)
    HttpResponse::Ok().body("CreateDef command received")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(create)
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}