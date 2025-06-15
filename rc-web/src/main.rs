use actix_web::web::Data;
use actix_web::{web, App};
use anyhow::Context;
use definitions_core::definitions_domain::*;
use disintegrate::NoSnapshot;
use disintegrate_postgres::{PgEventListener, PgEventListenerConfig, PgEventStore};
use log::error;
use rc_web::projections::definitions_read_model;
use rc_web::projections::definitions_read_model::ReadModelProjection;
use rc_web::routes::{api_routes, health_check};
use rc_web::{middleware, COMMANDS, DEFINITIONS, ENTITY, HEALTH, QUERY};
use sqlx::postgres::PgConnectOptions;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use utoipa::openapi::security::{HttpAuthScheme, SecurityScheme};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    security(("bearer_auth" = [])),
    info(
        title = "RC Web API",
        description = "Rest endpoints for the RC Web API",
    ),
    paths(
        rc_web::routes::definition_routes::create_def,
        rc_web::routes::definition_routes::activate_def,
        rc_web::routes::definition_routes::get_definitions,
        rc_web::routes::definition_routes::get_definitions_by_id,
        rc_web::routes::entity_routes::create_entity,
        rc_web::routes::health_check::hello,
        rc_web::routes::health_check::echo,
        rc_web::routes::health_check::healthz,
        rc_web::routes::health_check::readyz,
    ),
    tags(
    (name = DEFINITIONS, description = "Manage Definitions and Schemas"),
    (name = ENTITY, description = "Manage Entities and Entity LifeCycle"),
    (name = COMMANDS, description = "Write side commands which makes updates to the state"),
    (name = QUERY, description = "Order API endpoints"),
    (name = HEALTH, description = "Health API endpoints and Debug"),


    )
)]
pub struct ApiDoc;

/// Add the bearer_auth security scheme
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.components.as_mut().unwrap().add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                utoipa::openapi::security::HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    // let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let database_url_raw = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let database_url = if database_url_raw.contains("sslmode=") {
        database_url_raw
    } else {
        let sep = if database_url_raw.contains('?') {
            "&"
        } else {
            "?"
        };
        format!("{database_url_raw}{sep}sslmode=disable")
    };
    let connect_options = database_url.parse::<PgConnectOptions>()?;
    let shared_pool = PgPool::connect_with(connect_options)
        .await
        .context("Failed to connect to the database")?;

    let client_origin_url =
        env::var("CLIENT_ORIGIN_URL").context("CLIENT_ORIGIN_URL was not found")?;

    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let bind_port = env::var("PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .context("PORT must be a valid number")?;

    let serde = disintegrate::serde::json::Json::<DomainEvent>::default();
    let event_store = PgEventStore::new(shared_pool.clone(), serde).await?;

    let shared_pool_for_web = Arc::new(shared_pool.clone());
    let decision_maker = Arc::new(disintegrate_postgres::decision_maker(
        event_store.clone(),
        NoSnapshot,
    ));
    let api = Arc::new(ApiDoc::openapi());
    let client_origin_url = Arc::new(client_origin_url);

    let listener_event_store = event_store.clone();
    let listener_pool = shared_pool.clone();

    tokio::spawn(async move {
        let listener = match ReadModelProjection::new(listener_pool).await {
            Ok(listener) => listener,
            Err(e) => {
                error!("Failed to create ReadModelProjection: {}", e);
                return;
            }
        };

        if let Err(e) = PgEventListener::builder(listener_event_store)
            .register_listener(
                listener,
                PgEventListenerConfig::poller(Duration::from_millis(5000)).with_notifier(),
            )
            .start_with_shutdown(definitions_read_model::shutdown())
            .await
        {
            error!("event listener exited with error: {}", e);
        }
    });

    Ok(actix_web::HttpServer::new({
        let shared_pool_for_web = Arc::clone(&shared_pool_for_web);
        let decision_maker = Arc::clone(&decision_maker);
        let api = Arc::clone(&api);
        let client_origin_url = Arc::clone(&client_origin_url);

        move || {
            App::new()
                .app_data(Data::new((*decision_maker).clone()))
                .app_data(Data::new((*shared_pool_for_web).clone()))
                .service(
                    SwaggerUi::new("/swagger-ui/{_:.*}")
                        .url("/api-docs/openapi.json", (*api).clone()),
                )
                .service(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
                .service(Scalar::with_url("/scalar", (*api).clone()))
                .service(api_routes::routes())
                .service(
                    web::scope("")
                        .service(health_check::routes())
                        .wrap(middleware::security_headers::security_headers())
                        .wrap(middleware::logger::logger())
                        .wrap(middleware::cors::cors(&client_origin_url)),
                )
        }
    })
    .bind((bind_address.as_str(), bind_port))?
    .run()
    .await?)
}
