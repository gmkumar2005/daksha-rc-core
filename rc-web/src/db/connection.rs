use crate::app::SimpleLoggingQuery;
use async_trait::async_trait;
use cqrs_es::persist::GenericQuery;
use cqrs_es::Query;
use definitions_manager_lib::schema_def::SchemaDef;
use definitions_manager_lib::schema_def_queries::SchemaDefView;
use definitions_manager_lib::schema_def_services::{SchemaDefServices, SchemaDefServicesApi, SchemaValidationError};
use postgres_es::{
    PostgresCqrs, PostgresViewRepository};
use sqlite_es::{SqliteCqrs, SqliteViewRepository};
use sqlx::{Pool, Postgres, Sqlite};
use std::sync::{Arc, Mutex};


pub type SchemaDefQueryPg = GenericQuery<
    PostgresViewRepository<SchemaDefView, SchemaDef>,
    SchemaDefView,
    SchemaDef>;

pub type SchemaDefQuerySqlite = GenericQuery<
    SqliteViewRepository<SchemaDefView, SchemaDef>,
    SchemaDefView,
    SchemaDef>;

pub struct MockSchemaDefServices {
    pub get_user_id: Mutex<Result<(), SchemaValidationError>>,
}

impl Default for MockSchemaDefServices {
    fn default() -> Self {
        Self {
            get_user_id: Mutex::new(Ok(())),
        }
    }
}


#[async_trait]
impl SchemaDefServicesApi for MockSchemaDefServices {
    async fn get_user_id(&self, _user_id: &str) -> Result<(), SchemaValidationError> {
        self.get_user_id.lock().unwrap().clone()
    }
}


pub fn cqrs_framework_pg(
    pool: Pool<Postgres>,
) ->
    PostgresCqrs<SchemaDef>
{
    let schema_def_view_repo = Arc::new(PostgresViewRepository::new("schema_def_view", pool.clone()));
    // TODO: Fix error handling
    let schema_def_query = SchemaDefQueryPg::new(schema_def_view_repo.clone());
    // .use_error_handler(Box::new(|e| error!("{}", e)));
    let simple_query = SimpleLoggingQuery {};
    let queries: Vec<Box<dyn Query<SchemaDef>>> =
        vec![Box::new(schema_def_query), Box::new(simple_query)];
    let services = SchemaDefServices::new(Box::new(MockSchemaDefServices::default()));
    postgres_es::postgres_cqrs(pool, queries, services)
}

pub fn cqrs_framework_sqlite(
    pool: Pool<Sqlite>,
) ->
    SqliteCqrs<SchemaDef>
{
    let schema_def_view_repo = Arc::new(SqliteViewRepository::new("schema_def_view", pool.clone()));
    // TODO: Fix error handling
    let schema_def_query = SchemaDefQuerySqlite::new(schema_def_view_repo.clone());
    // .use_error_handler(Box::new(|e| error!("{}", e)));
    let simple_query = SimpleLoggingQuery {};
    let queries: Vec<Box<dyn Query<SchemaDef>>> =
        vec![Box::new(schema_def_query), Box::new(simple_query)];
    let services = SchemaDefServices::new(Box::new(MockSchemaDefServices::default()));
    sqlite_es::sqlite_cqrs(pool, queries, services)
}