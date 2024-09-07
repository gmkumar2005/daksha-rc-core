use definitions_manager_lib::schema_def::*;
use hamcrest2::prelude::*;
use std::fs::File;
use std::io::Read;
#[cfg(test)]
mod tests {
    use super::*;

    fn load_schema_from_file(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }
    #[tokio::test]
    async fn test_schema_def_initialization() {
        let schema = r#"
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
        let schema_doc = SchemaDef::new("1".to_string(), schema.clone()).unwrap();
        assert_eq!(schema_doc.id, "1");
        assert_eq!(schema_doc.title, "Example Schema");
        assert_eq!(schema_doc.schema, schema);
        assert_eq!(schema_doc.status, Status::Inactive);
    }

    #[tokio::test]
    async fn test_schema_def_validation() {
        let schema = r#"
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

        let schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        let schema_doc = schema_doc.validate_def().expect("TODO: panic message");
        assert_eq!(schema_doc.status, Status::Valid);
    }

    #[tokio::test]
    async fn test_schema_def_activation() {
        let schema = r#"
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

        let schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        let schema_doc = schema_doc.validate_def().expect("TODO: panic message");
        assert_eq!(schema_doc.status, Status::Valid);
        let schema_doc = schema_doc.activate().expect("TODO: panic message");
        assert_eq!(schema_doc.status, Status::Active);
    }

    #[tokio::test]
    async fn test_schema_def_activation_without_validation() {
        let schema = r#"
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

        let schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        let result = schema_doc.clone().activate();
        assert!(result.is_err());
        assert_eq!(
            "SchemaDoc must be valid before activation".to_string(),
            result.err().unwrap()
        );
        assert_eq!(schema_doc.status, Status::Inactive);
    }

    #[tokio::test]
    async fn test_schema_def_validation_institute() {
        let schema = load_schema_from_file("tests/resources/schemas/institute.json").unwrap();

        let schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        let schema_doc = schema_doc.validate_def().expect("TODO: panic message");
        assert_eq!(schema_doc.status, Status::Valid);
    }
    #[tokio::test]
    async fn test_schema_def_validation_student() {
        let schema = load_schema_from_file("tests/resources/schemas/student.json").unwrap();
        let schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        let schema_doc = schema_doc.validate_def().expect("TODO: panic message");
        assert_eq!(schema_doc.status, Status::Valid);
    }
    #[tokio::test]
    async fn test_schema_def_validation_teacher() {
        let schema = load_schema_from_file("tests/resources/schemas/teacher.json").unwrap();
        let schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        let schema_doc = schema_doc.validate_def().expect("TODO: panic message");
        assert_eq!(schema_doc.status, Status::Valid);
    }

    #[tokio::test]
    async fn test_schema_def_validation_not_a_json() {
        let schema = load_schema_from_file("tests/resources/schemas/not_a_json.json").unwrap();
        let result = SchemaDef::new("1".to_string(), schema);
        assert_that!(result.clone(), err());
        let error_message = result.err().unwrap();
        assert_that!(error_message, matches_regex(r".*Invalid JSON schema.*"));
    }

    #[tokio::test]
    async fn test_schema_def_no_title_schema() {
        let schema = load_schema_from_file("tests/resources/schemas/not_title.json").unwrap();
        let result = SchemaDef::new("1".to_string(), schema);
        assert_that!(result.clone(), err());
        let error_message = result.err().unwrap();
        assert_that!(
            error_message,
            matches_regex(r".*Title not found in schema.*")
        );
    }

    #[tokio::test]
    async fn test_validate_record() {
        let schema = r#"
    {
        "title": "Example Schema",
        "type": "object",
        "properties": {
            "example": {
                "type": "string"
            }
        },
        "required": ["example"]
    }
    "#.to_string();

        let schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        let valid_record = r#"{ "example": "test" }"#;
        let invalid_record = r#"{ "example": 123 }"#;
        let missing_field_record = r#"{ }"#;

        // Test valid record
        let result = schema_doc.validate_record(valid_record);
        assert!(result.is_ok());

        // Test invalid record
        let result = schema_doc.validate_record(invalid_record);
        assert!(result.is_err());
        let error_message: Vec<String> = result.err().unwrap().collect();
        assert_that!(&*error_message, contains("123 is not of type \"string\"".to_string()));

        // Test record with missing required field
        let result = schema_doc.validate_record(missing_field_record);
        assert!(result.is_err());
        let error_message: Vec<String> = result.err().unwrap().collect();
        assert_that!(&*error_message, contains("\"example\" is a required property".to_string()));
    }
}
