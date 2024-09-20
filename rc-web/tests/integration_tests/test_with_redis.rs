// use testcontainers::runners::AsyncRunner;
// use testcontainers_modules::postgres;
//
// #[tokio::test]
// async fn test_with_postgres() {
//     let container = postgres::Postgres::default().start().await.unwrap();
//     let host_port = container.get_host_port_ipv4(5432).await.unwrap();
//     let connection_string = &format!(
//         "postgres://postgres:postgres@127.0.0.1:{host_port}/postgres",
//     );
//     // container is up, you can use it
// }
