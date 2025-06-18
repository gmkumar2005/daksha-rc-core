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
        // Create the helper function for array to string conversion
        sqlx::query(
            r#"
            CREATE OR REPLACE FUNCTION jsonb_array_to_string(arr jsonb, delimiter text DEFAULT ',')
            RETURNS text
            LANGUAGE sql
            IMMUTABLE
            AS $$
                SELECT string_agg(value #>> '{}', delimiter)
                FROM jsonb_array_elements(arr);
            $$;
            "#,
        )
        .execute(&pool)
        .await?;

        // Create the definitions table with generated columns
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS definitions (
                id UUID PRIMARY KEY,
                json_schema_string JSONB NOT NULL,
                -- Generated column: Extract title from json schema
                title TEXT GENERATED ALWAYS AS (
                    json_schema_string->>'title'
                ) STORED,
                -- Generated column: Extract indexFields from _osConfig and convert array to comma-separated string
                index_fields TEXT GENERATED ALWAYS AS (
                    CASE
                        WHEN jsonb_typeof(json_schema_string->'_osConfig'->'indexFields') = 'array'
                        THEN jsonb_array_to_string(json_schema_string->'_osConfig'->'indexFields', ',')
                        ELSE NULL
                    END
                ) STORED,
                -- Additional generated columns for demonstration
                private_fields TEXT GENERATED ALWAYS AS (
                    CASE
                        WHEN jsonb_typeof(json_schema_string->'_osConfig'->'privateFields') = 'array'
                        THEN jsonb_array_to_string(json_schema_string->'_osConfig'->'privateFields', ',')
                        ELSE NULL
                    END
                ) STORED,
                unique_index_fields TEXT GENERATED ALWAYS AS (
                    CASE
                        WHEN jsonb_typeof(json_schema_string->'_osConfig'->'uniqueIndexFields') = 'array'
                        THEN jsonb_array_to_string(json_schema_string->'_osConfig'->'uniqueIndexFields', ',')
                        ELSE NULL
                    END
                ) STORED,
                -- Refactored: Replace system_fields_count with system_fields (comma-separated values)
                system_fields TEXT GENERATED ALWAYS AS (
                    CASE
                        WHEN jsonb_typeof(json_schema_string->'_osConfig'->'systemFields') = 'array'
                        THEN jsonb_array_to_string(json_schema_string->'_osConfig'->'systemFields', ',')
                        ELSE NULL
                    END
                ) STORED,
                -- New generated column: attestationAttributes (comma-separated values)
                attestation_attributes TEXT GENERATED ALWAYS AS (
                    CASE
                        WHEN jsonb_typeof(json_schema_string->'_osConfig'->'attestationAttributes') = 'array'
                        THEN jsonb_array_to_string(json_schema_string->'_osConfig'->'attestationAttributes', ',')
                        ELSE NULL
                    END
                ) STORED,
                -- New generated column: inviteRoles (comma-separated values)
                invite_roles TEXT GENERATED ALWAYS AS (
                    CASE
                        WHEN jsonb_typeof(json_schema_string->'_osConfig'->'inviteRoles') = 'array'
                        THEN jsonb_array_to_string(json_schema_string->'_osConfig'->'inviteRoles', ',')
                        ELSE NULL
                    END
                ) STORED,
                -- New generated column: roles (comma-separated values)
                roles TEXT GENERATED ALWAYS AS (
                    CASE
                        WHEN jsonb_typeof(json_schema_string->'_osConfig'->'roles') = 'array'
                        THEN jsonb_array_to_string(json_schema_string->'_osConfig'->'roles', ',')
                        ELSE NULL
                    END
                ) STORED,
                has_attestation_policies BOOLEAN GENERATED ALWAYS AS (
                    jsonb_typeof(json_schema_string->'_osConfig'->'attestationPolicies') = 'array'
                    AND jsonb_array_length(json_schema_string->'_osConfig'->'attestationPolicies') > 0
                ) STORED,
                record_status TEXT,
                created_at TIMESTAMPTZ,
                created_by TEXT,
                activated_by TEXT,
                activated_at TIMESTAMPTZ,
                updated_at TIMESTAMPTZ
            );
            "#,
        )
        .execute(&pool)
        .await?;

        // Create indexes for better performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_definitions_json_schema_gin ON definitions USING GIN (json_schema_string);")
            .execute(&pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_definitions_index_fields ON definitions (index_fields);")
            .execute(&pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_definitions_title ON definitions (title);")
            .execute(&pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_definitions_private_fields ON definitions (private_fields);")
            .execute(&pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_definitions_system_fields ON definitions (system_fields);")
            .execute(&pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_definitions_attestation_attributes ON definitions (attestation_attributes);")
            .execute(&pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_definitions_invite_roles ON definitions (invite_roles);")
            .execute(&pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_definitions_roles ON definitions (roles);")
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
                created_at,
                created_by,
                json_schema_string,
                ..
            } => {
                debug!(
                    "DomainEvent::DefCreated id {:#?} title is {}",
                    id,
                    title.clone()
                );
                let result=    sqlx::query(
                    "INSERT INTO definitions(id, json_schema_string, record_status, created_at, created_by, updated_at) VALUES($1, $2::json, $3,$4,$5,$6) ON CONFLICT DO NOTHING",
                )
                    .bind(id)
                    .bind(json_schema_string)
                    .bind(DefRecordStatus::Draft.to_string())
                    .bind(created_at)
                    .bind(created_by)
                    .bind(created_at)  // updated_at same as created_at for new records
                    .execute(&self.pool)
                    .await;
                if let Err(e) = &result {
                    debug!("Failed to insert definitions: {:?}", e);
                }
                result?;
            }
            DomainEvent::DefActivated {
                id,
                activated_by,
                activated_at,
                json_schema_string,
                ..
            } => {
                //TODO create projection table to hold entity data created using this schema
                debug!(
                    "DomainEvent::DefActivated id {:#?} activated_by is {}",
                    id,
                    activated_by.clone()
                );
                let result = sqlx::query(
                    "UPDATE definitions
                            SET
                                json_schema_string = $2::json,
                                record_status = $3,
                                activated_by = $4,
                                activated_at = $5
                            WHERE
                                id = $1;
                        ",
                )
                .bind(id)
                .bind(json_schema_string)
                .bind(DefRecordStatus::Active.to_string())
                .bind(activated_by.clone())
                .bind(activated_at)
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
