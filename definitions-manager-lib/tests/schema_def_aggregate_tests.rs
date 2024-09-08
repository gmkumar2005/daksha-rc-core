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
        pub create_def: Mutex<Result<(), SchemaValidationError>>,
        pub validate_def: Mutex<Result<(), SchemaValidationError>>,
        pub activate_def: Mutex<Result<(), SchemaValidationError>>,
        pub deactivate_def: Mutex<Result<(), SchemaValidationError>>,
        pub create_and_validate_def: Mutex<Result<(), SchemaValidationError>>,
    }

    impl Default for MockSchemaDefServices {
        fn default() -> Self {
            Self {
                create_def: Mutex::new(Ok(())),
                validate_def: Mutex::new(Ok(())),
                activate_def: Mutex::new(Ok(())),
                deactivate_def: Mutex::new(Ok(())),
                create_and_validate_def: Mutex::new(Ok(())),

            }
        }
    }
    impl MockSchemaDefServices {
        pub fn new() -> Self {
            Self::default()
        }
        fn create_def(&self, _id: &str, _schema: &str) -> Result<(), SchemaValidationError> {
            self.create_def.lock().unwrap().clone()
        }
        fn validate_def(&self, _id: &str) -> Result<(), SchemaValidationError> {
            self.validate_def.lock().unwrap().clone()
        }
        fn activate_def(&self, _id: &str) -> Result<(), SchemaValidationError> {
            self.activate_def.lock().unwrap().clone()
        }
        fn deactivate_def(&self, _id: &str) -> Result<(), SchemaValidationError> {
            self.deactivate_def.lock().unwrap().clone()
        }
        fn create_and_validate_def(&self, _id: &str, _schema: &str) -> Result<(), SchemaValidationError> {
            self.create_and_validate_def.lock().unwrap().clone()
        }
    }
    #[async_trait]
    impl SchemaDefServicesApi for MockSchemaDefServices {
        async fn create_def(&self, _id: &str, _schema: &str) -> Result<(), SchemaValidationError> {
            self.create_def.lock().unwrap().clone()
        }


        async fn validate_def(&self, _id: &str) -> Result<(), SchemaValidationError> {
            self.validate_def.lock().unwrap().clone()
        }

        async fn activate_def(&self, _id: &str) -> Result<(), SchemaValidationError> {
            self.activate_def.lock().unwrap().clone()
        }

        async fn deactivate_def(&self, _id: &str) -> Result<(), SchemaValidationError> {
            self.deactivate_def.lock().unwrap().clone()
        }
        async fn create_and_validate_def(&self, _id: &str, _schema: &str) -> Result<(), SchemaValidationError> {
            self.create_and_validate_def.lock().unwrap().clone()
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
