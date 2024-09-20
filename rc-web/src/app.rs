use actix_web::web::Data;
use postgres_es::{default_postgress_pool, PostgresCqrs, PostgresViewRepository};
use std::sync::Arc;
use async_trait::async_trait;
use cqrs_es::{EventEnvelope, Query};
use definitions_manager_lib::schema_def::SchemaDef;
use definitions_manager_lib::schema_def_queries::SchemaDefView;
use crate::db::connection::cqrs_framework;


pub async fn application_state_factory(connection_string: &str) -> Data<ApplicationState> {
    let pool =
        default_postgress_pool(
            connection_string)
            .await;
    let (cqrs, schema_def_query) =
        cqrs_framework(pool);
    Data::new(ApplicationState {
        app_name: String::from("Actix web"),
        cqrs,
        schema_def_query,
    })
}

#[derive(Clone)]
pub struct ApplicationState {
    pub app_name: String,
    pub cqrs: Arc<PostgresCqrs<SchemaDef>>,
    pub schema_def_query: Arc<PostgresViewRepository<SchemaDefView, SchemaDef>>,
}

//
pub struct SimpleLoggingQuery {}

#[async_trait]
impl Query<SchemaDef> for SimpleLoggingQuery {
    async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<SchemaDef>]) {
        for event in events {
            println!("aggregate_id-event.seq: {}-{} -- {:#?} -- metadata {:#?}", aggregate_id, event.sequence, &event.payload, &event.metadata);
        }
    }
}