use async_trait::async_trait;
use definitions_core::definitions_domain::{DefRecordStatus, DomainEvent};
use disintegrate::{query, EventListener, PersistedEvent, StreamQuery};
use disintegrate_postgres::PgEventId;
use log::debug;
use serde_json;

use sqlx::PgPool;
use tokio::signal;

use crate::projections::schema_projection::{
    generate_create_table_statement, generate_index_statements,
};

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
                .bind(json_schema_string.clone())
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

                // Create projection table and indices for the activated schema
                debug!("Starting projection table and indices creation for activated schema");
                if let Err(e) = self
                    .create_projection_table_and_indices(&json_schema_string)
                    .await
                {
                    debug!("Failed to create projection table and indices: {:?}", e);
                    return Err(e);
                }
                debug!("Successfully completed projection table and indices creation");
            }
            _ => {}
        }

        Ok(())
    }
}

impl ReadModelProjection {
    /// Creates projection table and indices for a given JSON schema
    ///
    /// This method generates a PostgreSQL projection table with:
    /// - An `entity_data JSONB NOT NULL` column to store the original JSON data
    /// - Generated columns for flattened schema attributes extracted from JSON paths
    /// - Indices based on the schema's `_osConfig` configuration
    ///
    /// ## Design Notes
    ///
    /// All primitive types (integer, number, boolean, date, datetime) are stored as TEXT
    /// in generated columns to ensure PostgreSQL immutability requirements. Generated
    /// columns must use immutable expressions, and type casting operations like `::INTEGER`
    /// or `::TIMESTAMPTZ` are not considered immutable by PostgreSQL.
    ///
    /// Applications can handle type conversion at query time or in application logic:
    /// ```sql
    /// -- Query with type conversion
    /// SELECT student_identitydetails_dob::DATE as dob_date
    /// FROM student_projection;
    /// ```
    async fn create_projection_table_and_indices(
        &self,
        json_schema_string: &str,
    ) -> Result<(), sqlx::Error> {
        debug!("Creating projection table and indices for JSON schema");

        // Parse the JSON schema string
        let schema: serde_json::Value = match serde_json::from_str(json_schema_string) {
            Ok(schema) => {
                debug!("Successfully parsed JSON schema");
                schema
            }
            Err(e) => {
                debug!("Failed to parse JSON schema: {:?}", e);
                return Err(sqlx::Error::Protocol(format!("Invalid JSON schema: {}", e)));
            }
        };

        // Extract and log the schema title
        let schema_title = schema
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("unknown");
        debug!("Processing schema with title: '{}'", schema_title);

        // Generate CREATE TABLE statement
        let create_table_sql = match generate_create_table_statement(&schema) {
            Ok(sql) => {
                debug!(
                    "Successfully generated CREATE TABLE statement for '{}'",
                    schema_title
                );
                sql
            }
            Err(e) => {
                debug!(
                    "Failed to generate CREATE TABLE statement for '{}': {}",
                    schema_title, e
                );
                return Err(sqlx::Error::Protocol(format!(
                    "Failed to generate CREATE TABLE: {}",
                    e
                )));
            }
        };

        // Execute CREATE TABLE statement
        debug!(
            "Executing CREATE TABLE statement for '{}_projection'",
            schema_title.to_lowercase()
        );
        debug!("CREATE TABLE SQL:\n{}", create_table_sql);
        if let Err(e) = sqlx::query(&create_table_sql).execute(&self.pool).await {
            debug!(
                "Failed to execute CREATE TABLE statement for '{}': {:?}",
                schema_title, e
            );
            return Err(e);
        }
        debug!(
            "Successfully created projection table '{}_projection'",
            schema_title.to_lowercase()
        );

        // Generate CREATE INDEX statements
        let index_statements = match generate_index_statements(&schema) {
            Ok(statements) => {
                debug!(
                    "Successfully generated {} CREATE INDEX statements for '{}'",
                    statements.len(),
                    schema_title
                );
                statements
            }
            Err(e) => {
                debug!(
                    "Failed to generate CREATE INDEX statements for '{}': {}",
                    schema_title, e
                );
                return Err(sqlx::Error::Protocol(format!(
                    "Failed to generate CREATE INDEX: {}",
                    e
                )));
            }
        };

        // Execute each CREATE INDEX statement
        let mut successful_indices = 0;
        let mut failed_indices = 0;

        debug!(
            "CREATE INDEX SQL statements:\n{}",
            index_statements.join("\n")
        );

        for (index_num, index_sql) in index_statements.iter().enumerate() {
            debug!(
                "Executing CREATE INDEX statement {} of {} for '{}'",
                index_num + 1,
                index_statements.len(),
                schema_title
            );
            if let Err(e) = sqlx::query(index_sql).execute(&self.pool).await {
                debug!(
                    "Failed to execute CREATE INDEX statement for '{}': {:?}",
                    schema_title, e
                );
                debug!("Failed SQL: {}", index_sql);
                failed_indices += 1;
                // Continue with other indices even if one fails
                debug!("Continuing with remaining index creation despite error");
            } else {
                debug!(
                    "Successfully executed CREATE INDEX statement {} for '{}'",
                    index_num + 1,
                    schema_title
                );
                successful_indices += 1;
            }
        }

        debug!(
            "Index creation summary for '{}': {} successful, {} failed out of {} total",
            schema_title,
            successful_indices,
            failed_indices,
            index_statements.len()
        );
        debug!(
            "Successfully completed projection table and indices creation for schema '{}'",
            schema_title
        );
        Ok(())
    }
}

pub async fn shutdown() {
    signal::ctrl_c().await.expect("failed to listen for event");
}
