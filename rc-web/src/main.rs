use actix_web::web;
use actix_web::web::{Data, ServiceConfig};
use anyhow::Context;
use definitions_core::definitions_domain::*;
use disintegrate::NoSnapshot;
use disintegrate_postgres::{PgEventListener, PgEventListenerConfig, PgEventStore};
use log::error;
use rc_web::projections::definitions_read_model;
use rc_web::projections::definitions_read_model::ReadModelProjection;
use rc_web::routes::{api_routes, health_check};
use rc_web::{middleware, COMMANDS, DEFINITIONS, ENTITY, HEALTH, QUERY};
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use std::env;
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
        rc_web::routes::health_check::hello,
        rc_web::routes::health_check::echo,
        rc_web::routes::health_check::healthz,
        rc_web::routes::health_check::readyz,
        rc_web::routes::definition_routes::activate_def,
        rc_web::routes::definition_routes::get_definitions,
        rc_web::routes::definition_routes::get_definitions_by_id,
        rc_web::routes::definition_routes::validate_def,
        rc_web::routes::definition_routes::create_def,
        rc_web::routes::entity_routes::create_entity,
    ),
    tags(
    (name = DEFINITIONS, description = "Manage Definitions and Schemas"),
    (name = QUERY, description = "Order API endpoints"),
    (name = HEALTH, description = "Health API endpoints and Debug"),
    (name = ENTITY, description = "Manage Entities and Entity LifeCycle"),
    (name = COMMANDS, description = "Write side commands which makes updates to the state"),
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

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] shared_pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    // get secret defined in `Secrets.toml` file.
    let client_origin_url = secrets
        .get("CLIENT_ORIGIN_URL")
        .context("CLIENT_ORIGIN_URL was not found")?;
    let serde = disintegrate::serde::json::Json::<DomainEvent>::default();
    let event_store = PgEventStore::new(shared_pool.clone(), serde)
        .await
        .map_err(|e| shuttle_runtime::Error::from(anyhow::Error::new(e)))?;

    let shared_pool_for_web = shared_pool.clone();

    let decision_maker = disintegrate_postgres::decision_maker(event_store.clone(), NoSnapshot);
    let api = ApiDoc::openapi();
    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(Data::new(decision_maker.clone()))
            .app_data(Data::new(shared_pool_for_web.clone()))
            .app_data(Data::new(secrets.clone()))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", api.clone()),
            )
            .service(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            .service(Scalar::with_url("/scalar", api))
            .service(api_routes::routes())
            .service(
                web::scope("")
                    .service(health_check::routes())
                    .wrap(middleware::security_headers::security_headers())
                    .wrap(middleware::logger::logger())
                    .wrap(middleware::cors::cors(&client_origin_url)),
            );
    };

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

    Ok(config.into())
}
