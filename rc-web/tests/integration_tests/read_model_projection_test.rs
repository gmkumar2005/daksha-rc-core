use super::{begin_transaction, get_shared_pool};
use chrono::Utc;
use definitions_core::definitions_domain::{
    generate_id_from_title, ActivateDefinitionCmd, CreateDefinitionCmd, DomainEvent,
};
use definitions_core::registry_domain::CreateEntityCmd;
use disintegrate::{EventListener, NoSnapshot};
use disintegrate_postgres::PgEventStore;
use rc_web::projections::definitions_read_model::ReadModelProjection;

use sqlx::{query, Row};
use std::ops::Deref;
use uuid::Uuid;

const STUDENT_SCHEMA_JSON: &str = r#"{
    "title": "Student",
    "type": "object",
    "properties": {
        "student": {
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" },
                "email": { "type": "string", "format": "email" }
            }
        }
    },
    "_osConfig": {
        "indexFields": ["name", "email"]
    }
}"#;

const STUDENT_ENTITY_JSON: &str = r#"{
    "student": {
        "name": "John Doe",
        "age": 20,
        "email": "john.doe@example.com"
    }
}"#;

fn create_test_def_cmd() -> CreateDefinitionCmd {
    CreateDefinitionCmd {
        id: generate_id_from_title("Student"),
        title: "Student".to_string(),
        definitions: vec!["Student".to_string()],
        json_schema_string: STUDENT_SCHEMA_JSON.to_string(),
        created_by: "test_user".to_string(),
    }
}

fn create_test_activate_cmd() -> ActivateDefinitionCmd {
    ActivateDefinitionCmd {
        id: generate_id_from_title("Student"),
        activated_at: Utc::now(),
        activated_by: "test_user".to_string(),
    }
}

fn create_test_entity_cmd() -> CreateEntityCmd {
    CreateEntityCmd {
        id: Uuid::now_v7(),
        entity_body: STUDENT_ENTITY_JSON.to_string(),
        entity_type: "Student".to_string(),
        created_by: "test_user".to_string(),
    }
}

