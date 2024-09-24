// The main entry point of the application. This file starts the Actix-Web server and configures the routes, middleware, and app state.
mod app;
mod command_extractor;
mod db;
mod config;
mod handlers;
mod models;

use actix_web::{App, HttpServer};
use config::AppConfig;
use dotenv::dotenv;
use utoipa::OpenApi;
use utoipa::openapi::OpenApiBuilder;
use utoipa_rapidoc::RapiDoc;
use utoipa_scalar::{Scalar, Servable};
use handlers::schema_def_handlers::{create_def, hello};
use crate::app::application_state_factory_pg;
use utoipa_swagger_ui::SwaggerUi;


#[derive(OpenApi)]
#[openapi(
    info(description = "Daksha-RC-core API"),
    paths(
        handlers::schema_def_handlers::create_def,
    ),
    components(schemas(handlers::schema_def_handlers::CreateDefRequest))
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    dotenv().ok();
    // Load the configuration
    let config = AppConfig::from_env().expect("Failed to load configuration");
    // Build the connection URL
    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database.user,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.dbname
    );
    println!("Database URL: {}", db_url);
    env_logger::init();
    let openapi = ApiDoc::openapi();
    let app_state = application_state_factory_pg(&db_url).await;
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(hello)
            .service(create_def)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .service(Scalar::with_url("/scalar", openapi.clone()))
            .service(RapiDoc::with_openapi("/api-docs/openapi2.json", openapi.clone()).path("/rapidoc"))
            .service(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}