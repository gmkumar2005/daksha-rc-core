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

        let mut schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        schema_doc.validate().expect("TODO: panic message");
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

        let mut schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        schema_doc.validate().expect("TODO: panic message");
        assert_eq!(schema_doc.status, Status::Valid);
        schema_doc.activate().expect("TODO: panic message");
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

        let mut schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        let result = schema_doc.activate();
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

        let mut schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        schema_doc.validate().expect("TODO: panic message");
        assert_eq!(schema_doc.status, Status::Valid);
    }
    #[tokio::test]
    async fn test_schema_def_validation_student() {
        let schema = load_schema_from_file("tests/resources/schemas/student.json").unwrap();
        let mut schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        schema_doc.validate().expect("TODO: panic message");
        assert_eq!(schema_doc.status, Status::Valid);
    }
    #[tokio::test]
    async fn test_schema_def_validation_teacher() {
        let schema = load_schema_from_file("tests/resources/schemas/teacher.json").unwrap();
        let mut schema_doc = SchemaDef::new("1".to_string(), schema).unwrap();
        assert_eq!(schema_doc.status, Status::Inactive);
        schema_doc.validate().expect("TODO: panic message");
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

    // #[tokio::test]
    // async fn test_schema_def_validation_not_a_schema(){
    //     let schema = load_schema_from_file("tests/resources/schemas/not_a_schema.json").unwrap();
    //     let mut result = SchemaDef::new("1".to_string(), schema);
    //     // assert_eq!(result.unwrap().status, Status::Inactive);
    //     let mut error_message = result.unwrap().validate();
    //     assert_that!(error_message, err());
    //     // assert_that!(result.unwrap_err(), matches_regex(r".*Title not found in schema.*"));
    // }
}
