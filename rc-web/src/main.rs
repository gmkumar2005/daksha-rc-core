// The main entry point of the application. This file starts the Actix-Web server and configures the routes, middleware, and app state.
mod app;
mod command_extractor;

use actix_web::{App, HttpServer};
use rc_web::handlers::schema_def_handlers::{hello, create, application_state_factory, create_def};
use dotenv::dotenv;
use rc_web::config::AppConfig;


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
    let app_state = application_state_factory(&db_url).await;
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(hello)
            .service(create)
            .service(create_def)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}