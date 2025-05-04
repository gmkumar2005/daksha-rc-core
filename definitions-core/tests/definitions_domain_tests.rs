// use mockall::*;

mod common;
use serde_json::Value;
#[cfg(test)]
mod test {
    use super::*;
    use crate::common::test_harness::SimpleTestHarness;
    use crate::common::*;
    use definitions_core::definitions_domain::DefError::{ModifyNotAllowed, TitleIsNotMutable};
    use definitions_core::definitions_domain::{
        generate_id_from_title, DefRecordStatus, DomainEvent,
    };
    #[test]
    fn test_create_definition() {
        let create_def_cmd = create_def_cmd_1();
        SimpleTestHarness::given([])
            .when(create_def_cmd)
            .then_assert(|events| {
                assert_eq!(events.len(), 1);
                let event = &events[0];
                if let DomainEvent::DefCreated {
                    id: def_id,
                    title,
                    created_by,
                    json_schema_string,
                    ..
                } = event
                {
                    assert_eq!(def_id, &generate_id_from_title("test_title"));
                    assert_eq!(title, "test_title");
                    assert_eq!(created_by, "test_created_by");
                    assert_eq!(json_schema_string, &get_valid_json_string());
                } else {
                    assert!(
                        matches!(event, DomainEvent::DefCreated { .. }),
                        "Event is not of type DomainEvent::DefCreated"
                    );
                }
            });
    }

    #[test]
    fn test_create_definition_with_then() {
        let create_def_cmd = create_def_cmd_1();
        SimpleTestHarness::given([])
            .when(create_def_cmd)
            .then_assert(|events| {
                assert_eq!(events.len(), 1);
                let event = &events[0];
                if let DomainEvent::DefCreated {
                    id: def_id,
                    title,
                    created_by,
                    json_schema_string,
                    ..
                } = event
                {
                    assert_eq!(def_id, &generate_id_from_title("test_title"));
                    assert_eq!(title, "test_title");
                    assert_eq!(created_by, "test_created_by");
                    assert_eq!(json_schema_string, &get_valid_json_string());
                } else {
                    assert!(
                        matches!(event, DomainEvent::DefCreated { .. }),
                        "Event is not of type DomainEvent::DefCreated"
                    );
                }
            });
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
        disintegrate::TestHarness::given([def_created_valid_json_draft()])
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
    fn test_mutate_tile_should_fail() {
        disintegrate::TestHarness::given([
            def_created_valid_json_draft(),
            def_validated_valid_json(),
            def_activated_valid_json(),
        ])
        .when(get_update_def_cmd_mutate())
        .then_err(TitleIsNotMutable(
            "example_schema".to_string(),
            "test_title".to_string(),
        ));
    }

    #[test]
    fn test_update_with_valid_schema_should_fail() {
        SimpleTestHarness::given([def_created_valid_json_draft()])
            .when(get_update_def_cmd_mutate())
            .then_err(ModifyNotAllowed(DefRecordStatus::Draft));
    }

    #[test]
    fn test_update_with_valid_schema_should_succeed() {
        SimpleTestHarness::given([
            def_created_valid_json_draft(),
            def_validated_valid_json(),
            def_activated_valid_json(),
        ])
        .when(get_update_title_def_cmd())
        .then_assert(|events| {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            if let DomainEvent::DefUpdated {
                id: def_id,
                title,
                updated_by,
                json_schema_string,
                ..
            } = event
            {
                assert_eq!(def_id, &generate_id_from_title("test_title"));
                assert_eq!(title, "test_title");
                assert_eq!(updated_by, "test_updated_by");
                assert_eq!(json_schema_string, &get_updated_json_string_test_title());
            } else {
                assert!(
                    matches!(event, DomainEvent::DefUpdated { .. }),
                    "Event is not of type DomainEvent::DefUpdated"
                );
            }
        });
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