#[tokio::test]
async fn test_read_model_projection_entity_created_with_new_columns() -> anyhow::Result<()> {
    let pool = get_shared_pool().await;
    let mut tx = begin_transaction(&pool).await?;

    // Setup event store and decision maker
    let serde = disintegrate::serde::json::Json::<DomainEvent>::default();
    let event_store = PgEventStore::new(pool.clone(), serde).await?;
    let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);

    // Create ReadModelProjection
    let read_model_projection = ReadModelProjection::new(pool.clone()).await?;

    // Step 1: Create definition
    let create_def_cmd = create_test_def_cmd();
    let def_events = decision_maker.make(create_def_cmd).await?;

    // Handle DefCreated event
    for event in def_events {
        read_model_projection.handle(event).await?;
    }

    // Step 2: Activate definition (this should create the projection table)
    let activate_def_cmd = create_test_activate_cmd();
    let activate_events = decision_maker.make(activate_def_cmd).await?;

    // Handle DefActivated event
    for event in activate_events {
        read_model_projection.handle(event).await?;
    }

    // Verify that the projection table was created with the correct structure
    let table_info = query(
        "SELECT column_name, data_type, is_nullable
         FROM information_schema.columns
         WHERE table_name = 'student_projection'
         ORDER BY ordinal_position",
    )
    .fetch_all(&mut *tx)
    .await?;

    // Check that required columns exist
    let column_names: Vec<String> = table_info
        .iter()
        .map(|row| row.get::<String, _>("column_name"))
        .collect();

    assert!(column_names.contains(&"id".to_string()));
    assert!(column_names.contains(&"entity_type".to_string()));
    assert!(column_names.contains(&"created_by".to_string()));
    assert!(column_names.contains(&"created_at".to_string()));
    assert!(column_names.contains(&"registry_def_id".to_string()));
    assert!(column_names.contains(&"registry_def_version".to_string()));
    assert!(column_names.contains(&"version".to_string()));
    assert!(column_names.contains(&"entity_data".to_string()));

    // Step 3: Create entity
    let create_entity_cmd = create_test_entity_cmd();
    let entity_id = create_entity_cmd.id;
    let entity_events = decision_maker.make(create_entity_cmd).await?;

    // Extract data from EntityCreated event for verification
    let mut entity_created_data = None;
    for event in &entity_events {
        if let DomainEvent::EntityCreated {
            id,
            registry_def_id,
            registry_def_version,
            entity_body,
            entity_type,
            created_at,
            created_by,
            version,
        } = event.deref()
        {
            entity_created_data = Some((
                *id,
                *registry_def_id,
                *registry_def_version,
                entity_body.clone(),
                entity_type.clone(),
                *created_at,
                created_by.clone(),
                *version,
            ));
            break;
        }
    }

    // Handle EntityCreated event
    for event in entity_events {
        read_model_projection.handle(event).await?;
    }

    let (
        expected_id,
        expected_registry_def_id,
        expected_registry_def_version,
        expected_entity_body,
        expected_entity_type,
        expected_created_at,
        expected_created_by,
        expected_version,
    ) = entity_created_data.expect("EntityCreated event should exist");

    // Step 4: Verify that the entity was inserted with all required columns
    let inserted_row = query(
        "SELECT id, entity_type, created_by, created_at, registry_def_id, registry_def_version, version, entity_data
         FROM student_projection
         WHERE id = $1"
    )
    .bind(entity_id)
    .fetch_one(&mut *tx)
    .await?;

    // Verify all columns have correct values
    assert_eq!(inserted_row.get::<Uuid, _>("id"), expected_id);
    assert_eq!(
        inserted_row.get::<String, _>("entity_type"),
        expected_entity_type
    );
    assert_eq!(
        inserted_row.get::<String, _>("created_by"),
        expected_created_by
    );
    assert_eq!(
        inserted_row.get::<chrono::DateTime<Utc>, _>("created_at"),
        expected_created_at
    );
    assert_eq!(
        inserted_row.get::<Uuid, _>("registry_def_id"),
        expected_registry_def_id
    );
    assert_eq!(
        inserted_row.get::<i32, _>("registry_def_version"),
        expected_registry_def_version.get() as i32
    );
    assert_eq!(
        inserted_row.get::<i32, _>("version"),
        expected_version.get() as i32
    );

    // Verify entity_data contains the expected JSON
    let entity_data: serde_json::Value = inserted_row.get("entity_data");
    let expected_json: serde_json::Value = serde_json::from_str(&expected_entity_body)?;
    assert_eq!(entity_data, expected_json);

    // Step 5: Verify that generated columns work (if any exist)
    let generated_columns = query(
        "SELECT student_name, student_age, student_email
         FROM student_projection
         WHERE id = $1",
    )
    .bind(entity_id)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(gen_row) = generated_columns {
        // If generated columns exist, verify they extracted values correctly
        if let Ok(name) = gen_row.try_get::<Option<String>, _>("student_name") {
            assert_eq!(name, Some("John Doe".to_string()));
        }
        if let Ok(age) = gen_row.try_get::<Option<String>, _>("student_age") {
            assert_eq!(age, Some("20".to_string()));
        }
        if let Ok(email) = gen_row.try_get::<Option<String>, _>("student_email") {
            assert_eq!(email, Some("john.doe@example.com".to_string()));
        }
    }

    tx.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn test_projection_table_structure() -> anyhow::Result<()> {
    let pool = get_shared_pool().await;
    let mut tx = begin_transaction(&pool).await?;

    // Setup event store and decision maker
    let serde = disintegrate::serde::json::Json::<DomainEvent>::default();
    let event_store = PgEventStore::new(pool.clone(), serde).await?;
    let decision_maker = disintegrate_postgres::decision_maker(event_store, NoSnapshot);

    // Create ReadModelProjection
    let read_model_projection = ReadModelProjection::new(pool.clone()).await?;

    // Create and activate definition to create projection table
    let create_def_cmd = create_test_def_cmd();
    let def_events = decision_maker.make(create_def_cmd).await?;

    for event in def_events {
        read_model_projection.handle(event).await?;
    }

    let activate_def_cmd = create_test_activate_cmd();
    let activate_events = decision_maker.make(activate_def_cmd).await?;

    for event in activate_events {
        read_model_projection.handle(event).await?;
    }

    // Check table structure in detail
    let constraints = query(
        "SELECT constraint_name, constraint_type
         FROM information_schema.table_constraints
         WHERE table_name = 'student_projection'",
    )
    .fetch_all(&mut *tx)
    .await?;

    // Verify primary key exists
    let has_primary_key = constraints
        .iter()
        .any(|row| row.get::<String, _>("constraint_type") == "PRIMARY KEY");
    assert!(
        has_primary_key,
        "student_projection table should have a primary key"
    );

    // Check column types
    let column_info = query(
        "SELECT column_name, data_type, is_nullable, column_default
         FROM information_schema.columns
         WHERE table_name = 'student_projection'
         ORDER BY ordinal_position",
    )
    .fetch_all(&mut *tx)
    .await?;

    for column in &column_info {
        let column_name: String = column.get("column_name");
        let data_type: String = column.get("data_type");
        let is_nullable: String = column.get("is_nullable");

        match column_name.as_str() {
            "id" => {
                assert_eq!(data_type, "uuid");
                assert_eq!(is_nullable, "NO");
            }
            "entity_type" => {
                assert_eq!(data_type, "text");
                assert_eq!(is_nullable, "NO");
            }
            "created_by" => {
                assert_eq!(data_type, "text");
                assert_eq!(is_nullable, "NO");
            }
            "created_at" => {
                assert_eq!(data_type, "timestamp with time zone");
                assert_eq!(is_nullable, "NO");
            }
            "registry_def_id" => {
                assert_eq!(data_type, "uuid");
                assert_eq!(is_nullable, "NO");
            }
            "registry_def_version" => {
                assert_eq!(data_type, "integer");
                assert_eq!(is_nullable, "NO");
            }
            "version" => {
                assert_eq!(data_type, "integer");
                assert_eq!(is_nullable, "NO");
            }
            "entity_data" => {
                assert_eq!(data_type, "jsonb");
                assert_eq!(is_nullable, "NO");
            }
            _ => {} // Generated columns can vary
        }
    }

    tx.rollback().await?;
    Ok(())
}
