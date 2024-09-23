mod common;
mod integration_tests;
use rc_web::config::AppConfig;
use std::env;
use std::sync::{Once, OnceLock};

static INIT: Once = Once::new();
static CONFIG: OnceLock<AppConfig> = OnceLock::new();
static DB_URL: OnceLock<DatabaseUrl> = OnceLock::new();

#[derive(Debug)]
enum DatabaseUrl {
    SqliteInMemory,
    SqliteInDemoDb,
    PostgresRc,
}

impl DatabaseUrl {
    fn value(&self) -> &str {
        match self {
            DatabaseUrl::SqliteInMemory => "sqlite::memory:",
            DatabaseUrl::SqliteInDemoDb => "sqlite://target/demo.db",
            DatabaseUrl::PostgresRc => "postgres://daksha_rc:daksha_rc@localhost:5432/daksha_rc",
        }
    }
}

#[ctor::ctor]
fn init() {
    INIT.call_once(|| {
        env::set_var("RUST_LOG", "debug");
        env::set_var("RUN_ENV", "Testing");
        let _ = env_logger::builder().is_test(true).try_init();
        let config = AppConfig::from_env().expect("Failed to load configuration");
        CONFIG.set(config).expect("Failed to set configuration");
        DB_URL.set(DatabaseUrl::SqliteInMemory).expect("Failed to set database URL");
    });
}

#[cfg(test)]
mod tests {
    use crate::{DatabaseUrl, DB_URL};
    use actix_web::body;
    use actix_web::http::header::ContentType;
    use actix_web::http::StatusCode;
    use actix_web::test::TestRequest;
    use actix_web::{test, App};
    use dotenv::dotenv;
    use hamcrest2::prelude::*;
    use rc_web::app::{application_state_factory_pg, application_state_factory_sqlite};
    use rc_web::config::AppConfig;
    use rc_web::handlers::schema_def_handlers::{create_def, hello, CreateDefRequest};
    use serde_json::Value;

    #[actix_web::test]
    async fn test_index_get() {
        let db_url = DB_URL.get().expect("Database URL not initialized").value();
        let app_state = match DB_URL.get().expect("Database URL not initialized") {
            DatabaseUrl::PostgresRc => application_state_factory_pg(db_url).await,
            _ => application_state_factory_sqlite(db_url).await,
        };

        let app = test::init_service(
            App::new()
                .app_data(app_state.clone()) // Inject state into the test app
                .service(hello)
        ).await;

        let req = TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body = resp.into_body();
        let body_bytes = body::to_bytes(body).await.unwrap();
        let body_string = String::from_utf8(body_bytes.to_vec()).unwrap();

        println!("Body: {:?}", body_string);
        // assert_eq!(body_string, "Hello, Actix web!");
        assert_that!(body_string, is(equal_to("Hello, Actix web!")));
    }

    // Initialize the logger for tests
    #[ctor::ctor]
    fn init() {
        std::env::set_var("RUST_LOG", "debug");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[actix_web::test]
    async fn test_create_def() {
        let db_url = DB_URL.get().expect("Database URL not initialized").value();

        let app_state = match DB_URL.get().expect("Database URL not initialized") {
            DatabaseUrl::PostgresRc => application_state_factory_pg(db_url).await,
            _ => application_state_factory_sqlite(db_url).await,
        };
        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .service(create_def)
        ).await;

        let example_schema = r#"
        {
            "title": "example_schema",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "#
            .to_string();

        let payload = CreateDefRequest {
            id: String::from("example_schema"),
            schema: example_schema,
        };

        let req = TestRequest::post()
            .uri("/create_def")
            .insert_header(ContentType::json())
            .insert_header(("User-Agent", "Mozilla Test"))
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_that!(resp.status(), is(equal_to(StatusCode::CREATED)));

        // check location header
        let location = resp.headers().get("Location").unwrap();
        assert_that!(location, is(equal_to("/schema_def/example_schema")));
    }


    #[actix_web::test]
    async fn test_create_def_with_validation_error() {
        let db_url = DB_URL.get().expect("Database URL not initialized").value();
        let app_state = match DB_URL.get().expect("Database URL not initialized") {
            DatabaseUrl::PostgresRc => application_state_factory_pg(db_url).await,
            _ => application_state_factory_sqlite(db_url).await,
        };
        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .service(create_def)
        ).await;

        let example_schema = r###"
        {
            "title": "Example Faulty Schema,
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###
            .to_string();

        let payload = CreateDefRequest {
            id: String::from("example_faulty_schema"),
            schema: example_schema,
        };

        let req = TestRequest::post()
            .uri("/create_def")
            .insert_header(ContentType::json())
            .insert_header(("User-Agent", "Mozilla Test"))
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_that!(resp.status(), is(equal_to(StatusCode::BAD_REQUEST)));
        let body = test::read_body(resp).await;
        let body_string = String::from_utf8(body.to_vec()).unwrap();
        let json_body: Value = serde_json::from_str(&body_string).unwrap();
        let error_message = json_body.get("ValidationError")
            .and_then(|val| val.get("error_message"))
            .and_then(|val| val.as_str())
            .unwrap();
        assert_that!(error_message, matches_regex(r"Invalid JSON schema.*"));
    }

    #[actix_web::test]
    async fn test_config_files_loading() {
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
        println!("This test case is for visual inspection ensure proper configurations are loaded. \
        The below line should print the database URL defined in the Testing.toml");
        println!("Database URL: {}", db_url);
    }

}