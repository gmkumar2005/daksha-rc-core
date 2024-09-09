#[cfg(test)]
mod aggregate_tests {
    use super::*;

    use cqrs_es::test::TestFramework;
    use async_trait::async_trait;
    use std::sync::Mutex;
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


    #[test]
    fn test_create_def() {
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
        let command = ValidateDef;
        let expected = SchemaDefEvent::DefValidated;
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
        let validated = SchemaDefEvent::DefValidated;
        let command = ActivateDef;
        let expected = SchemaDefEvent::DefActivated;
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
        let validated = SchemaDefEvent::DefValidated;
        let activated = SchemaDefEvent::DefActivated;
        let expected = SchemaDefEvent::DefActivated;

        let services = SchemaDefServices::new(Box::new(MockSchemaDefServices::default()));
        let command = ActivateDef;
        SchemaDefTestFramework::with(services)
            .given(vec![activated])
            .when(command)
            .then_expect_error_message("Error 400: SchemaDoc must be valid before \
            activation; cannot move status from Active to Active");
    }
}
