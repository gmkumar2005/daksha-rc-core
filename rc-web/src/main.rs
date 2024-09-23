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
use handlers::schema_def_handlers::{create_def, hello};
use crate::app::application_state_factory_pg;

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
    let app_state = application_state_factory_pg(&db_url).await;
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(hello)
            .service(create_def)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}