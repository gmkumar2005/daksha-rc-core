// The main entry point of the application. This file starts the Actix-Web server and configures the routes, middleware, and app state.
mod app;
mod command_extractor;
mod db;
mod config;
mod handlers;
mod models;

use crate::app::{application_state_factory_pg, run_migrations};
use actix_web::{App, HttpServer};
use config::AppConfig;
use dotenv::dotenv;
use handlers::schema_def_handlers::{create_def, hello};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_scalar::{Scalar, Servable};
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
    let config = AppConfig::from_env().expect("Failed to load configuration");
    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database.user,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.dbname
    );
    println!("Database URL: {}", &db_url);
    let db_url_clone = db_url.clone();
    tokio::task::spawn_blocking(move || {
        run_migrations(&db_url_clone).expect("RC migrations failed");
    }).await.expect("Failed to run migrations");

    env_logger::init();
    let openapi = ApiDoc::openapi();
    let app_state = application_state_factory_pg(&db_url.clone()).await;
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