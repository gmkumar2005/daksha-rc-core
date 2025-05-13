use crate::integration_tests::create_def_cmd_1;
use definitions_core::definitions_domain::{generate_id_from_title, DomainEvent};
use disintegrate::NoSnapshot;
use disintegrate_postgres::PgEventStore;
use sqlx::{query, PgPool, Row, Transaction};
use std::ops::Deref;
use testcontainers::{ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};
use tokio::sync::OnceCell;

// Shared pool across tests
static POOL: OnceCell<PgPool> = OnceCell::const_new();
static POSTGRES_CONTAINER: OnceCell<ContainerAsync<Postgres>> = OnceCell::const_new();
// let container: ContainerAsync<Postgres>

// Initialize the shared PostgreSQL container
async fn initialize_container() -> ContainerAsync<Postgres> {
    let container = postgres::Postgres::default()
        .with_tag("17.2-bookworm")
        .start()
        .await
        .unwrap();
    container
}

// Get the shared PostgreSQL container
async fn get_postgres_container<'a>() -> &'a ContainerAsync<Postgres> {
    let container = POSTGRES_CONTAINER.get_or_init(initialize_container).await;
    container
}

// Initialize the shared connection pool
async fn initialize_pool() -> PgPool {
    let container = get_postgres_container().await;
    let host_port = container.get_host_port_ipv4(5432).await.unwrap();
    let connection_string =
        &format!("postgres://postgres:postgres@127.0.0.1:{host_port}/postgres",);
    let pool = PgPool::connect(&connection_string).await.unwrap();
    pool
}

// Get the shared connection pool (singleton)
async fn get_shared_pool() -> PgPool {
    POOL.get_or_init(|| async { initialize_pool().await })
        .await
        .clone()
}
// let txn: Transaction<Postgres>
async fn begin_transaction(pool: &PgPool) -> anyhow::Result<Transaction<sqlx::postgres::Postgres>> {
    // let mut conn = pool.acquire().await?;
    // let txn = conn.begin().await?;
    Ok(pool.begin().await?)
}

#[tokio::test]
#[cfg(feature = "integration_tests")]
async fn test_with_postgres() -> anyhow::Result<()> {
    let container = postgres::Postgres::default().start().await?;
    let host_port = container.get_host_port_ipv4(5432).await?;
    let connection_string =
        &format!("postgres://postgres:postgres@127.0.0.1:{host_port}/postgres",);

    // Create a connection pool
    let pool = PgPool::connect(&connection_string).await?;
    // Run the SQL query `SELECT 1 = 1` and fetch its result
    let result = query("SELECT 1 = 1 AS result").fetch_one(&pool).await?;

    let outcome: bool = result.get("result");

    // Assert that the query returned `true`
    assert!(outcome);
    Ok(())
}

#[tokio::test]
#[cfg(feature = "integration_tests")]
async fn test_with_shared_pool() -> anyhow::Result<()> {
    let pool = get_shared_pool().await;
    let mut tx = begin_transaction(&pool).await?;

    // let result = query("SELECT 1 = 1 AS result").fetch_one(&pool).await?;
    let result = query("SELECT 1 = 1 AS result").fetch_one(&mut *tx).await?;

    let outcome: bool = result.get("result");

    // Assert that the query returned `true`
    assert!(outcome);
    Ok(())
}

#[tokio::test]
#[cfg(feature = "integration_tests")]
async fn test_with_shared_pool_2() -> anyhow::Result<()> {
    let pool = get_shared_pool().await;
    let mut tx = begin_transaction(&pool).await?;
    let result = query("SELECT 1 = 1 AS result").fetch_one(&mut *tx).await?;

    let outcome: bool = result.get("result");

    // Assert that the query returned `true`
    assert!(outcome);
    Ok(())
}

#[tokio::test]
#[cfg(feature = "integration_tests")]
async fn test_create_definition_with_postgres() -> anyhow::Result<()> {
    let pool = get_shared_pool().await;
    let serde = disintegrate::serde::json::Json::<DomainEvent>::default();
    let event_store = PgEventStore::new(pool.clone(), serde).await?;
    let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);
    let create_def_cmd = create_def_cmd_1().clone();
    let exec_results = decision_maker.make(create_def_cmd).await?;

    let (title, def_id) = exec_results
        .iter()
        .find_map(|ev| match ev.deref() {
            DomainEvent::DefCreated { title, id, .. } => Some((title, id)),
            _ => None,
        })
        .unwrap();

    // assert_eq!(title, "test_title");
    assert_eq!(
        title, "test_title",
        "Expected title to be 'test_title', but got '{}'",
        title
    );
    assert_eq!(
        def_id.to_string(),
        generate_id_from_title("test_title").to_string()
    );

    Ok(())
}
