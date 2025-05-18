use actix_web::web::{Data, ServiceConfig};
use definitions_core::definitions_domain::*;
use disintegrate::NoSnapshot;
use disintegrate_postgres::{PgEventListener, PgEventListenerConfig, PgEventStore};
use log::error;
use rc_web::projections::definitions_read_model;
use rc_web::projections::definitions_read_model::ReadModelProjection;
use rc_web::routes::definition_routes::{
    activate_def, create_def, get_definitions, get_definitions_by_id, validate_def,
};
use rc_web::routes::entity_routes::create_entity;
use rc_web::routes::health_check::{echo, healthz, hello, readyz};
use rc_web::routes::{api_routes, definition_routes, entity_routes, health_check};
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::PgPool;
use std::env;
use std::time::Duration;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] shared_pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let serde = disintegrate::serde::json::Json::<DomainEvent>::default();
    let event_store = PgEventStore::new(shared_pool.clone(), serde)
        .await
        .map_err(|e| shuttle_runtime::Error::from(anyhow::Error::new(e)))?;

    let shared_pool_for_web = shared_pool.clone();

    let decision_maker = disintegrate_postgres::decision_maker(event_store.clone(), NoSnapshot);
    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(Data::new(decision_maker.clone()))
            .app_data(Data::new(shared_pool_for_web.clone()))
            .service(api_routes::routes())
            .service(health_check::routes());
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
