use async_trait::async_trait;
use definitions_core::definitions_domain::{DefRecordStatus, DomainEvent};
use disintegrate::{query, EventListener, PersistedEvent, StreamQuery};
use disintegrate_postgres::PgEventId;
use log::debug;
use sqlx::PgPool;
use tokio::signal;

pub struct ReadModelProjection {
    query: StreamQuery<PgEventId, DomainEvent>,
    pool: PgPool,
}

impl ReadModelProjection {
    pub async fn new(pool: PgPool) -> Result<Self, sqlx::Error> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS definition (
            id UUID PRIMARY KEY,                -- Matches DefId mapped as TEXT/UUID
            title TEXT ,
            json_schema_string JSONB,
            record_status TEXT,
            created_at TIMESTAMPTZ ,    -- DateTime<Utc> in Rust becomes TIMESTAMPTZ
            created_by TEXT,
            activated_by TEXT
        )
            "#,
        )
        .execute(&pool)
        .await?;
        Ok(Self {
            query: query!(DomainEvent),
            pool,
        })
    }
}

#[async_trait]
impl EventListener<i64, DomainEvent> for ReadModelProjection {
    type Error = sqlx::Error;
    fn id(&self) -> &'static str {
        "definition"
    }

    fn query(&self) -> &StreamQuery<PgEventId, DomainEvent> {
        &self.query
    }

    async fn handle(&self, event: PersistedEvent<i64, DomainEvent>) -> Result<(), Self::Error> {
        match event.into_inner() {
            DomainEvent::DefCreated {
                id,
                title,
                definitions,
                created_at,
                created_by,
                json_schema_string,
            } => {
                debug!(
                    "DomainEvent::DefCreated id {:#?} title is {}",
                    id,
                    title.clone()
                );
                let result=    sqlx::query(
                    "INSERT INTO definition(id, title, json_schema_string, record_status, created_at, created_by) VALUES($1, $2, $3::json, $4,$5,$6) ON CONFLICT DO NOTHING",
                )
                    .bind(id)
                    .bind(title)
                    .bind(json_schema_string)
                    .bind(DefRecordStatus::Draft.to_string())
                    .bind(created_at)
                    .bind(created_by)
                    .execute(&self.pool)
                    .await;
                if let Err(e) = &result {
                    // Print the error in debug format
                    debug!("Failed to insert definition: {:?}", e);
                }
                result?;
            }
            DomainEvent::DefActivated {
                id,
                activated_at,
                activated_by,
                json_schema_string,
            } => {
                debug!(
                    "DomainEvent::DefActivated id {:#?} activated_by is {}",
                    id,
                    activated_by.clone()
                );
                let result = sqlx::query(
                    "UPDATE definition
                            SET
                                json_schema_string = $2::json,
                                record_status = $3,
                                activated_by = $4
                            WHERE
                                id = $1;
                        ",
                )
                .bind(id)
                .bind(json_schema_string)
                .bind(DefRecordStatus::Active.to_string())
                .bind(activated_by.clone())
                .execute(&self.pool)
                .await;
                if let Err(e) = &result {
                    // Print the error in debug format
                    debug!("Failed to update definition: {:?}", e);
                }
                result?;
            }
            _ => {}
        }
        Ok(())
    }
}

pub async fn shutdown() {
    signal::ctrl_c().await.expect("failed to listen for event");
}
