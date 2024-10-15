// use async_trait::async_trait;
// use sqlx::PgPool;
// use sqlx::SqlitePool;
// use crate::read_side_processor::{ProjectionOffsetStore, ProjectionOffsetStoreRepository};
//
// pub struct PostgresProjectionOffsetStoreRepository {
//     pool: PgPool,
// }
//
// impl PostgresProjectionOffsetStoreRepository {
//     pub fn new(pool: PgPool) -> Self {
//         Self { pool }
//     }
// }
//
// #[async_trait]
// impl ProjectionOffsetStoreRepository for PostgresProjectionOffsetStoreRepository {
//     async fn insert_record(&self, record: ProjectionOffsetStore) -> anyhow::Result<(), anyhow::Error> {
//         sqlx::query!(
//             r#"
//             INSERT INTO pekko_projection_offset_store (projection_name, projection_key, current_offset, manifest, mergeable, last_updated)
//             VALUES ($1, $2, $3, $4, $5, $6)
//             "#,
//             record.projection_name,
//             record.projection_key,
//             record.current_offset,
//             record.manifest,
//             record.mergeable,
//             record.last_updated
//         )
//             .execute(&self.pool)
//             .await?;
//         Ok(())
//     }
//
//     async fn read_record(&self, projection_name: &str, projection_key: &str) -> anyhow::Result<Option<ProjectionOffsetStore>, anyhow::Error> {
//         let record = sqlx::query_as!(
//             ProjectionOffsetStore,
//             r#"
//             SELECT projection_name, projection_key, current_offset, manifest, mergeable, last_updated
//             FROM pekko_projection_offset_store
//             WHERE projection_name = $1 AND projection_key = $2
//             "#,
//             projection_name,
//             projection_key
//         )
//             .fetch_optional(&self.pool)
//             .await?;
//         Ok(record)
//     }
// }

//
//
// pub struct SqliteProjectionOffsetStoreRepository {
//     pool: SqlitePool,
// }
//
// impl SqliteProjectionOffsetStoreRepository {
//     pub fn new(pool: SqlitePool) -> Self {
//         Self { pool }
//     }
// }
//
// #[async_trait]
// impl ProjectionOffsetStoreRepository for SqliteProjectionOffsetStoreRepository {
//     async fn insert_record(&self, record: ProjectionOffsetStore) -> anyhow::Result<(), anyhow::Error> {
//         sqlx::query!(
//             r#"
//             INSERT INTO pekko_projection_offset_store (projection_name, projection_key, current_offset, manifest, mergeable, last_updated)
//             VALUES ($1, $2, $3, $4, $5, $6)
//             "#,
//             record.projection_name,
//             record.projection_key,
//             record.current_offset,
//             record.manifest,
//             record.mergeable,
//             record.last_updated
//         )
//             .execute(&self.pool)
//             .await?;
//         Ok(())
//     }
//
//     async fn read_record(&self, projection_name: &str, projection_key: &str) -> Result<Option<ProjectionOffsetStore>,anyhow::Error> {
//         let record = sqlx::query_as!(
//             ProjectionOffsetStore,
//             r#"
//             SELECT projection_name, projection_key, current_offset, manifest, mergeable, last_updated
//             FROM pekko_projection_offset_store
//             WHERE projection_name = $1 AND projection_key = $2
//             "#,
//             projection_name,
//             projection_key
//         )
//             .fetch_optional(&self.pool)
//             .await?;
//         Ok(record)
//     }
// }