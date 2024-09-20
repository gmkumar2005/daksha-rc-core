use crate::app::SimpleLoggingQuery;
use async_trait::async_trait;
use cqrs_es::persist::GenericQuery;
use cqrs_es::Query;
use definitions_manager_lib::schema_def::SchemaDef;
use definitions_manager_lib::schema_def_queries::SchemaDefView;
use definitions_manager_lib::schema_def_services::{SchemaDefServices, SchemaDefServicesApi, SchemaValidationError};
use postgres_es::{
    PostgresCqrs, PostgresViewRepository};
use sqlx::{Pool, Postgres};
use std::sync::{Arc, Mutex};
// pub async fn configure_repo() -> PostgresEventRepository {
//     let connection_string = "postgresql://daksha_rc:daksha_rc@localhost:5432/daksha_rc";
//     let pool: Pool<Postgres> = default_postgress_pool(connection_string).await;
//     PostgresEventRepository::new(pool)
// }

// type PgViewRepository = PostgresViewRepository<SchemaDefView, SchemaDef>;
//
// pub fn configure_view_repository(db_pool: Pool<Postgres>) -> PgViewRepository {
//     PostgresViewRepository::new("schema_def_view", db_pool)
// }

pub type SchemaDefQuery = GenericQuery<
    PostgresViewRepository<SchemaDefView, SchemaDef>,
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


pub fn cqrs_framework(
    pool: Pool<Postgres>,
) -> (
    Arc<PostgresCqrs<SchemaDef>>,
    Arc<PostgresViewRepository<SchemaDefView, SchemaDef>>,
) {
    let schema_def_view_repo = Arc::new(PostgresViewRepository::new("schema_def_view", pool.clone()));
    // let mut schema_def_query = SchemaDefQuery::new(schema_def_view_repo.clone());
    // schema_def_query.use_error_handler(Box::new(|e| error!("{}", e)));
    // TODO: Fix error handling
    let schema_def_query = SchemaDefQuery::new(schema_def_view_repo.clone());
    // .use_error_handler(Box::new(|e| error!("{}", e)));
    let simple_query = SimpleLoggingQuery {};
    let queries: Vec<Box<dyn Query<SchemaDef>>> =
        vec![Box::new(schema_def_query), Box::new(simple_query)];
    let services = SchemaDefServices::new(Box::new(MockSchemaDefServices::default()));
    (
        Arc::new(postgres_es::postgres_cqrs(pool, queries, services)),
        schema_def_view_repo,
    )
}