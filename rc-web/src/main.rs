use actix_web::web::{Data, ServiceConfig};
use definitions_core::definitions_domain::*;
use disintegrate::NoSnapshot;
use disintegrate_postgres::PgEventStore;
use rc_web::routes::definition_routes::{activate_def, create_def, validate_def};
use rc_web::routes::entity_routes::create_entity;
use rc_web::routes::health_check::{echo, healthz, hello, readyz};
use shuttle_actix_web::ShuttleActixWeb;
use sqlx::PgPool;
use std::env;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] shared_pool: PgPool,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let serde = disintegrate::serde::json::Json::<DomainEvent>::default();
    let event_store = PgEventStore::new(shared_pool.clone(), serde)
        .await
        .map_err(|e| shuttle_runtime::Error::from(anyhow::Error::new(e)))?;

    let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
    let config = move |cfg: &mut ServiceConfig| {
        cfg.app_data(Data::new(decision_maker.clone()))
            .app_data(Data::new(shared_pool.clone()))
            .service(echo)
            .service(healthz)
            .service(readyz)
            .service(create_def)
            .service(hello)
            .service(validate_def)
            .service(create_entity)
            .service(activate_def);
    };

    Ok(config.into())
}
