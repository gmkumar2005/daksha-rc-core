use sqlx::migrate::MigrateDatabase;
use sqlx::Sqlite;

// const DB_URL: &str = "sqlite:///Users/mallru/workspace/dpghackhaton/daksha-rc-core/definitions-infra-sqlite/target/sqlite.db";

// #[tokio::test]
// async fn test_schema_def_initialization() {
//     let schema = r###"
//         {
//             "title": "example_schema",
//             "type": "object",
//             "properties": {
//                 "example": {
//                     "type": "string"
//                 }
//             }
//         }
//         "###
//         .to_string();
//
//     if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
//         println!("Creating database {}", DB_URL);
//         match Sqlite::create_database(DB_URL).await {
//             Ok(_) => println!("Create db success"),
//             Err(error) => panic!("error: {}", error),
//         }
//     } else {
//         println!("Database already exists");
//     }
// }