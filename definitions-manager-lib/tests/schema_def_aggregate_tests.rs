#[cfg(test)]
mod aggregate_tests {
    use std::collections::HashMap;
    use super::*;

    use cqrs_es::test::TestFramework;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use cqrs_es::mem_store::MemStore;
    use cqrs_es::{CqrsFramework, EventEnvelope, Query};
    use definitions_manager_lib::schema_def::SchemaDef;
    use definitions_manager_lib::schema_def_commands::SchemaDefCommand::*;
    use definitions_manager_lib::schema_def_events::SchemaDefEvent;
    use definitions_manager_lib::schema_def_services::*;

    type SchemaDefTestFramework = TestFramework<SchemaDef>;


    pub struct MockSchemaDefServices {
        pub get_user_id: Mutex<Result<(), SchemaValidationError>>,
    }

    impl Default for MockSchemaDefServices {
        fn default() -> Self {
            Self {
                get_user_id: Mutex::new(Ok(())),
            }
        }
    }
    impl MockSchemaDefServices {
        pub fn new() -> Self {
            Self::default()
        }
        fn create_def(&self, _id: &str, _schema: &str) -> Result<(), SchemaValidationError> {
            self.get_user_id.lock().unwrap().clone()
        }
    }
    #[async_trait]
    impl SchemaDefServicesApi for MockSchemaDefServices {
        async fn get_user_id(&self, user_id: &str) -> Result<(), SchemaValidationError> {
            self.get_user_id.lock().unwrap().clone()
        }
    }

    fn remove_spaces_and_returns(input: &str) -> String {
        input.replace(" ", "").replace("\n", "").replace("\r", "")
    }

    struct SimpleLoggingQuery {}
    #[async_trait]
    impl Query<SchemaDef> for SimpleLoggingQuery {
        async fn dispatch(&self, aggregate_id: &str, events: &[EventEnvelope<SchemaDef>]) {
            for event in events {
                println!("aggregate_id-event.seq: {}-{} -- {:#?} -- metadata {:#?}", aggregate_id, event.sequence, &event.payload, &event.metadata);
            }
        }
    }

    #[test]
    fn test_create_def() {
        let valid_schema_with_title = r###"
        {
            "title": "Example Schema",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###.to_string();

        let expected = SchemaDefEvent::DefCreated { id: "123".to_string(), schema: valid_schema_with_title.clone() };
        let services = SchemaDefServices::new(Box::new(MockSchemaDefServices::default()));
        let command = CreateDef { id: "123".to_string(), schema: valid_schema_with_title.clone() };
        SchemaDefTestFramework::with(services)
            .given_no_previous_events()
            .when(command)
            .then_expect_events(vec![expected]);
    }
    #[test]
    fn test_validation_def() {
        let valid_schema_with_title = r###"
        {
            "title": "Example Schema",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###.to_string();

        let services = SchemaDefServices::new(Box::new(MockSchemaDefServices::default()));

        let created = SchemaDefEvent::DefCreated { id: "123".to_string(), schema: valid_schema_with_title.clone() };
        let command = ValidateDef;
        let expected = SchemaDefEvent::DefValidated{id: "123".to_string() } ;
        SchemaDefTestFramework::with(services)
            .given(vec![created])
            .when(command)
            .then_expect_events(vec![expected]);
    }

    #[test]
    fn test_activate_def() {
        let valid_schema_with_title = r#"
        {
            "title": "Example Schema",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "#
            .to_string();

        let services = SchemaDefServices::new(Box::new(MockSchemaDefServices::default()));

        let created = SchemaDefEvent::DefCreated { id: "123".to_string(), schema: valid_schema_with_title.clone() };
        let validated = SchemaDefEvent::DefValidated{id: "123".to_string() };
        let command = ActivateDef;
        let expected = SchemaDefEvent::DefActivated{id: "123".to_string() };
        SchemaDefTestFramework::with(services)
            .given(vec![created, validated])
            .when(command)
            .then_expect_events(vec![expected]);
    }

    #[test]
    fn test_activate_def_twice_should_fail() {
        let valid_schema_with_title = r#"
        {
            "title": "Example Schema",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "#
            .to_string();


        let created = SchemaDefEvent::DefCreated { id: "123".to_string(), schema: valid_schema_with_title.clone() };
        let validated = SchemaDefEvent::DefValidated{id: "123".to_string() };
        let activated = SchemaDefEvent::DefActivated{id: "123".to_string() };
        let expected = SchemaDefEvent::DefActivated{id: "123".to_string() };

        let services = SchemaDefServices::new(Box::new(MockSchemaDefServices::default()));
        let command = ActivateDef;
        SchemaDefTestFramework::with(services)
            .given(vec![activated])
            .when(command)
            .then_expect_error_message("Error 400: SchemaDoc must be valid before \
            activation; cannot move status from Active to Active");
    }

    #[tokio::test]
    async fn test_with_in_memory_event_store(){
        let event_store = MemStore::<SchemaDef>::default();
        let mut metadata = HashMap::new();
        metadata.insert("time".to_string(), chrono::Utc::now().to_rfc3339());
        let query = SimpleLoggingQuery {};
        let services = SchemaDefServices::new(Box::new(MockSchemaDefServices::default()));
        let cqrs = CqrsFramework::new(event_store, vec![Box::new(query)], services);
        let aggregate_id = "aggregate-instance-A";
        let valid_schema_with_title = r###"
        {
            "title": "Example_Schema",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###.to_string();
        let valid_schema_single_line = remove_spaces_and_returns(&valid_schema_with_title);

        let create_def_command = CreateDef { id: "123".to_string(), schema: valid_schema_single_line.clone() };
        let aggregate_id = "Example_Schema";
        let validate_def_command = ValidateDef;
        let activate_def_command = ActivateDef;
        let activate_def_command2 = ActivateDef;
        let _ = cqrs.execute_with_metadata(aggregate_id, create_def_command,metadata.clone()).await;
        let _ = cqrs.execute_with_metadata(aggregate_id, validate_def_command,metadata.clone()).await;
        let _ = cqrs.execute_with_metadata(aggregate_id, activate_def_command,metadata.clone()).await;
        let _ = cqrs.execute_with_metadata(aggregate_id, activate_def_command2,metadata.clone()).await;
    }
}
