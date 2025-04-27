// use once_cell::sync::Lazy;
// use sqlx::PgPool;
// use std::sync::Arc;
// use testcontainers_modules::postgres as postgresc;
// use testcontainers_modules::testcontainers::runners::AsyncRunner;
// use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};
// use tokio::sync::Mutex;
// use tokio_postgres::NoTls;
//
// type SharedPostgres = Arc<Mutex<ContainerAsync<postgresc::Postgres>>>;
//
// static POSTGRES_CONTAINER: Lazy<SharedPostgres> = Lazy::new(|| {
//     let container = tokio::runtime::Runtime::new().unwrap().block_on(async {
//         postgresc::Postgres::default()
//             .with_tag("17.2-bookworm")
//             .with_container_name("bookworm_test")
//             .start().await.unwrap()
//     });
//     Arc::new(Mutex::new(container))
// });
// async fn setup_lazy_pool() -> SharedPostgres {
//     let container = postgresc::Postgres::default()
//         .with_tag("17.2-bookworm")
//         .with_container_name("bookworm_test")
//         .start().await.unwrap();
//     Arc::new(Mutex::new(container))
// }
//
// async fn setup_pool() -> PgPool {
//     let container = &*POSTGRES_CONTAINER.lock().await;
//     let host_ip = container.get_host().await;
//     let host_port = container.get_host_port_ipv4(5432).await.unwrap();
//     // prepare connection string
//     let connection_string = &format!(
//         "postgres://postgres:postgres@127.0.0.1:{}/postgres",
//         host_port
//     );
//
//     let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
//         .await
//         .unwrap();
//     tokio::spawn(async move {
//         if let Err(e) = connection.await {
//             eprintln!("connection error: {}", e);
//         }
//     });
//
//     let db: PgPool = PgPool::connect(&connection_string).await.unwrap();
//
//     db
// }
// async fn setup() -> PgPool {
//     let container = postgresc::Postgres::default()
//         .with_tag("17.2-bookworm")
//         .with_container_name("bookworm_test")
//         .start()
//         .await
//         .unwrap();
//     let host_ip = container.get_host().await;
//     let host_port = container.get_host_port_ipv4(5432).await.unwrap();
//     // prepare connection string
//     let connection_string = &format!(
//         "postgres://postgres:postgres@127.0.0.1:{}/postgres",
//         host_port
//     );
//
//     let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
//         .await
//         .unwrap();
//     tokio::spawn(async move {
//         if let Err(e) = connection.await {
//             eprintln!("connection error: {}", e);
//         }
//     });
//
//     let db: PgPool = PgPool::connect(&connection_string).await.unwrap();
//
//     db
// }
//
// #[tokio::test]
// async fn test() {
//     // let pool = setup().await;
//
//     let container = postgresc::Postgres::default()
//         .with_tag("17.2-bookworm")
//         .with_container_name("bookworm_test")
//         .start()
//         .await
//         .unwrap();
//     let host_ip = container.get_host().await;
//     let host_port = container.get_host_port_ipv4(5432).await.unwrap();
//     // prepare connection string
//     let connection_string = &format!(
//         "postgres://postgres:postgres@127.0.0.1:{}/postgres",
//         host_port
//     );
//
//     let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
//         .await
//         .unwrap();
//     tokio::spawn(async move {
//         if let Err(e) = connection.await {
//             eprintln!("connection error: {}", e);
//         }
//     });
//
//     let pool: PgPool = PgPool::connect(&connection_string).await.unwrap();
//
//     // test code
//     let row: (i32,) = sqlx::query_as("SELECT 1 + 1")
//         .fetch_one(&pool)
//         .await
//         .unwrap();
//     assert_eq!(row.0, 2);
// }
//
// #[tokio::test]
// async fn test_with_pool() {
//     let pool: PgPool = setup_pool().await;
//
//     // test code
//     let row: (i32,) = sqlx::query_as("SELECT 1 + 1")
//         .fetch_one(&pool)
//         .await
//         .unwrap();
//     assert_eq!(row.0, 2);
// }
