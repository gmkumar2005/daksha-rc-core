// use sqlx::postgres::PgPoolOptions;
// use sqlx::{query, PgPool, Row};
use sqlx::{query, PgPool, Row, Transaction};
use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};
use tokio::sync::OnceCell;

// Shared pool across tests
static POOL: OnceCell<PgPool> = OnceCell::const_new();
static POSTGRES_CONTAINER: OnceCell<ContainerAsync<Postgres>> = OnceCell::const_new();
// let container: ContainerAsync<Postgres>

// Initialize the shared PostgreSQL container
async fn initialize_container() -> ContainerAsync<Postgres> {
    let container = postgres::Postgres::default().start().await.unwrap();
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

// Each test uses a separate transaction
// async fn run_in_transaction<F>(test_fn: F) -> anyhow::Result<()>
// where
//     F: Fn(Transaction<'_, sqlx::Postgres>) -> anyhow::Result<()> + Send + Sync,
// {
//     let pool = get_shared_pool().await;
//
//     // Begin a new transaction
//     let mut txn = pool.begin().await?;
//     let result = test_fn(txn).await;
//
//     // Rollback the transaction after the test to maintain isolation
//     txn.rollback().await?;
//     result
// }

#[tokio::test]
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
async fn test_with_shared_pool_2() -> anyhow::Result<()> {
    let pool = get_shared_pool().await;
    let mut tx = begin_transaction(&pool).await?;
    let result = query("SELECT 1 = 1 AS result").fetch_one(&mut *tx).await?;

    let outcome: bool = result.get("result");

    // Assert that the query returned `true`
    assert!(outcome);
    Ok(())
}
