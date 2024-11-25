mod common;
use serde_json::Value;
#[cfg(test)]
mod test {
    use super::*;
    use crate::common::*;
    #[test]
    fn test_create_definition() {
        let create_def_cmd = create_def_cmd_1();
        disintegrate::TestHarness::given([])
            .when(create_def_cmd)
            .then(get_expected_def_created());
    }

    #[test]
    fn test_validate_definition_empty_schema() {
        disintegrate::TestHarness::given([get_expected_def_created_empty_json()])
            .when(get_validate_def_cmd())
            .then([get_expected_validation_failed()]);
    }

    #[test]
    fn test_validate_definition_invalid_json() {
        disintegrate::TestHarness::given([get_def_created_invalid_json()])
            .when(get_validate_def_cmd())
            .then([get_expected_validation_failed_invalid_json()]);
    }

    #[test]
    fn test_validate_with_valid_schema() {
        disintegrate::TestHarness::given([get_def_created_valid_json()])
            .when(get_validate_def_cmd())
            .then([get_expected_validation_success()]);
    }

    #[test]
    fn test_validate_with_empty_title() {
        disintegrate::TestHarness::given([get_def_created_empty_title()])
            .when(get_validate_def_cmd())
            .then([get_expected_validation_failed_empty_title()]);
    }

    #[test]
    fn test_update_with_valid_schema() {
        disintegrate::TestHarness::given([get_def_created_valid_json()])
            .when(get_update_def_cmd())
            .then([get_expected_def_updated()]);
    }

    #[test]
    fn simple_schema_test() {
        // let schema = r###"
        //     {"type": "string"}
        //     "###.to_string();

        let schema = r###"
        {
            "title": "example_schema",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###
        .to_string();
        let schema = r###"
        {
            "title": "example_schema",
            "type": "object",
            "properties": {
                "example": {
                    "type": "string"
                }
            }
        }
        "###
        .to_string();
        // let schema = serde_json::from_str(&schema).map_err(|e| format!("Invalid JSON schema: {}", e));
        let schema_value: Value = serde_json::from_str(&schema)
            .map_err(|e| format!("Invalid JSON schema: {}", e))
            .unwrap();
        let title = schema_value["title"]
            .as_str()
            .ok_or("Title not found in schema")
            .unwrap()
            .to_string();
        assert_eq!(title, "example_schema");
        // json!({"type": "123"});
        // let validator = jsonschema::draft7::new(&schema).map_err(|e| format!("Schema compilation error: {:?}", e)).unwrap();

        // assert!(validator.is_valid(&json!("Hello")));
        // assert!(validator.validate(&json!("Hello")).is_ok());
    }
}
