use std::fs;
use std::path::Path;

mod common;

/// Reads the student JSON schema from 'tests/resources/schemas/Student_Schema.json'
pub fn read_student_schema() -> std::io::Result<String> {
    let path = Path::new("tests/resources/schemas/Student_Schema.json");
    fs::read_to_string(path)
}
#[cfg(test)]
mod test {
    use crate::common::test_harness::SimpleTestHarness;
    use crate::common::{
        get_create_entity_cmd, get_create_entity_cmd_with_invalid_student,
        get_def_activated_valid_student_json, get_def_created_valid_student_json,
        get_def_validated_valid_student_json,
    };
    use crate::read_student_schema;
    use definitions_core::definitions_domain::{
        generate_id_from_title, DefRecordStatus, DomainEvent,
    };
    use definitions_core::registry_domain::EntityError;

    #[test]
    fn test_create_entity() {
        let create_entity_cmd = get_create_entity_cmd();
        SimpleTestHarness::given([
            get_def_created_valid_student_json(),
            get_def_validated_valid_student_json(),
            get_def_activated_valid_student_json(),
        ])
        .when(create_entity_cmd)
        .then_assert(|events| {
            assert_eq!(events.len(), 1);
            let event = &events[0];
            println!("EntityCreated: {:#?}", event);
            if let DomainEvent::EntityCreated {
                registry_def_id: def_id,
                created_by,
                ..
            } = event
            {
                assert_eq!(def_id, &generate_id_from_title("Student"));
                assert_eq!(created_by, "test_user");
            } else {
                assert!(
                    matches!(event, DomainEvent::EntityCreated { .. }),
                    "Event is not of type DomainEvent::EntityCreated"
                );
            }
        });
    }

    #[test]
    fn test_create_entity_should_fail_if_json_is_invalid() {
        let create_entity_cmd = get_create_entity_cmd_with_invalid_student();
        SimpleTestHarness::given([
            get_def_created_valid_student_json(),
            get_def_validated_valid_student_json(),
            get_def_activated_valid_student_json(),
        ])
        .when(create_entity_cmd)
        .then_err_assert(|entity_error| match entity_error {
            EntityError::JsonSchemaError(ref entity_name, ref error_message) => {
                assert_eq!(entity_name, "Student");
                assert!(
                    error_message.contains("fullName") && error_message.contains("gender"),
                    "Error message does not contain required keywords: fullName and gender"
                );
                assert!(
                    error_message.contains("100") && error_message.contains("Child"),
                    "Error message does not contain required keywords: 100 and Child"
                );
            }
            other => panic!("Expected EntityError::JsonSchemaError, got: {:?}", other),
        });
    }
    #[test]
    fn test_create_entity_should_fail_if_definition_is_not_active() {
        let create_entity_cmd = get_create_entity_cmd();
        SimpleTestHarness::given([get_def_created_valid_student_json()])
            .when(create_entity_cmd.clone())
            .then_err(EntityError::DefinitionNotInProperState(
                DefRecordStatus::Active,
                DefRecordStatus::Draft,
            ));

        SimpleTestHarness::given([
            get_def_created_valid_student_json(),
            get_def_validated_valid_student_json(),
        ])
        .when(create_entity_cmd)
        .then_err(EntityError::DefinitionNotInProperState(
            DefRecordStatus::Active,
            DefRecordStatus::Valid,
        ));
    }

    #[test]
    fn simple_json_schema_test() -> anyhow::Result<()> {
        let student_json_schema = read_student_schema()?;
        let student_json_instance = r###"
        {
          "Student": {
            "identityDetails": {
              "fullName": "John",
              "gender": "Male"
            }
          }
        }
                "###
        .to_string();

        let schema: serde_json::Value = serde_json::from_str(&student_json_schema)?;
        let instance: serde_json::Value = serde_json::from_str(&student_json_instance)?;

        let validator = jsonschema::draft7::new(&schema)?;

        for error in validator.iter_errors(&instance) {
            eprintln!("Error: {}", error);
            eprintln!("Location: {}", error.instance_path);
        }

        Ok(())
    }
}
