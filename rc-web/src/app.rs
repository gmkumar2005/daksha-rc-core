use crate::db::connection::{cqrs_framework_pg, cqrs_framework_sqlite};
use actix_web::web::Data;
use anyhow::Error;
use async_trait::async_trait;
use cqrs_es::{Aggregate, AggregateError, EventEnvelope, Query};
use definitions_manager_lib::schema_def::SchemaDef;
use postgres_es::{default_postgress_pool, PostgresCqrs};
use sqlite_es::{default_sqlite_pool, SqliteCqrs};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

pub struct SimpleLoggingQuery {}

#[async_trait]
impl Query<SchemaDef> for SimpleLoggingQuery {
    async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<SchemaDef>]) {
        for event in events {
            println!("aggregate_id-event.seq: {}-{} -- {:#?} -- metadata {:#?}", aggregate_id, event.sequence, &event.payload, &event.metadata);
        }
    }
}

// Define our enum
pub enum CqrsAdapter<A: Aggregate> {
    PostgresAdapter(PostgresCqrs<A>),
    SqliteAdapter(SqliteCqrs<A>),
}
impl<A: Aggregate> CqrsAdapter<A> {
    pub fn append_query(self, query: Box<dyn Query<A>>) -> Self
    {
        match self {
            CqrsAdapter::PostgresAdapter(cqrs) => CqrsAdapter::PostgresAdapter(cqrs.append_query(query)),
            CqrsAdapter::SqliteAdapter(cqrs) => CqrsAdapter::SqliteAdapter(cqrs.append_query(query)),
        }
    }
    pub async fn execute_with_metadata(
        &self,
        aggregate_id: &str,
        command: A::Command,
        metadata: HashMap<String, String>,
    ) -> Result<(), AggregateError<A::Error>> {
        match self {
            CqrsAdapter::PostgresAdapter(cqrs) => cqrs.execute_with_metadata(aggregate_id, command, metadata).await,
            CqrsAdapter::SqliteAdapter(cqrs) => cqrs.execute_with_metadata(aggregate_id, command, metadata).await,
        }
    }
    pub async fn execute(
        &self,
        aggregate_id: &str,
        command: A::Command,
    ) -> Result<(), AggregateError<A::Error>> {
        match self {
            CqrsAdapter::PostgresAdapter(cqrs) => cqrs.execute(aggregate_id, command).await,
            CqrsAdapter::SqliteAdapter(cqrs) => cqrs.execute(aggregate_id, command).await,
        }
    }
    pub fn new_postgres(cqrs: PostgresCqrs<A>) -> Self {
        CqrsAdapter::PostgresAdapter(cqrs)
    }
    pub fn new_sqlite(cqrs: SqliteCqrs<A>) -> Self {
        CqrsAdapter::SqliteAdapter(cqrs)
    }
}

#[derive(Clone)]
pub struct SimpleApplicationState {
    pub app_name: String,
    pub cqrs: Arc<CqrsAdapter<SchemaDef>>,
}

pub async fn application_state_factory_pg(connection_string: &str) -> Data<SimpleApplicationState> {
    let pool =
        default_postgress_pool(
            connection_string)
            .await;
    sqlx::migrate!().run(&pool).await.unwrap();
    let cqrs = cqrs_framework_pg(pool);
    Data::new(SimpleApplicationState {
        app_name: String::from("Actix web"),
        cqrs: Arc::new(CqrsAdapter::new_postgres(cqrs)),
    })
}

pub fn run_migrations(connection_string: &str) -> Result<(), Error> {
    let rt = Runtime::new()?;
    if connection_string.contains("sqlite://") {
        rt.block_on(async {
            let db_pool = default_sqlite_pool(connection_string).await;
            sqlx::migrate!().run(&db_pool).await
        })
    } else {
        rt.block_on(async {
            let db_pool = default_postgress_pool(connection_string).await;
            sqlx::migrate!().run(&db_pool).await
        })
    }?;

    Ok(())
}

pub async fn application_state_factory_sqlite(connection_string: &str) -> Data<SimpleApplicationState> {
    let pool =
        default_sqlite_pool(
            connection_string)
            .await;
    let migration_lock = Arc::new(Mutex::new(()));
    let lock = migration_lock.lock().await;
    sqlx::migrate!().run(&pool).await.unwrap();
    drop(lock);  // Release the lock
    let cqrs = cqrs_framework_sqlite(pool);
    Data::new(SimpleApplicationState {
        app_name: String::from("Actix web"),
        cqrs: Arc::new(CqrsAdapter::new_sqlite(cqrs)),
    })
}